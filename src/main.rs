use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use clap::Parser;
use icalendar::{Calendar, Component, Event, EventLike, EventStatus};
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Write;
use std::process::exit;

#[derive(Parser, Debug)]
#[command(
    version = "2025.1",
    about = "Chaos Communication Congress Self Organized Session to iCal Converter"
)]
struct Args {
    /// The URL of the CCC event (e.g., https://events.ccc.de/congress/2025/hub/...)
    #[arg(short, long)]
    url: String,

    #[arg(short, long, default_value = "2025")]
    year: u64,

    /// Output filename
    #[arg(short, long, default_value = "event.ics")]
    output: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let response = reqwest::blocking::get(&args.url)?.text()?;
    let document = Html::parse_document(&response);

    // 1. Precise 2025 Hub Selectors
    let title_sel = Selector::parse(".hub-head-main").unwrap();
    let description_sel = Selector::parse(".hub-text").unwrap();
    let time_sel = Selector::parse(".hub-event-details__time").unwrap();
    let day_sel = Selector::parse(".hub-event-details__day").unwrap();

    let title = document
        .select(&title_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Event".to_string());

    let description = document
        .select(&description_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    // 2. Parse Day/Time
    let mut start_str = String::new();
    let mut end_str = String::new();
    let mut day = 26;
    let location = "CCH Hamburg".to_string();

    if let Some(time_meta) = document.select(&time_sel).next() {
        let time_infos = time_meta.text().collect::<String>().to_string();
        let time_infos = time_infos.split("-").map(|s| s.trim()).collect::<Vec<_>>();
        start_str = time_infos.get(0).unwrap().to_string();
        end_str = time_infos.get(1).unwrap().to_string();

        if let Some(day_info) = document.select(&day_sel).next() {
            let day_info = day_info.text().collect::<String>();
            let day_num: i8 = day_info
                .split(" ")
                .last()
                .and_then(|d| d.parse().ok())
                .unwrap();
            day += day_num;
        }
    }

    let fmt = "%Y-%m-%d %H:%M:%S";
    let dt_begin_str = format!("{}-12-{day} {start_str}:00", args.year);
    let dt_end_str = format!("{}-12-{day} {end_str}:00", args.year);
    let dt_start = NaiveDateTime::parse_from_str(&dt_begin_str, fmt).unwrap();

    let dt_end = NaiveDateTime::parse_from_str(&dt_end_str, fmt).unwrap();

    // 3. Build iCal
    let event = Event::new()
        .summary(&title)
        .description(&description)
        .starts(dt_start)
        .ends(dt_end)
        .location(&location)
        .status(EventStatus::Confirmed)
        .done();

    let mut calendar = Calendar::new();
    calendar.push(event);

    let mut file = File::create(&args.output)?;
    write!(file, "{}", calendar.to_string())?;

    println!("Event '{}' exported to {}", title, args.output);
    Ok(())
}
