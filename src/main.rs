use env_logger::Builder;
use std::env;
use std::thread;
use std::time::{Duration, Instant};

use wait_for_db::common;
use wait_for_db::odbc;
use wait_for_db::pg;

fn main() {
    let opt = common::parse_args();
    Builder::new()
        .parse_filters(&env::var("WAIT_DB_LOG").unwrap_or_else(|_| "odbc=off".to_string()))
        .init();

    if opt.pause_seconds == 0 {
        println!("Pause between checks should be at least 1 second");
        std::process::exit(exitcode::USAGE);
    }

    let timeout = opt.timeout_seconds.map(Duration::from_secs);
    let start = Instant::now();
    while timeout.is_none() || start.elapsed() < timeout.unwrap() {
        match if opt.mode == common::DbMode::Odbc {
            odbc::connect(&opt)
        } else {
            pg::connect(&opt)
        } {
            Ok(results) => {
                if opt.sql_query.is_none() {
                    println!("Success");
                } else {
                    println!("Success {:?}", results);
                }
                std::process::exit(exitcode::OK);
            }
            Err(dberror) => match dberror.kind {
                common::DbErrorLifetime::Permanent => {
                    println!("Permanent error: {:?}", dberror.error);
                    std::process::exit(exitcode::UNAVAILABLE);
                }
                common::DbErrorLifetime::Temporary => {
                    let pause_time = Duration::from_secs(opt.pause_seconds);
                    if let Some(t) = timeout {
                        let remaining = t - start.elapsed();
                        if remaining < pause_time {
                            println!(
                                "Temporary error (exiting as out of time): {:?}",
                                dberror.error
                            );
                            std::process::exit(exitcode::UNAVAILABLE);
                        }
                    }
                    println!(
                        "Temporary error (pausing for {} second{}): {:?}",
                        opt.pause_seconds,
                        if opt.pause_seconds == 1 { "" } else { "s" },
                        dberror.error
                    );
                    thread::sleep(pause_time);
                }
            },
        }
    }
    std::process::exit(exitcode::UNAVAILABLE);
}
