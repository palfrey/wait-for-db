use crate::common::{DbError, DbErrorLifetime, DbErrorType, Opts};
use postgres::{Client, NoTls};
use std::collections::HashMap;
use std::error::Error;

impl From<&postgres::error::DbError> for DbError {
    #[rustfmt::skip]
    fn from(e: &postgres::error::DbError) -> Self {
        let kind = if *e.code() == postgres::error::SqlState::SYNTAX_ERROR {
            DbErrorLifetime::Permanent
        } else {
            DbErrorLifetime::Temporary
        };
        println!("e.code: {:?}", e.code());
        DbError {
            kind: kind,
            error: DbErrorType::PostgresError { error: Box::new(e.clone()) },
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
        DbError {
            kind: DbErrorLifetime::Temporary,
            error: DbErrorType::PostgresError { error: Box::new(e) },
        }
    }
}

pub fn connect(opts: &Opts) -> std::result::Result<Vec<HashMap<String, String>>, DbError> {
    let mut conn = Client::connect(opts.connection_string.as_str(), NoTls)?;
    if let Some(ref sql_query) = opts.sql_query {
        execute_statement(&mut conn, sql_query)
    } else {
        return Ok(Vec::new());
    }
}

fn execute_statement<'env>(
    conn: &mut postgres::Client,
    sql_query: &String,
) -> Result<Vec<HashMap<String, String>>, DbError> {
    let mut results: Vec<HashMap<String, String>> = Vec::new();
    let rows = conn.query(sql_query.as_str(), &[])?;
    for row in rows.iter() {
        let mut result: HashMap<String, String> = HashMap::new();
        let cols = row.columns();
        for i in 0..cols.len() {
            let val = row
                .try_get::<usize, String>(i)
                .unwrap_or("".to_string());
            result.insert(cols[i].name().to_string(), val);
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
                .sql_query("SHOW IS_SUPERUSER"),
        )
        .unwrap();
    }

    #[test]
    #[cfg_attr(postgres_driver = "", ignore)]
    fn test_postgres_with_bad_query() {
        let err = connect(
            &Opts::new()
                .connection_string(postgres_connect())
                .sql_query("foobar"),
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
