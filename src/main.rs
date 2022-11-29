use crate::timelogger::{log, summary, TimeLogger};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Storage jfile for time stamps, can be set with environment variable TLR_LOG.
    /// Note: specified path overrides the environment variable.
    #[arg(required = true, env = "TLR_LOG")]
    log_file: PathBuf,
    /// Print a sumray of NUM days, starting with the latest
    #[arg(short, long)]
    print: Option<usize>,
}

pub mod timelogger;

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let fp: PathBuf = args.log_file;
    let f = fs::read(&fp)?;
    let mut tlr: TimeLogger = match f.is_empty() {
        true => TimeLogger::default(),
        false => serde_yaml::from_slice(&f).expect("Corrupt log file"),
    };

    if let Some(num_last_complete_days) = args.print {
        summary(&tlr, num_last_complete_days);
        return Ok(());
    };

    log(&mut tlr);
    summary(&tlr, 3);

    fs::write(fp, serde_yaml::to_string(&tlr).unwrap()).unwrap();
    Ok(())
}
