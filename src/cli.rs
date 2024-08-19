use anyhow::Result;
use bluegone::Pid;
use clap::{builder::EnumValueParser, value_parser, Arg, ArgAction, ArgGroup, ArgMatches, Command};

use crate::{
    backends::{Backend, Temperature},
    config::{Configuration, Mode},
    daemon::{self, find_process_by_id, get_current_schedule, parse_schedule},
    state,
};

pub fn init_info_subcommand() -> Command {
    Command::new("info").about("List various configured options")
}

pub fn handle_info_subcommand(
    args: &ArgMatches,
    backend: &Backend,
    config: &Configuration,
    sys: &mut sysinfo::System,
) -> Result<()> {
    let process = match state::read::<Pid>() {
        Some(pid) => find_process_by_id(pid, sys),
        None => None,
    };

    let schedule = parse_schedule(config);
    let schedule = get_current_schedule(schedule);

    match process {
        Some(process) => println!("Daemon active (pid: {})", process.pid()),
        None => println!("Daemon inactive"),
    }

    Ok(())
}

pub fn init_set_subcommand() -> Command {
    Command::new("set")
        .about("Set temperature or preset")
        .arg(
            Arg::new("temperature")
                .short('t')
                .long("temperature")
                .help("Temperature to set in Kelvin, (0-6500)") // TODO: Custom parser with nice help message
                .value_parser(value_parser!(f64)),
        )
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_parser(EnumValueParser::<Mode>::new())
                .help("Set current mode, if set to dynamic the daemon will manage temperature"),
        )
        .arg(
            Arg::new("preset")
                .short('p')
                .long("preset")
                .help("Preset to apply"),
        )
        .group(
            ArgGroup::new("set_target")
                .args(["temperature", "mode", "preset"])
                .required(true)
                .multiple(false),
        )
}

pub fn handle_set_subcommand(
    args: &ArgMatches,
    backend: &Backend,
    config: &Configuration,
) -> Result<()> {
    if let Some(value) = args.get_one::<f64>("temperature") {
        let temperature = Temperature::new(value.to_owned());
        backend.set_temperature(temperature)?;
        state::write(Mode::Static)?;
        return Ok(());
    }

    if let Some(value) = args.get_one::<String>("preset") {
        let preset = config.presets.iter().find(|p| p.name == value.clone());
        if let Some(preset) = preset {
            backend.set_temperature(preset.temperature)?;
            state::write(Mode::Static)?;
            return Ok(());
        }
    }

    if let Some(value) = args.get_one::<Mode>("mode") {
        state::write(value.clone())?;
        return Ok(());
    }

    anyhow::bail!("No argument found")
}

pub fn init_daemon_subcommand() -> Command {
    Command::new("daemon")
        .about("Control the daemon")
        .subcommand_required(true)
        .subcommand(
            Command::new("start").about("Start the daemon").arg(
                Arg::new("background")
                    .short('b')
                    .long("background")
                    .action(ArgAction::SetTrue)
                    .help("Start daemon as a background process"),
            ),
        )
        .subcommand(Command::new("stop").about("Stop the daemon"))
}

pub fn handle_daemon_subcommand(
    args: &ArgMatches,
    backend: &Backend,
    config: &Configuration,
    sys: &mut sysinfo::System,
) -> Result<()> {
    sys.refresh_all();

    match args.subcommand() {
        Some(("start", args)) => {
            daemon::start_daemon(args, config.clone(), backend, sys)?;
        }
        Some(("stop", _)) => {
            daemon::stop_daemon(sys)?;
        }
        None | Some((_, _)) => anyhow::bail!("No subcommand provided"),
    }

    Ok(())
}

pub fn init_list_subcommand() -> Command {
    Command::new("list")
        .subcommand_required(true)
        .about("List various configured options")
        .subcommand(Command::new("presets").about("List all presets"))
}

pub fn handle_list_subcommand(args: &ArgMatches, config: &Configuration) -> Result<()> {
    match args.subcommand() {
        Some(("presets", _)) => {
            for preset in config.presets.clone() {
                println!("{}: {}K", preset.name, preset.temperature);
            }
        }
        None | Some((_, _)) => anyhow::bail!("No subcommand provided"),
    };

    Ok(())
}
