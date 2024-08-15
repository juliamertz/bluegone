use crate::{
    backends::{Backend, Temperature},
    config::{self, Configuration, Mode},
    state::{self},
    utils::RemoveSeconds,
};
use anyhow::Result;
use bluegone::Pid;
use chrono::Timelike;

pub fn start_daemon(config: Configuration, backend: &Backend) -> Result<()> {
    if config.mode == config::Mode::Static {
        anyhow::bail!("Static mode is not supported in the daemon.");
    }

    state::write::<Pid>(std::process::id().into())?;

    let schedule = parse_schedule(&config);
    dbg!(schedule);
    start_event_loop(&config, backend)?;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct ScheduleBlock {
    start: chrono::NaiveTime,
    end: chrono::NaiveTime,
    temperature: Temperature,
}

fn parse_schedule(config: &Configuration) -> Vec<ScheduleBlock> {
    let mut mapped: Vec<(chrono::NaiveTime, Temperature)> = config
        .schedule
        .iter()
        .filter_map(|s| {
            match s.get_time(&config.location) {
                Ok(time) => Some((time, s.get_temperature(&config.presets))),
                Err(_) => None, // FIX: Don't just ignore invalid schedule entries
            }
        })
        .collect();
    mapped.sort_by(|a, b| a.0.cmp(&b.0));

    let mut result: Vec<ScheduleBlock> = Vec::with_capacity(config.schedule.len());
    for i in 0..mapped.len() {
        let end_index = if i + 1 >= mapped.len() { 0 } else { i + 1 };
        let (start, temperature) = mapped[i];
        let (end, _) = mapped[end_index];
        result.push(ScheduleBlock {
            start,
            end,
            temperature,
        })
    }

    result
}

pub fn get_current_schedule_block(schedule: Vec<ScheduleBlock>) -> Option<ScheduleBlock> {
    let now = chrono::Local::now();
    let midnight = chrono::NaiveTime::parse_from_str("00:00:00", "%H:%M:%S").unwrap();

    schedule.into_iter().find(|schedule| {
        let now = now.time();
        if schedule.end < schedule.start {
            if now >= schedule.start {
                return true;
            }

            if now > midnight && now < schedule.end {
                return true;
            }
        }

        now >= schedule.start && now <= schedule.end
    })
}

fn start_event_loop(config: &Configuration, backend: &Backend) -> Result<()> {
    // wait till the next full minute so we get a nice round number
    let now = chrono::Local::now();
    let next_minute = now.with_minute(now.minute() + 1).unwrap().remove_seconds();

    // TODO: Handle events that are scheduled before the next minute
    let until_next_minute = next_minute.signed_duration_since(now).to_std()?;
    println!("Sleeping until next minute: {:?}", until_next_minute);
    std::thread::sleep(until_next_minute);

    loop {
        let now = chrono::Local::now().remove_seconds().time();
        let mode: Mode = match state::read() {
            Ok(mode) => mode,
            Err(_) => config.mode.clone(),
        };

        println!("Checking event at {:?}", now);

        if mode == Mode::Dynamic {
            let schedule = parse_schedule(config); // TODO: optimize
            if let Some(block) = get_current_schedule_block(schedule) {
                backend.set_temperature(block.temperature)?;
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
