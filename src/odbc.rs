use odbc::{create_environment_v3, Connection, Data, DiagnosticRecord, NoData, Statement};
use std::ffi::CStr;

use crate::common::{DbError, DbErrorLifetime, DbErrorType, Opts};
use std::collections::HashMap;

impl From<DiagnosticRecord> for DbError {
    #[rustfmt::skip]
    fn from(item: DiagnosticRecord) -> Self {
        let state = CStr::from_bytes_with_nul(item.get_raw_state())
            .unwrap()
            .to_str()
            .unwrap();
        let kind = if state == "IM002"  // no driver
                    || state == "01000" // no driver at path
                    || state == "IM004" // bad driver
                    || state == "42601" // bad query
        {
            DbErrorLifetime::Permanent
        } else {
            DbErrorLifetime::Temporary
        };
        DbError {
            kind: kind,
            error: DbErrorType::OdbcError { error: item },
        }
    }
}

pub fn connect(opts: &Opts) -> std::result::Result<Vec<HashMap<String, String>>, DbError> {
    let env = create_environment_v3().map_err(|e| e.unwrap())?;
    let conn = env.connect_with_connection_string(&opts.connection_string)?;
    if let Some(ref sql_text) = opts.sql_text {
        execute_statement(&conn, sql_text)
    } else {
        return Ok(Vec::new());
    }
}

fn execute_statement<'env>(
    conn: &Connection<'env>,
    sql_text: &String,
) -> Result<Vec<HashMap<String, String>>, DbError> {
    let stmt = Statement::with_parent(conn)?;
    let mut results: Vec<HashMap<String, String>> = Vec::new();
    match stmt.exec_direct(&sql_text)? {
        Data(mut stmt) => {
            let col_count = stmt.num_result_cols()? as u16;
            let mut cols: HashMap<u16, odbc::ColumnDescriptor> = HashMap::new();
            for i in 1..(col_count + 1) {
                cols.insert(i, stmt.describe_col(i)?);
            }
            while let Some(mut cursor) = stmt.fetch()? {
                let mut result: HashMap<String, String> = HashMap::new();
                for i in 1..(col_count + 1) {
                    match cursor.get_data::<String>(i as u16)? {
                        Some(val) => {
                            result.insert(cols[&i].name.clone(), val);
                        }
                        None => {
                            result.insert(cols[&i].name.clone(), "".to_string());
                        }
                    }
                }
                results.push(result);
            }
        }
        NoData(_) => println!("Query executed, no data returned"),
    }

    Ok(results)
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
            assert!(desc.contains("Can't open lib 'foo' : file not found"), desc);
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
                desc.contains("Driver's SQLAllocHandle on SQL_HANDLE_HENV failed"),
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
                .sql_text("SELECT 1 from sqlite_master"),
        )
        .unwrap();
    }

    #[test]
    #[cfg_attr(postgres_driver = "", ignore)]
    fn test_postgres_with_no_server() {
        let err = connect(&Opts::new().connection_string(format!(
            "Driver={};",
            std::env::var("POSTGRES_DRIVER").unwrap()
        )))
        .unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Temporary, "{:?}", err);
        if let DbErrorType::OdbcError { error } = err.error {
            let desc = format!("{}", error);
            assert!(
                desc.contains("could not connect to server: No such file or directory"),
                desc
            );
        }
    }

    fn postgres_connect() -> String {
        format!(
            "Driver={};Server={};Port={};Uid={};Pwd={};",
            std::env::var("POSTGRES_DRIVER").unwrap(),
            std::env::var("POSTGRES_SERVER").unwrap(),
            std::env::var("POSTGRES_PORT").unwrap(),
            std::env::var("POSTGRES_USERNAME").unwrap(),
            std::env::var("POSTGRES_PASSWORD").unwrap(),
        )
    }

    #[test]
    #[cfg_attr(postgres_driver = "", ignore)]
    fn test_postgres_with_server() {
        connect(
            &Opts::new()
                .connection_string(postgres_connect())
                .sql_text("SHOW config_file"),
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
