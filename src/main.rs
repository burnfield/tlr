use crate::timelogger::TimeLogger;
use clap::Parser;
use console::Term;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}
pub mod timelogger;

fn main() -> std::io::Result<()> {
    let log: PathBuf = dirs::home_dir().unwrap().join(".tlr");
    let mut tlr: TimeLogger = serde_yaml::from_slice(&fs::read(&log).unwrap()).unwrap();
    let _args = Args::parse();

    let term = Term::stdout();
    tlr.fix_incomplete(&term);
    tlr.summary(&term)?;
    term.write_line("")?;
    tlr.log();

    fs::write(log, serde_yaml::to_string(&tlr).unwrap()).unwrap();
    Ok(())
}
