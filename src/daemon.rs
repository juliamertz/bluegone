use crate::{
    backends::{Backend, Temperature},
    config::{self, Configuration, Mode},
    state,
    utils::{self, RemoveSeconds},
};
use anyhow::Result;
use bluegone::Pid;
use chrono::{NaiveTime, Timelike};
use clap::ArgMatches;
use daemonize_me::Daemon;
use signal_hook::{consts::{SIGINT, SIGTERM}, iterator::Signals};
use std::thread;
use sysinfo::System;

pub fn start_daemon(
    args: &ArgMatches,
    config: Configuration,
    backend: &Backend,
    sys: &mut System,
) -> Result<()> {
    if config.mode == config::Mode::Static {
        anyhow::bail!("Static mode is not supported in the daemon.");
    }

    // If there is a lingering pid file we check if that process is still running
    // if not we can delete it and continue
    if let Some(pid) = state::read::<Pid>() {
        match find_process_by_id(pid, sys) {
            None => state::delete::<Pid>(),
            Some(_) => anyhow::bail!("Daemon already running."),
        }?;
    }

    env_logger::Builder::new()
        .target(env_logger::Target::Stdout)
        .init();

    match args.get_one::<bool>("background") {
        Some(true) => {
            let log_file = utils::new_log_file()?;
            Daemon::new()
                .pid_file(state::file_path::<Pid>(), Some(false))
                .stdout(log_file)
                .start()?
        }
        _ => {
            let pid: Pid = std::process::id().into();
            state::write(pid)?;
        }
    }

    spawn_signal_handler()?;
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

pub fn find_process_by_id(pid: Pid, sys: &mut System) -> Option<&sysinfo::Process> {
    let pid_state = &sysinfo::Pid::from_u32(pid.as_u32());
    sys.processes().get(pid_state)
}

#[derive(Debug, Clone, Copy)]
pub struct ScheduleBlock {
    start: NaiveTime,
    end: NaiveTime,
    temperature: Temperature,
}

impl ScheduleBlock {
    pub fn new(start: NaiveTime, end: NaiveTime, temperature: Temperature) -> Self {
        Self {
            start,
            end,
            temperature,
        }
    }
}

pub fn parse_schedule(config: &Configuration) -> Vec<ScheduleBlock> {
    let mut mapped: Vec<(NaiveTime, Temperature)> = config
        .schedule
        .iter()
        .filter_map(|s| match s.get_time(&config.location) {
            Ok(time) => {
                let temperature = s.get_temperature(&config.presets);
                Some((time, temperature))
            }
            Err(_) => {
                eprintln!("Using time based schedule entry but no location was provided!");
                None
            }
        })
        .collect();

    mapped.sort_by_key(|&(time, _)| time);

    mapped
        .iter()
        .enumerate()
        .map(|(i, &(start, temperature))| {
            let end = mapped.get(i + 1).map_or(mapped[0].0, |&(end, _)| end);
            ScheduleBlock::new(start, end, temperature)
        })
        .collect()
}

pub fn get_current_schedule(schedule: Vec<ScheduleBlock>) -> Option<ScheduleBlock> {
    let now = chrono::Local::now().time();

    schedule.into_iter().find(|block| {
        if block.end < block.start {
            // Schedule spans midnight
            now >= block.start || now < block.end
        } else {
            // Regular schedule
            now >= block.start && now <= block.end
        }
    })
}

fn spawn_signal_handler() -> Result<()> {
    let mut signals = Signals::new([SIGINT, SIGTERM])?;

    thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal {:?}", sig);
        }
    });

    Ok(())
}

fn start_event_loop(config: &Configuration, backend: &Backend) -> Result<()> {
    // wait till the next full minute so we get a nice round number
    let now = chrono::Local::now();
    let next_minute = now.with_minute(now.minute() + 1).unwrap().remove_seconds();

    // TODO: Handle events that are scheduled before the next minute
    let until_next_minute = next_minute.signed_duration_since(now).to_std()?;
    log::debug!("Sleeping until next minute: {:?}", until_next_minute);
    std::thread::sleep(until_next_minute);

    loop {
        let now = chrono::Local::now().remove_seconds().time();
        let mode: Mode = match state::read() {
            Some(mode) => mode,
            None => config.mode.clone(),
        };

        log::debug!("Checking event at {:?}", now);

        match mode {
            Mode::Dynamic => {
                let schedule = parse_schedule(config); // TODO: optimize
                if let Some(block) = get_current_schedule(schedule) {
                    log::info!("matched schedule: {:?}", block);
                    backend.set_temperature(block.temperature)?;
                    log::info!("set temperature to {}", block.temperature);
                }
            }
            Mode::Static => {
                log::debug!("Mode is set to static, sleeping until next minute");
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
