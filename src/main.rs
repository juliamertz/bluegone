mod backends;
mod config;
mod utils;

use anyhow::Result;
use backends::Backend;
use clap::Parser;
use config::Configuration;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Temperature in Kelvin, between 0 and 6500
    #[arg(short, long)]
    temperature: Option<f64>,
    /// Backend to use (X11 or TTY)
    #[arg(short, long)]
    backend: Option<String>,
    /// Preset to apply
    #[arg(short, long)]
    preset: Option<String>,
}

fn main() -> Result<()> {
    let config = Configuration::parse_from_file("config.toml")?;
    // dbg!(&config);

    // if let Some(location) = config.location {
    //     let (latitude, longitude) = (location.latitude, location.longitude);
    //     let (sunrise, sunset) = utils::sunrise_and_set(latitude, longitude)?;
    //     println!("Sunrise: {}, Sunset: {}", sunrise, sunset);
    // }

    let args = Args::parse();

    let backend = match args.backend {
        Some(val) => &Backend::try_from(val.as_str())?,
        None => &config.backend,
    };

    if let Some(preset) = args.preset {
        let preset = config.presets.iter().find(|p| p.name == preset);
        if let Some(preset) = preset {
            backend.set_temperature(preset.temperature)?;
        }
    }

    if let Some(temp) = args.temperature {
        backend.set_temperature(temp)?;
    }

    Ok(())
}
