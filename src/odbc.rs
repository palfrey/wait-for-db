use odbc_api::{
    ColumnDescription, Connection, ConnectionOptions, Cursor, DataType, Environment, Nullability, ResultSetMetadata, buffers::TextRowSet, Error
};

use crate::common::{DbError, DbErrorLifetime, DbErrorType, Opts};
use std::collections::HashMap;

impl From<Error> for DbError {
    fn from(item: Error) -> Self {
        if let Error::Diagnostics{record, function: _} = item {
            let state = str::from_utf8(&record.state.0)
                .unwrap()
                .trim_matches(char::from(0))
                .to_string();

            let kind = match state.as_str() {
                "IM002"  // no driver
                | "01000" // no driver at path
                | "IM004" // bad driver
                | "42601" // bad query                
                => DbErrorLifetime::Permanent,
                _ => DbErrorLifetime::Temporary,
            };
            return DbError {
                kind,
                error: DbErrorType::OdbcError { error: record },
            };
        }
        unimplemented!()
    }
}

pub fn connect(opts: &Opts) -> std::result::Result<Vec<HashMap<String, String>>, DbError> {
    // We know this is going to be the only ODBC environment in the entire process, so this is
    // safe.
    let env =  Environment::new()?;
    let conn = env.connect_with_connection_string(&opts.connection_string, ConnectionOptions::default())?;
    if let Some(ref sql_query) = opts.sql_query {
        execute_statement(&conn, sql_query)
    } else {
        Ok(Vec::new())
    }
}

const BATCH_SIZE: usize = 100000;

fn execute_statement(
    conn: &Connection,
    sql_query: &str,
) -> Result<Vec<HashMap<String, String>>, DbError> {
    let mut results: Vec<HashMap<String, String>> = Vec::new();
    match conn.execute(sql_query, (), None)? {
        Some(mut cursor) => {
            let col_count = cursor.num_result_cols()? as u16;
            let mut cols: HashMap<usize, String> = HashMap::new();
            for i in 1..(col_count + 1) {
                let mut cd = ColumnDescription {
                    name: Vec::new(),
                    data_type: DataType::Unknown,
                    nullability: Nullability::Unknown,
                };
                cursor.describe_col(i, &mut cd)?;
                cols.insert((i - 1).into(), cd.name_to_string().unwrap());
            }
            let mut buffers = TextRowSet::for_cursor(BATCH_SIZE, &mut cursor, None)?;
            let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;
            while let Some(batch) = row_set_cursor.fetch()? {
                // Within a batch, iterate over every row
                for row_index in 0..batch.num_rows() {
                    let mut result: HashMap<String, String> = HashMap::new();
                    for col_index in 0..batch.num_cols() {
                        let item = batch.at(col_index, row_index);
                        match item {
                            Some(val) => {
                                result.insert(
                                    cols[&col_index].clone(),
                                    std::str::from_utf8(val).unwrap().to_string(),
                                );
                            }
                            None => {
                                result.insert(cols[&col_index].clone(), "".to_string());
                            }
                        }
                    }
                    results.push(result);
                }
            }
        }
        None => println!("Query executed, no data returned"),
    }

    Ok(results)
}

// only for tests
#[doc(hidden)]
pub fn postgres_connect() -> String {
    format!(
        "Driver={};Server={};Port={};Uid={};Pwd={};",
        std::env::var("POSTGRES_DRIVER").unwrap(),
        std::env::var("POSTGRES_SERVER").unwrap(),
        std::env::var("POSTGRES_PORT").unwrap(),
        std::env::var("POSTGRES_USERNAME").unwrap(),
        std::env::var("POSTGRES_PASSWORD").unwrap(),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_connect_no_driver() {
        let err = connect(&Opts::new()).unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Permanent, "{:?}", err);
    }

    #[test]
    fn test_connect_with_missing_driver() {
        let err = connect(&Opts::new().connection_string("Driver=foo;Server=blah;")).unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Permanent, "{:?}", err);
        if let DbErrorType::OdbcError { error } = err.error {
            let desc = format!("{}", error);
            assert!(
                desc.contains("Can't open lib 'foo' : file not found"),
                "{}",
                desc
            );
        }
    }

    #[test]
    fn test_connect_with_bad_driver() {
        let err = connect(&Opts::new().connection_string(format!(
            "Driver={};Server=blah;",
            std::env::current_exe().unwrap().to_str().unwrap()
        )))
        .unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Permanent, "{:?}", err);
        if let DbErrorType::OdbcError { error } = err.error {
            let desc = format!("{}", error);
            assert!(
                desc.contains("State: 01000")  // Also says "file not found", which is wrong, but what unixodbc says for "no symbols" for some reason
                 || desc.contains("Driver's SQLAllocHandle on SQL_HANDLE_HENV failed"), // Right error message, but which one turns up depends on the odbc version
                "{}",
                desc
            );
        }
    }

    #[test]
    #[cfg_attr(sqlite_driver = "", ignore)]
    fn test_sqlite_with_good_driver() {
        connect(
            &Opts::new()
                .connection_string(format!(
                    "Driver={};Database=test.db;",
                    std::env::var("SQLITE_DRIVER").unwrap()
                ))
                .sql_query("SELECT 1 from sqlite_master"),
        )
        .unwrap();
    }

    #[test]
    #[cfg_attr(postgres_driver = "", ignore)]
    fn test_postgres_with_wrong_server() {
        let err = connect(&Opts::new().connection_string(format!(
            "Driver={};Server=localhost;Port=9000",
            std::env::var("POSTGRES_DRIVER").unwrap()
        )))
        .unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Temporary, "{:?}", err);
        if let DbErrorType::OdbcError { error } = err.error {
            let desc = format!("{}", error);
            assert!(desc.contains("Connection refused"), "{}", desc);
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
                "{}",
                desc
            );
        }
    }
}
