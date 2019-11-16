use crate::common::{DbError, DbErrorLifetime, DbErrorType, Opts};
use postgres::{Connection, Error, TlsMode};
use std::collections::HashMap;

impl From<Error> for DbError {
    #[rustfmt::skip]
    fn from(item: Error) -> Self {
        let kind = if item.as_connection().is_some() || item.as_db().and_then(|e| if e.code == postgres::error::SYNTAX_ERROR { Some(()) } else { None }).is_some() {
            DbErrorLifetime::Permanent
        } else {
            DbErrorLifetime::Temporary
        };
        DbError {
            kind: kind,
            error: DbErrorType::PostgresError { error: item },
        }
    }
}

pub fn connect(opts: &Opts) -> std::result::Result<Vec<HashMap<String, String>>, DbError> {
    let conn = Connection::connect(opts.connection_string.as_str(), TlsMode::None)?;
    if let Some(ref sql_text) = opts.sql_text {
        execute_statement(&conn, sql_text)
    } else {
        return Ok(Vec::new());
    }
}

fn execute_statement<'env>(
    conn: &postgres::Connection,
    sql_text: &String,
) -> Result<Vec<HashMap<String, String>>, DbError> {
    let mut results: Vec<HashMap<String, String>> = Vec::new();
    let rows = conn.query(&sql_text, &[])?;
    let cols = rows.columns();
    for row in rows.iter() {
        let mut result: HashMap<String, String> = HashMap::new();
        for i in 0..cols.len() {
            let val = row
                .get_opt::<usize, String>(i)
                .unwrap_or(Ok("".to_string()));
            result.insert(cols[i].name().to_string(), val.unwrap_or("".to_string()));
        }
        results.push(result);
    }

    Ok(results)
}

// only for tests
#[doc(hidden)]
pub fn postgres_connect() -> String {
    format!(
        "postgresql://{}:{}@{}:{}",
        std::env::var("POSTGRES_USERNAME").unwrap(),
        std::env::var("POSTGRES_PASSWORD").unwrap(),
        std::env::var("POSTGRES_SERVER").unwrap(),
        std::env::var("POSTGRES_PORT").unwrap(),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg_attr(postgres_driver = "", ignore)]
    fn test_postgres_with_no_server() {
        let err = connect(&Opts::new().connection_string("postgresql://")).unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Temporary, "{:?}", err);
        if let DbErrorType::OdbcError { error } = err.error {
            let desc = format!("{}", error);
            assert!(
                desc.contains("could not connect to server: No such file or directory"),
                desc
            );
        }
    }

    #[test]
    #[cfg_attr(postgres_driver = "", ignore)]
    fn test_postgres_with_server() {
        connect(
            &Opts::new()
                .connection_string(postgres_connect())
                .sql_text("SHOW IS_SUPERUSER"),
        )
        .unwrap();
    }

    #[test]
    #[cfg_attr(postgres_driver = "", ignore)]
    fn test_postgres_with_bad_query() {
        let err = connect(
            &Opts::new()
                .connection_string(postgres_connect())
                .sql_text("foobar"),
        )
        .unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Permanent, "{:?}", err);
        if let DbErrorType::OdbcError { error } = err.error {
            let desc = format!("{}", error);
            assert!(
                desc.contains("ERROR: syntax error at or near \"foobar\""),
                desc
            );
        }
    }
}
