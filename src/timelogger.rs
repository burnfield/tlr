use chrono::prelude::*;
use chrono::{NaiveDate, NaiveTime};
use comfy_table::Table;
use console::style;
use console::Term;
use dialoguer::Input;
use humantime::format_duration;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
pub struct TimeLogger {
    workday_minutes: Option<i64>,
    log: BTreeMap<NaiveDate, Vec<NaiveTime>>,
}

pub fn log(log: &mut TimeLogger) {
    let log = &mut log.log;
    let now: NaiveDateTime = Local::now().naive_local();
    let date: NaiveDate = now.date();

    let prompt = &date.format("Today %a %Y-%m-%d");
    let proposal = &now.time().format("%H:%M");
    search_and_fix_odd_time_stamps(log);

    log.entry(date)
        .and_modify(|time_stamps| {
            let proposal = &format!("{} {}", chain_time_stamps(time_stamps), proposal);
            edit_time_stamps(time_stamps, prompt, proposal);
        })
        .or_insert_with(|| {
            let time_stamps = &mut Vec::new();
            edit_time_stamps(time_stamps, prompt, proposal);
            time_stamps.to_vec()
        });
}

pub fn summary(tlr: &TimeLogger) {
    let log: &BTreeMap<NaiveDate, Vec<NaiveTime>> = &tlr.log;
    let work_day_minutes: Option<i64> = tlr.workday_minutes;
    let today: NaiveDate = Local::now().naive_local().date();

    let mut table = Table::new();

    // table header
    let mut header = vec!["Date", "Duration", "Time stamps"];
    if work_day_minutes.is_some() {
        header.push("Over time");
        header.push("Aggregated over time");
    }
    table.set_header(header);

    // table content
    let mut sum_ot: chrono::Duration = chrono::Duration::zero();
    let all_complete_days = log
        .iter()
        .filter(|(date, _time_stamps)| *date != &today)
        .filter_map(|(date, time_stamps)| {
            generate_tlr_table_rows(date, time_stamps, &mut sum_ot, work_day_minutes)
        })
        .collect::<Vec<Vec<String>>>();

    // takes the last complete 5 days
    let num_last_complete_days = 5;
    table.add_rows(
        all_complete_days
            .iter()
            .rev()
            .take(num_last_complete_days)
            .rev(),
    );

    println!("{table}");
}

fn search_and_fix_odd_time_stamps(log: &mut BTreeMap<NaiveDate, Vec<NaiveTime>>) {
    let today: NaiveDate = Local::now().naive_local().date();
    // Uneven time stamps correction
    log.iter_mut()
        .filter(|(date, _time_stamps)| *date != &today)
        .filter(|(_d, time_stamps)| (time_stamps.len() % 2) != 0)
        .for_each(fix_odd_time_stamps);
}

fn fix_odd_time_stamps(args: (&NaiveDate, &mut Vec<NaiveTime>)) {
    let (date, time_stamps) = args;
    let term = Term::stdout();
    term.write_line(
        &style("Impossible to count odd number of time stamps, please fix!")
            .bold()
            .red()
            .to_string(),
    )
    .unwrap();

    let prompt = date.format("Fixing %a %Y-%m-%d").to_string();
    let proposal = format!("{} {}", chain_time_stamps(time_stamps), "??:??");
    edit_time_stamps(time_stamps, &prompt, &proposal)
}

fn non_linear(time_stamps: &[NaiveTime]) -> bool {
    time_stamps.iter().tuple_windows().any(|(x, y)| x > y)
}

fn edit_time_stamps<T, S>(time_stamps: &mut Vec<NaiveTime>, prompt: T, proposal: &S)
where
    T: std::fmt::Display,
    S: std::fmt::Display,
{
    let prompt: String = style(prompt).bold().to_string();
    let proposal = proposal.to_string();

    // Loops until a parseable result has been found
    loop {
        if let Ok(res) = Input::<String>::new()
            .with_prompt(&prompt)
            .with_initial_text(&proposal)
            .interact_text()
            .unwrap()
            .trim()
            .split(' ')
            .map(|x| NaiveTime::parse_from_str(x, "%H:%M"))
            .collect::<Result<Vec<NaiveTime>, _>>()
        {
            if non_linear(&res) {
                continue;
            }
            *time_stamps = res.to_vec();
            break;
        }
    }
}

fn generate_tlr_table_rows(
    date: &NaiveDate,
    time_stamps: &[NaiveTime],
    sum_ot: &mut chrono::Duration,
    work_day_minutes: Option<i64>,
) -> Option<Vec<String>> {
    // default part
    let total_time = sum_timestamps(time_stamps).ok()?;
    let time_stamps: String = chain_time_stamps(time_stamps);
    let date = date.format("%a %Y-%m-%d").to_string();
    let mut row = vec![date, format_chrono_duration(total_time), time_stamps];

    // optional part
    if let Some(work_day_minutes) = work_day_minutes {
        let day_ot = chrono::Duration::minutes(-work_day_minutes) + total_time;
        row.push(format_chrono_duration(day_ot));
        *sum_ot = day_ot + *sum_ot;
        row.push(format_chrono_duration(*sum_ot));
    };
    Some(row)
}

fn chain_time_stamps(time_stamps: &[NaiveTime]) -> String {
    time_stamps
        .iter()
        .map(|x| x.format("%H:%M").to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

fn format_chrono_duration(duration: chrono::Duration) -> String {
    // humantime doesnt take chrono::Duration
    match duration < chrono::Duration::zero() {
        true => format!("-{}", format_duration((-duration).to_std().unwrap())),
        false => format_duration(duration.to_std().unwrap()).to_string(),
    }
}

fn sum_timestamps(
    timestamps: &[NaiveTime],
) -> Result<chrono::Duration, Box<dyn std::error::Error>> {
    Ok(timestamps
        .chunks(2)
        .map(|timeinterval| {
            let [start, end] = <[NaiveTime; 2]>::try_from(timeinterval)?;
            Ok(end - start)
        })
        .collect::<Result<Vec<chrono::Duration>, Box<dyn std::error::Error>>>()?
        .iter()
        .fold(chrono::Duration::zero(), |acc, b| acc + *b))
}
