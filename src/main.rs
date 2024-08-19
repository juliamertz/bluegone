mod backends;
mod cli;
mod config;
mod daemon;
mod state;
mod utils;

use std::path::PathBuf;
use anyhow::Result;
use backends::Backend;
use clap::{builder::EnumValueParser, command, value_parser, Arg};
use config::Configuration;

fn main() -> Result<()> {
    let args = command!("bluegone")
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("config")
                .short('c')
                .required(false)
                .long("config")
                .help("Path to configuration file")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("backend")
                .short('b')
                .required(false)
                .long("backend")
                .help("Backend to use")
                .value_parser(EnumValueParser::<Backend>::new()),
        )
        .subcommand(cli::init_info_subcommand())
        .subcommand(cli::init_daemon_subcommand())
        .subcommand(cli::init_list_subcommand())
        .subcommand(cli::init_set_subcommand())
        .get_matches();

    let mut sys = sysinfo::System::new_all();

    let config = Configuration::get_config(&args)?;
    let backend = match args.get_one::<Backend>("backend") {
        Some(backend) => backend,
        None => &config.backend,
    };

    match args.subcommand() {
        Some(("set", args)) => cli::handle_set_subcommand(args, backend, config),
        Some(("info", args)) => cli::handle_info_subcommand(args, backend, config, &mut sys),
        Some(("daemon", args)) => cli::handle_daemon_subcommand(args, backend, config, &mut sys),
        Some(("list", args)) => cli::handle_list_subcommand(args, config),
        None | Some((_, _)) => anyhow::bail!("No subcommand provided."),
    }
}
