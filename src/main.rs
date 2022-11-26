use crate::timelogger::{TimeLogger, log, fix_incomplete, summary};
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

    fix_incomplete(&mut tlr.log);
    log(&mut tlr.log);
    summary(&tlr.log);

    fs::write(log_file, serde_yaml::to_string(&tlr).unwrap()).unwrap();
    Ok(())
}
