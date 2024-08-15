use crate::{
    backends::{Backend, Temperature},
    config::{self, Configuration, Mode},
    state,
    utils::RemoveSeconds,
};
use anyhow::Result;
use bluegone::Pid;
use chrono::Timelike;
use daemonize_me::Daemon;
use sysinfo::System;

pub fn start_daemon(
    background: &bool,
    config: Configuration,
    backend: &Backend,
    sys: &mut System,
) -> Result<()> {
    if config.mode == config::Mode::Static {
        anyhow::bail!("Static mode is not supported in the daemon.");
    }

    if let Some(pid) = state::read::<Pid>() {
        match find_process_by_id(pid, sys) {
            None => state::delete::<Pid>(),
            Some(_) => anyhow::bail!("Daemon already running."),
        }?;
    }

    match *background {
        true => {
            Daemon::new()
                .pid_file(state::file_path::<Pid>(), Some(false))
                // .stdout(stdout).stderr(stderr) TODO: LOG FILE
                .start()?;
        }
        false => {
            let pid: Pid = std::process::id().into();
            state::write(pid)?;
        }
    }

    start_event_loop(&config, backend)?;

    Ok(())
}

pub fn stop_daemon(sys: &mut System) -> Result<()> {
    match state::read::<Pid>() {
        Some(pid) => match find_process_by_id(pid, sys) {
            Some(process) => process.kill(),
            None => anyhow::bail!("No active daemon found."),
        },
        None => anyhow::bail!("No active daemon found."),
    };

    Ok(())
}

fn find_process_by_id(pid: Pid, sys: &mut System) -> Option<&sysinfo::Process> {
    let pid_state = &sysinfo::Pid::from_u32(pid.as_u32());
    sys.processes().get(pid_state)
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
        .filter_map(|s| match s.get_time(&config.location) {
            Ok(time) => Some((time, s.get_temperature(&config.presets))),
            Err(_) => {
                eprintln!("Using time based schedule entry but no location was provided!");
                None
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

pub fn get_current_schedule(schedule: Vec<ScheduleBlock>) -> Option<ScheduleBlock> {
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
            Some(mode) => mode,
            None => config.mode.clone(),
        };

        println!("Checking event at {:?}", now);

        match mode {
            Mode::Dynamic => {
                let schedule = parse_schedule(config); // TODO: optimize
                if let Some(block) = get_current_schedule(schedule) {
                    backend.set_temperature(block.temperature)?;
                    println!("Setting temperature to {}", block.temperature)
                }
            }
            Mode::Static => {
                println!("Mode is set to static, sleeping until next minute")
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
