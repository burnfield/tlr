use chrono::prelude::*;
use chrono::{Duration, NaiveDate, NaiveTime};
use console::{style, Term};
use dialoguer::Input;
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

pub fn summary(log: &mut BTreeMap<NaiveDate, Vec<NaiveTime>>, term: &Term) -> std::io::Result<()> {
    let today: NaiveDate = Local::now().naive_local().date();
    let mut sum_ot: Duration = Duration::zero();
    log.iter().for_each(|(date, timestamps)| {
        // TODO(Oskar): figure out how to throw a propper error from map
        let is_today = &today != date;
        let mut day_ot = Duration::zero();
        if is_today {
            day_ot = sum_timestamps(timestamps);
        }
        sum_ot = sum_ot + day_ot;
        let mut sum_ot_str = style(format!("OT({})", format_duration(sum_ot)));
        if sum_ot >= Duration::hours(8) || sum_ot <= Duration::hours(-8) {
            sum_ot_str = sum_ot_str.red().bold();
        } else if sum_ot >= Duration::hours(4) || sum_ot <= Duration::hours(-4) {
            sum_ot_str = sum_ot_str.yellow();
        } else {
            sum_ot_str = sum_ot_str.green();
        }
        //Day date timestamps overtime sum_ot
        let timestamps: String = chain_time_stamps(timestamps);

        if (today - *date) <= Duration::days(7) && is_today {
            term.write_line(
                format!(
                    "{} {} {}",
                    style(format!("date({} {})", date.weekday(), date)).magenta(),
                    sum_ot_str,
                    style(format!("time stamps({})", timestamps)).cyan(),
                )
                .as_str(),
            )
            .unwrap();
        }
    });
    Ok(())
}

fn chain_time_stamps(time_stamps: &[NaiveTime]) -> String {
    time_stamps
        .iter()
        .map(|x| x.format("%H:%M").to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

fn format_duration(duration: Duration) -> String {
    let mut tmp = duration;
    let days = tmp.num_days();

    tmp = tmp - Duration::days(days);
    let hours = tmp.num_hours();

    tmp = tmp - Duration::hours(hours);
    let minutes = tmp.num_minutes();

    tmp = tmp - Duration::minutes(minutes);
    let seconds = tmp.num_seconds();
    if minutes < 0 {
        format!("-{}:{}:{}", -hours, -minutes, -seconds)
    } else {
        format!("{}:{}:{}", hours, minutes, seconds)
    }
}

fn sum_timestamps(timestamps: &[NaiveTime]) -> Duration {
    // TODO(Oskar): ensure order within vector
    timestamps
        .to_vec()
        .chunks(2)
        .map(|timeinterval| {
            let [start, end] =
                <[NaiveTime; 2]>::try_from(timeinterval).expect("Odd log points on day");
            end - start
        })
        .fold(Duration::minutes(-468), |acc, b| acc + b)
}
