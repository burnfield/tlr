use crate::timelogger::{TimeLogger, log, summary};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}
pub mod timelogger;

fn main() -> std::io::Result<()> {
    let log_file: PathBuf = dirs::home_dir().unwrap().join(".tlr");
    let mut tlr: TimeLogger = serde_yaml::from_slice(&fs::read(&log_file).unwrap()).unwrap();
    let _args = Args::parse();

    log(&mut tlr);
    summary(&tlr);

    fs::write(log_file, serde_yaml::to_string(&tlr).unwrap()).unwrap();
    Ok(())
}
