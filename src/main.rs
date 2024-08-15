mod backends;
mod config;
mod daemon;
mod state;
mod utils;

use anyhow::Result;
use backends::Backend;
use clap::{Parser, Subcommand};
use config::{Configuration, Mode};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Backend to use (X11 or TTY)
    #[arg(short, long)]
    backend: Option<String>,
    /// Path to config file
    #[arg(short, long)]
    config: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct Group {
    /// Temperature in Kelvin, between 0 and 6500
    #[arg(short, long)]
    temperature: Option<f64>,
    /// Preset to apply
    #[arg(short, long)]
    preset: Option<String>,
    /// Which mode to use (dynamic, static)
    #[arg(short, long)]
    mode: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ListPresets,
    Set {
        #[clap(flatten)]
        group: Group,
    },
    Daemon {
        #[arg(long)]
        start: bool,
        #[arg(long)]
        stop: bool,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let config = Configuration::get_config(&args)?;

    let backend = match args.backend {
        Some(val) => Backend::try_from(val.as_str())?,
        None => config.backend.clone(),
    };

    match args.command {
        Some(Commands::Daemon { start, stop }) => {
            if start {
                daemon::start_daemon(config.clone(), &backend)?;
            } else if stop {
                // return daemon::stop_daemon();
            }
        }
        Some(Commands::Set {
            group:
                Group {
                    temperature: Some(value),
                    ..
                },
            ..
        }) => backend.set_temperature(value)?,

        Some(Commands::Set {
            group: Group {
                mode: Some(value), ..
            },
        }) => {
            state::write(Mode::try_from(value)?)?;
            println!("Ok!");
        }

        Some(Commands::Set {
            group: Group {
                preset: Some(value),
                ..
            },
            ..
        }) => {
            let preset = config.presets.iter().find(|p| p.name == value);
            if let Some(preset) = preset {
                backend.set_temperature(preset.temperature)?;
            }
        }
        Some(Commands::ListPresets) => {
            for preset in config.presets {
                println!("{}: {}K", preset.name, preset.temperature);
            }
        }
        _ => {}
    }

    Ok(())
}
