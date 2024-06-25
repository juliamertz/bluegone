mod backends;
mod utils;

use anyhow::Result;
use backends::Backend;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Temperature in Kelvin, between 0 and 6500
    #[arg(short, long)]
    temperature: Option<f64>,
    /// Backend to use
    #[arg(short, long)]
    backend: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let backend = match args.backend {
        Some(val) => Backend::try_from(val.as_str())?,
        None => Backend::X11,
    };

    if let Some(temp) = args.temperature {
        backend.set_temperature(temp)?;
    }

    Ok(())
}
