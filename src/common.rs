use clap::{App, Arg};
use odbc::DiagnosticRecord;

#[derive(PartialEq)]
pub enum DbMode {
    Odbc,
    Postgres,
}

impl DbMode {
    fn from_str(s: &str) -> DbMode {
        if s.to_lowercase() == "odbc" {
            DbMode::Odbc
        } else {
            DbMode::Postgres
        }
    }
}

pub struct Opts {
    pub mode: DbMode,
    pub connection_string: String,
    pub sql_text: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub quiet: bool,
    pub pause_seconds: u64,
}

pub fn parse_args() -> Opts {
    let matches = App::new("wait-for-db")
        .arg(
            Arg::with_name("mode")
                .short("m")
                .long("mode")
                .required(true)
                .takes_value(true)
                .possible_values(&["odbc", "postgres"])
                .help("Database mode"),
        )
        .arg(
            Arg::with_name("connection-string")
                .short("c")
                .long("connection-string")
                .required(true)
                .takes_value(true)
                .help("Connection string"),
        )
        .arg(
            Arg::with_name("sql-text")
                .short("s")
                .long("sql-text")
                .takes_value(true)
                .help("SQL Query (default: no query)"),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .takes_value(true)
                .help("Timeout (seconds)"),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Quiet mode"),
        )
        .arg(
            Arg::with_name("pause")
                .short("p")
                .long("pause")
                .takes_value(true)
                .help("Pause between checks (seconds)")
                .default_value("3"),
        )
        .get_matches();
    Opts {
        mode: DbMode::from_str(matches.value_of("mode").unwrap()),
        connection_string: matches.value_of("connection-string").unwrap().to_string(),
        sql_text: matches
            .value_of("sql-text")
            .and_then(|s| Some(s.to_string())),
        timeout_seconds: matches
            .value_of("timeout")
            .and_then(|s| u64::from_str_radix(s, 10).ok()),
        quiet: matches.is_present("quiet"),
        pause_seconds: matches
            .value_of("pause")
            .and_then(|s| u64::from_str_radix(s, 10).ok())
            .unwrap(),
    }
}

#[cfg(test)]
impl Opts {
    pub fn new() -> Self {
        Opts {
            mode: DbMode::Odbc,
            connection_string: "".to_string(),
            sql_text: None,
            timeout_seconds: None,
            quiet: false,
            pause_seconds: 3,
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
        self.sql_text = Some(st.into());
        self
    }
}

#[derive(Debug, PartialEq)]
pub enum DbErrorLifetime {
    Permanent,
    Temporary,
}

#[derive(Debug)]
pub enum DbErrorType {
    OdbcError { error: DiagnosticRecord },
    PostgresError { error: postgres::Error },
}

#[derive(Debug)]
pub struct DbError {
    pub kind: DbErrorLifetime,
    pub error: DbErrorType,
}
