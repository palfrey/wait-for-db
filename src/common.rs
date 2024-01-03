use clap::Arg;
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
    let matches = clap::Command::new("wait-for-db")
        .version(clap::crate_version!())
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .required(true)
                .value_parser(["odbc", "postgres"])
                .help("Database mode"),
        )
        .arg(
            Arg::new("connection-string")
                .short('c')
                .long("connection-string")
                .required(true)
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("Connection string"),
        )
        .arg(
            Arg::new("sql-query")
                .short('s')
                .long("sql-query")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("SQL query that should return at least one row (default: no querying)"),
        )
        .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .value_parser(clap::value_parser!(u64))
                .help("Timeout in seconds (default: wait forever)"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(clap::ArgAction::SetTrue)
                .help("Quiet mode"),

        )
        .arg(
            Arg::new("pause")
                .short('p')
                .long("pause")
                .value_parser(clap::value_parser!(u64))
                .help("Pause between checks (seconds)")
                .default_value("3"),
        )
        .get_matches();
    Opts {
        mode: DbMode::from_str(matches.get_one::<String>("mode").unwrap()),
        connection_string: matches.get_one::<String>("connection-string").unwrap().to_string(),
        sql_query: matches.get_one::<String>("sql-query").cloned(),
        timeout_seconds: matches
            .get_one::<u64>("timeout").copied(),
        quiet: matches.contains_id("quiet"),
        pause_seconds: matches
        .get_one::<u64>("pause").copied().unwrap()
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
