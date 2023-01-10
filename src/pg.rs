use crate::common::{DbError, DbErrorLifetime, DbErrorType, Opts};
use postgres::Client;
use regex::Regex;
use rustls::client::HandshakeSignatureValid;
use rustls::DigitallySignedStruct;
use rustls::{
    client::{ServerCertVerified, ServerCertVerifier},
    Certificate, ServerName,
};
use std::error::Error;
use std::sync::Arc;
use std::{collections::HashMap, time::SystemTime};

impl From<&postgres::error::DbError> for DbError {
    fn from(e: &postgres::error::DbError) -> Self {
        let kind = if *e.code() == postgres::error::SqlState::SYNTAX_ERROR {
            DbErrorLifetime::Permanent
        } else {
            DbErrorLifetime::Temporary
        };
        DbError {
            kind,
            error: DbErrorType::PostgresError {
                error: Box::new(e.clone()),
            },
        }
    }
}

impl From<postgres::error::Error> for DbError {
    #[rustfmt::skip]
    fn from(e: postgres::error::Error) -> Self {
        if let Some(error) = e.source() {
            if let Some(dberror) = error.downcast_ref::<postgres::error::DbError>() {
                return dberror.into();
            }
        }

        // FIXME: Hack because https://github.com/sfackler/rust-postgres/issues/583
        let dump = format!("{:?}", e);
        let kind_re = Regex::new(r"kind: ([A-Za-z]+)").unwrap();
        let lifetime = if let Some(captures) = kind_re.captures(&dump) {
            match &captures[1] {
                "ConfigParse" => DbErrorLifetime::Permanent,
                _ => DbErrorLifetime::Temporary
            }
        } else {
            DbErrorLifetime::Temporary
        };

        DbError {
            kind: lifetime,
            error: DbErrorType::PostgresError { error: Box::new(e) },
        }
    }
}

// Dummy "just let everything through" cert verifier
struct PassEverythingVerifier {}

impl ServerCertVerifier for PassEverythingVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &Certificate,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &Certificate,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    fn request_scts(&self) -> bool {
        false
    }
}

pub fn connect(opts: &Opts) -> std::result::Result<Vec<HashMap<String, String>>, DbError> {
    let mut config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(rustls::RootCertStore::empty())
        .with_no_client_auth();
    config
        .dangerous()
        .set_certificate_verifier(Arc::new(PassEverythingVerifier {}));
    let connector = tokio_postgres_rustls::MakeRustlsConnect::new(config);
    let mut conn = Client::connect(opts.connection_string.as_str(), connector)?;
    if let Some(ref sql_query) = opts.sql_query {
        execute_statement(&mut conn, sql_query)
    } else {
        Ok(Vec::new())
    }
}

fn execute_statement(
    conn: &mut postgres::Client,
    sql_query: &str,
) -> Result<Vec<HashMap<String, String>>, DbError> {
    let mut results: Vec<HashMap<String, String>> = Vec::new();
    let rows = conn.query(sql_query, &[])?;
    for row in rows.iter() {
        let mut result: HashMap<String, String> = HashMap::new();
        let cols = row.columns();
        for (i, col) in cols.iter().enumerate() {
            let val = row
                .try_get::<usize, String>(i)
                .unwrap_or_else(|_| "".to_string());
            result.insert(col.name().to_string(), val);
        }
        results.push(result);
    }

    Ok(results)
}

// only for tests
#[doc(hidden)]
pub fn postgres_connect(sslmode: &str) -> String {
    format!(
        "postgresql://{}:{}@{}:{}?sslmode={}",
        std::env::var("POSTGRES_USERNAME").unwrap(),
        std::env::var("POSTGRES_PASSWORD").unwrap(),
        std::env::var("POSTGRES_SERVER").unwrap(),
        std::env::var("POSTGRES_PORT").unwrap(),
        sslmode
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_postgres_with_no_server() {
        let err = connect(&Opts::new().connection_string("postgresql://")).unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Temporary, "{:?}", err);
        if let DbErrorType::OdbcError { error } = err.error {
            let desc = format!("{}", error);
            assert!(
                desc.contains("could not connect to server: No such file or directory"),
                "{}",
                desc
            );
        }
    }

    #[test]
    fn test_postgres_with_bad_url() {
        let err = connect(&Opts::new().connection_string("postgresql://test:test@localhost:port"))
            .unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Permanent, "{:?}", err);
    }

    #[test]
    #[cfg_attr(postgres_driver = "", ignore)]
    fn test_postgres_with_server() {
        connect(
            &Opts::new()
                .connection_string(postgres_connect("disable"))
                .sql_query("SHOW IS_SUPERUSER"),
        )
        .unwrap();
    }

    #[test]
    #[cfg_attr(postgres_driver = "", ignore)]
    fn test_postgres_with_secure_server() {
        connect(
            &Opts::new()
                .connection_string(postgres_connect("require"))
                .sql_query("SHOW IS_SUPERUSER"),
        )
        .unwrap();
    }

    #[test]
    #[cfg_attr(postgres_driver = "", ignore)]
    fn test_postgres_with_bad_query() {
        let err = connect(
            &Opts::new()
                .connection_string(postgres_connect("disable"))
                .sql_query("foobar"),
        )
        .unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Permanent, "{:?}", err);
        if let DbErrorType::OdbcError { error } = err.error {
            let desc = format!("{}", error);
            assert!(
                desc.contains("ERROR: syntax error at or near \"foobar\""),
                "{}",
                desc
            );
        }
    }
}
