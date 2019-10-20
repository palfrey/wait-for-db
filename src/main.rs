use std::io;
use structopt::StructOpt;
use odbc::{create_environment_v3, DiagnosticRecord, Connection, Statement, Data, NoData};
use std::ffi::CStr;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opts {
}

#[derive(Debug, PartialEq)]
enum DbErrorType {
    Permenant,
    Temporary
}

#[derive(Debug)]
struct DbError {
    kind: DbErrorType,
    error: failure::Error
}

impl From<DiagnosticRecord> for DbError {
    fn from(item: DiagnosticRecord) -> Self {
        let state = CStr::from_bytes_with_nul(item.get_raw_state()).unwrap().to_str().unwrap();
        let kind = if state == "IM002" {
            DbErrorType::Permenant
        } else {
            DbErrorType::Temporary
        };
        DbError {
            kind: kind,
            error: std::convert::From::from(item)
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

    let conn = env.connect_with_connection_string("foo")?;
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

#[test]
fn test_connect_no_driver() {
    let err = connect(Opts{}).unwrap_err();
    assert_eq!(err.kind, DbErrorType::Permenant);
}