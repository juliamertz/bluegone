mod backends;
mod config;
mod daemon;
mod utils;

use anyhow::Result;
use backends::Backend;
use clap::{Parser, Subcommand};
use config::Configuration;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Backend to use (X11 or TTY)
    #[arg(short, long)]
    backend: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

// #[derive(Debug, clap::Args)]
// #[clap(name = "command")]
// pub struct MyCommand {
//     #[clap(flatten)]
//     group: Group,
//     #[clap(name = "others", long)]
//     other_commands: Option<String>,
// }

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct Group {
    /// Temperature in Kelvin, between 0 and 6500
    #[arg(short, long)]
    temperature: Option<f64>,
    /// Preset to apply
    #[arg(short, long)]
    preset: Option<String>,
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
    let config = Configuration::get_config()?;
    let args = Cli::parse();

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
        }) => backend.set_temperature(value)?,

        Some(Commands::Set {
            group: Group {
                preset: Some(value),
                ..
            },
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
