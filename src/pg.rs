use crate::common::{DbError, DbErrorLifetime, DbErrorType, Opts};
use postgres::{Connection, Error, TlsMode};
use std::collections::HashMap;

impl From<Error> for DbError {
    #[rustfmt::skip]
    fn from(item: Error) -> Self {
        let kind = if item.as_connection().is_some() {
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
