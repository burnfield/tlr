use chrono::prelude::*;
use chrono::{NaiveDate, NaiveTime};
use comfy_table::Table;
use console::style;
use dialoguer::Input;
use humantime::format_duration;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
pub struct TimeLogger {
    pub log: BTreeMap<NaiveDate, Vec<NaiveTime>>,
}

pub fn log(log: &mut BTreeMap<NaiveDate, Vec<NaiveTime>>) {
    let now: NaiveDateTime = Local::now().naive_local();
    let date: NaiveDate = now.date();

    let prompt = &date.format("Today %a %Y-%m-%d");
    let proposal = &now.time().format("%H:%M");

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

pub fn fix_incomplete(log: &mut BTreeMap<NaiveDate, Vec<NaiveTime>>) {
    let today: NaiveDate = Local::now().naive_local().date();
    log.iter_mut()
        .filter(|(date, time_stamps)| (time_stamps.len() % 2) != 0 && *date != &today)
        .for_each(|(date, time_stamps)| {
            let prompt = date.format("Fixing %a %Y-%m-%d");
            let proposal = &format!("{} {}", chain_time_stamps(time_stamps), "??:??");
            edit_time_stamps(time_stamps, prompt, proposal)
        });
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
            .split(' ')
            .map(|x| NaiveTime::parse_from_str(x, "%H:%M"))
            .collect::<Result<Vec<NaiveTime>, _>>()
        {
            *time_stamps = res.to_vec();
            break;
        }
    }
}

pub fn summary(log: &BTreeMap<NaiveDate, Vec<NaiveTime>>) {
    let mut table = Table::new();
    table.set_header(vec![
        "Date",
        "Duration",
        "Over time",
        "Aggregated over time",
        "Time stamps",
    ]);
    let today: NaiveDate = Local::now().naive_local().date();
    let mut sum_ot: chrono::Duration = chrono::Duration::zero();

    log.iter()
        .filter(|(date, _time_stamps)| *date != &today)
        .for_each(|(date, time_stamps)| {
            let total_time = sum_timestamps(time_stamps);
            let day_ot = chrono::Duration::minutes(-468) + total_time;
            sum_ot = sum_ot + day_ot;
            //Day date timestamps overtime sum_ot
            let time_stamps: String = chain_time_stamps(time_stamps);

            if (today - *date) <= chrono::Duration::days(7) {
                let date = date.format("%a %Y-%m-%d").to_string();
                table.add_row(vec![
                    date,
                    format_chrono_duration(total_time),
                    format_chrono_duration(day_ot),
                    format_chrono_duration(sum_ot),
                    time_stamps,
                ]);
            }
        });

    println!("{table}");
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
    .to_string()
}

fn sum_timestamps(timestamps: &[NaiveTime]) -> chrono::Duration {
    // TODO(Oskar): ensure order within vector
    timestamps
        .to_vec()
        .chunks(2)
        .map(|timeinterval| {
            let [start, end] =
                <[NaiveTime; 2]>::try_from(timeinterval).expect("Odd log points on day");
            end - start
        })
        .fold(chrono::Duration::zero(), |acc, b| acc + b)
}
