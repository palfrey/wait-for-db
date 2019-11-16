use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

extern crate wait_for_db;

#[test]
#[cfg_attr(postgres_driver = "", ignore)]
fn command_line_timeout_with_odbc() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("--timeout=1")
        .arg("--mode=odbc")
        .arg(format!(
            "--connection-string={}",
            wait_for_db::odbc::postgres_connect()
        ))
        .arg("--sql-text=select 1 from foo");
    cmd.assert().failure().stdout(
        predicate::str::contains("Temporary error (exiting as out of time)").and(
            predicate::str::contains("ERROR: relation \"foo\" does not exist"),
        ),
    );

    Ok(())
}

#[test]
#[cfg_attr(postgres_driver = "", ignore)]
fn command_line_timeout_with_postgres() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("--timeout=1")
        .arg("--mode=postgres")
        .arg(format!(
            "--connection-string={}",
            wait_for_db::pg::postgres_connect()
        ))
        .arg("--sql-text=select 1 from foo");
    cmd.assert().failure().stdout(
        predicate::str::contains("Temporary error (exiting as out of time)").and(
            predicate::str::contains("message: \"relation \\\"foo\\\" does not exist\""),
        ),
    );

    Ok(())
}
