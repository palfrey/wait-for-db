use std::io;
use structopt::StructOpt;
use odbc::{create_environment_v3, DiagnosticRecord, Connection, Statement, Data, NoData};
use std::ffi::CStr;
use failure::Fail;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opts {
    connection_string: String,
}

impl Default for Opts {
    fn default() -> Self {
        Opts {
            connection_string: "".to_string()
        }
    }
}

#[derive(Debug, PartialEq)]
enum DbErrorLifetime {
    Permenant,
    Temporary
}

#[derive(Debug, Fail)]
enum DbErrorType {
    #[fail(display = "odbc error: {}", error)]
    OdbcError {
        error: DiagnosticRecord
    },

    #[fail(display = "postgres error")]
    PostgresError
}

#[derive(Debug)]
struct DbError {
    kind: DbErrorLifetime,
    error: DbErrorType
}

impl From<DiagnosticRecord> for DbError {
    fn from(item: DiagnosticRecord) -> Self {
        let state = CStr::from_bytes_with_nul(item.get_raw_state()).unwrap().to_str().unwrap();
        let kind = if state == "IM002" || state == "01000" || state == "IM004" {
            DbErrorLifetime::Permenant
        } else {
            DbErrorLifetime::Temporary
        };
        println!("{:?}", item);
        DbError {
            kind: kind,
            error: DbErrorType::OdbcError {error: item}
        }
    }
}

fn main() {
    let opt = Opts::from_args();
    env_logger::init();

    match connect(opt) {
        Ok(()) => println!("Success"),
        Err(diag) => println!("Error: {:?}", diag),
    }
}

fn connect(opts: Opts) -> std::result::Result<(), DbError> {
    let env = create_environment_v3().map_err(|e| e.unwrap())?;
    let conn = env.connect_with_connection_string(&opts.connection_string)?;
    execute_statement(&conn)
}

fn execute_statement<'env>(conn: &Connection<'env>) -> Result<(), DbError> {
    let stmt = Statement::with_parent(conn)?;

    let mut sql_text = String::new();
    println!("Please enter SQL statement string: ");
    io::stdin().read_line(&mut sql_text).unwrap();

    match stmt.exec_direct(&sql_text)? {
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
        let err = connect(Opts::default()).unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Permenant, "{:?}", err);
    }

    #[test]
    fn test_connect_with_missing_driver() {
        let err = connect(Opts{connection_string: "Driver=foo;Server=blah;".to_string()}).unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Permenant, "{:?}", err);
        if let DbErrorType::OdbcError {error} = err.error {
            let desc = format!("{}", error);
            assert!(desc.contains("Can't open lib 'foo' : file not found"), desc);
        }
    }

    #[test]
    fn test_connect_with_bad_driver() {
        let err = connect(Opts{connection_string: format!("Driver={};Server=blah;", std::env::current_exe().unwrap().to_str().unwrap())}).unwrap_err();
        assert_eq!(err.kind, DbErrorLifetime::Permenant, "{:?}", err);
        if let DbErrorType::OdbcError {error} = err.error {
            let desc = format!("{}", error);
            assert!(desc.contains("Driver's SQLAllocHandle on SQL_HANDLE_HENV failed"), desc);
        }
    }
}