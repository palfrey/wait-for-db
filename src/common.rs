use failure::Fail;
use odbc::DiagnosticRecord;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opts {
    pub connection_string: String,
    pub sql_text: String,
}

impl Default for Opts {
    fn default() -> Self {
        Opts {
            connection_string: "".to_string(),
            sql_text: "".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum DbErrorLifetime {
    Permanent,
    Temporary,
}

#[derive(Debug, Fail)]
pub enum DbErrorType {
    #[fail(display = "odbc error: {}", error)]
    OdbcError { error: DiagnosticRecord },

    #[fail(display = "postgres error")]
    PostgresError,
}

#[derive(Debug)]
pub struct DbError {
    pub kind: DbErrorLifetime,
    pub error: DbErrorType,
}
