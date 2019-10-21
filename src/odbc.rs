use odbc::{create_environment_v3, Connection, Data, DiagnosticRecord, NoData, Statement};
use std::ffi::CStr;

use crate::common::{DbError, DbErrorLifetime, DbErrorType, Opts};

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
                    || state == "08001" // can't connect
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

pub fn connect(opts: Opts) -> std::result::Result<(), DbError> {
    let env = create_environment_v3().map_err(|e| e.unwrap())?;
    let conn = env.connect_with_connection_string(&opts.connection_string)?;
    execute_statement(&conn, opts)
}

fn execute_statement<'env>(conn: &Connection<'env>, opts: Opts) -> Result<(), DbError> {
    let stmt = Statement::with_parent(conn)?;

    match stmt.exec_direct(&opts.sql_text)? {
        Data(mut stmt) => {
            let cols = stmt.num_result_cols()?;
            while let Some(mut cursor) = stmt.fetch()? {
                for i in 1..(cols + 1) {
                    match cursor.get_data::<&str>(i as u16)? {
                        Some(val) => print!(" {}", val),
                        None => print!(" NULL"),
                    }
                }
                println!("");
            }
        }
        NoData(_) => println!("Query executed, no data returned"),
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_connect_no_driver() {
        let err = connect(Opts::new()).unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Permanent, "{:?}", err);
    }

    #[test]
    fn test_connect_with_missing_driver() {
        let err = connect(Opts::new().connection_string("Driver=foo;Server=blah;")).unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Permanent, "{:?}", err);
        if let DbErrorType::OdbcError { error } = err.error {
            let desc = format!("{}", error);
            assert!(desc.contains("Can't open lib 'foo' : file not found"), desc);
        }
    }

    #[test]
    fn test_connect_with_bad_driver() {
        let err = connect(Opts::new().connection_string(format!(
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
            Opts::new()
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
        let err = connect(Opts::new().connection_string(format!(
            "Driver={};",
            std::env::var("POSTGRES_DRIVER").unwrap()
        )))
        .unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Permanent, "{:?}", err);
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
            Opts::new()
                .connection_string(format!(
                    "Driver={};Server={};Port={};Uid={};Pwd={};",
                    std::env::var("POSTGRES_DRIVER").unwrap(),
                    std::env::var("POSTGRES_SERVER").unwrap(),
                    std::env::var("POSTGRES_PORT").unwrap(),
                    std::env::var("POSTGRES_USERNAME").unwrap(),
                    std::env::var("POSTGRES_PASSWORD").unwrap(),
                ))
                .sql_text("SHOW config_file"),
        )
        .unwrap();
    }
}
