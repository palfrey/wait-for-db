mod common;
mod odbc;

use structopt::StructOpt;

fn main() {
    let opt = common::Opts::from_args();
    env_logger::init();

    match odbc::connect(opt) {
        Ok(()) => println!("Success"),
        Err(diag) => println!("Error: {:?}", diag),
    }
}
