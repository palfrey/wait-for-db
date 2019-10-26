use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn postgres_connect() -> String {
    format!(
        "Driver={};Server={};Port={};Uid={};Pwd={};",
        std::env::var("POSTGRES_DRIVER").unwrap(),
        std::env::var("POSTGRES_SERVER").unwrap(),
        std::env::var("POSTGRES_PORT").unwrap(),
        std::env::var("POSTGRES_USERNAME").unwrap(),
        std::env::var("POSTGRES_PASSWORD").unwrap(),
    )
}

#[test]
#[cfg_attr(postgres_driver = "", ignore)]
fn command_line_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("--timeout=1")
        .arg(format!("--connection-string={}", postgres_connect()))
        .arg("--sql-text=select 1 from foo");
    cmd.assert().failure().stdout(
        predicate::str::contains("Temporary error (exiting as out of time)").and(
            predicate::str::contains("ERROR: relation \"foo\" does not exist"),
        ),
    );

    Ok(())
}
