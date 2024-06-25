use crate::{
    backends::Backend,
    config::{self, Configuration, Schedule, ScheduleTrigger},
    utils::RemoveSeconds,
};
use anyhow::Result;
use chrono::Timelike;

pub fn start_daemon(config: Configuration, backend: &Backend) -> Result<()> {
    let schedule_needs_location = config
        .schedule
        .iter()
        .any(|schedule| matches!(schedule.get_trigger(), ScheduleTrigger::Light(_)));

    if config.location.is_none() && schedule_needs_location {
        anyhow::bail!("Location configuration is required for scheduling with sunrise/sunset.");
    }

    let next_event =
        &config::get_next_scheduled_event(config.schedule.clone(), config.location.clone());
    match next_event {
        Ok(event) => start_event_loop(&config, event.clone(), backend)?,
        Err(_) => {
            std::thread::sleep(std::time::Duration::from_secs(60 * 5));
            start_daemon(config, backend)?;
        }
    }

    Ok(())
}

fn start_event_loop(config: &Configuration, event: Schedule, backend: &Backend) -> Result<()> {
    // wait till the next full minute so we get a nice round number
    let now = chrono::Local::now();
    let next_minute = now.with_minute(now.minute() + 1).unwrap().remove_seconds();

    // add check to see if the event happens before this time runs out, if so
    // we can just spawn a new thread and let it sleep until the event happens
    let until_next_minute = next_minute.signed_duration_since(now).to_std()?;
    println!("Sleeping until next minute: {:?}", until_next_minute);
    std::thread::sleep(until_next_minute);

    loop {
        let now = chrono::Local::now().remove_seconds().time();

        println!("Checking event at {:?}", now);
        dbg!(now, event.get_trigger().get_time(&config.location)?);

        let event_time = event.get_trigger().get_time(&config.location)?;
        if now == event_time {
            println!("Event time reached: {:?}", event_time);
            match event {
                Schedule::Preset { preset, .. } => {
                    let preset = config
                        .presets
                        .iter()
                        .find(|p| p.name == preset)
                        .expect("Preset not found");
                    backend.set_temperature(preset.temperature)?;
                }
                Schedule::Temperature { temperature, .. } => {
                    backend.set_temperature(temperature)?;
                }
            }
            break;
        }

        std::thread::sleep(std::time::Duration::from_secs(60));
    }

    if let Ok(next_event) =
        config::get_next_scheduled_event(config.schedule.clone(), config.location.clone())
    {
        start_event_loop(config, next_event, backend)?;
    }

    unreachable!()
}
