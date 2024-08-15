use anyhow::Result;
use clap::{builder::EnumValueParser, value_parser, Arg, ArgGroup, ArgMatches, Command};

use crate::{
    backends::Backend,
    config::{Configuration, Mode},
    daemon::{self},
    state::{self},
};

pub fn init_set_subcommand() -> Command {
    Command::new("set")
        .about("Set temperature or preset")
        .arg(
            Arg::new("temperature")
                .short('t')
                .long("temperature")
                .help("Temperature to set in kelven, (0-6500)")
                .value_parser(value_parser!(f64)),
        )
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_parser(EnumValueParser::<Mode>::new())
                .help("Set mode the daemon uses, (static|dynamic)"),
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
        backend.set_temperature(value.to_owned())?;
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
                    .required(false)
                    .long("background")
                    .help("Start daemon as a background process")
                    .value_parser(EnumValueParser::<Backend>::new()),
            ),
        )
        .subcommand(Command::new("stop").about("Stop the daemon"))
}

pub fn handle_daemon_subcommand(
    args: &ArgMatches,
    backend: &Backend,
    config: &Configuration,
) -> Result<()> {
    match args.subcommand() {
        Some(("start", _)) => {
            daemon::start_daemon(config.clone(), backend)?;
        }
        Some(("stop", _)) => unimplemented!(),
        None | Some((_, _)) => anyhow::bail!("No subcommand provided"),
    }

    Ok(())
}

pub fn init_list_subcommand() -> Command {
    Command::new("list")
        .subcommand_required(true)
        .about("List x")
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
