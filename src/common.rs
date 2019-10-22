use failure::Fail;
use odbc::DiagnosticRecord;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opts {
    #[structopt(required = true, long, short, help = "Connection string")]
    pub connection_string: String,

    #[structopt(long, short, help = "SQL Query (default: no query)")]
    pub sql_text: String,

    #[structopt(short = "t", long = "timeout", help = "Timeout (seconds)")]
    pub timeout_seconds: Option<u64>,

    #[structopt(short, long, help = "Quiet mode")]
    pub quiet: bool,
}

#[cfg(test)]
impl Opts {
    pub fn new() -> Self {
        Opts {
            connection_string: "".to_string(),
            sql_text: "".to_string(),
            timeout_seconds: None,
            quiet: false,
        }
    }

    pub fn connection_string<I>(mut self, cs: I) -> Self
    where
        I: Into<String>,
    {
        self.connection_string = cs.into();
        self
    }

    pub fn sql_text<I>(mut self, st: I) -> Self
    where
        I: Into<String>,
    {
        self.sql_text = st.into();
        self
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
