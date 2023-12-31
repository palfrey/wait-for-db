use clap::{App, Arg};
use odbc_api::handles::Record;
use url::ParseError;

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
    pub sql_query: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub quiet: bool,
    pub pause_seconds: u64,
}

pub fn parse_args() -> Opts {
    let matches = App::new("wait-for-db")
        .version(clap::crate_version!())
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
            Arg::with_name("sql-query")
                .short("s")
                .long("sql-query")
                .takes_value(true)
                .help("SQL query that should return at least one row (default: no querying)"),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .takes_value(true)
                .help("Timeout in seconds (default: wait forever)"),
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
        sql_query: matches.value_of("sql-query").map(|s| s.to_string()),
        timeout_seconds: matches
            .value_of("timeout")
            .and_then(|s| s.parse::<u64>().ok()),
        quiet: matches.is_present("quiet"),
        pause_seconds: matches
            .value_of("pause")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap(),
    }
}

#[cfg(test)]
impl Opts {
    pub fn new() -> Self {
        Opts {
            mode: DbMode::Odbc,
            connection_string: "".to_string(),
            sql_query: None,
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

    pub fn sql_query<I>(mut self, st: I) -> Self
    where
        I: Into<String>,
    {
        self.sql_query = Some(st.into());
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
    OdbcError { error: Record },
    PostgresError { error: Box<dyn std::error::Error> },
    UrlError { error: ParseError },
}

#[derive(Debug)]
pub struct DbError {
    pub kind: DbErrorLifetime,
    pub error: DbErrorType,
}
