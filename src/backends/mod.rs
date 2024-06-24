mod tty;
mod x11;

pub use tty::*;
pub use x11::*;

use crate::utils::temp_to_gamma;
use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::randr::*;
use x11rb::rust_connection::RustConnection;

pub trait GammaSetter {
    fn set_gamma(&self, gamma_r: f64, gamma_g: f64, gamma_b: f64) -> Result<()>;
    fn set_temperature(&self, temp: Temperature) -> Result<()>;
}

pub type Temperature = f64;
pub type GammaValue = Vec<u16>;

pub fn get_backend_from_str(name: &str) -> Result<Box<dyn GammaSetter>> {
    Ok(match name.to_lowercase().as_str() {
        "tty" => Box::new(TTY),
        "x11" => Box::new(X11),
        _ => anyhow::bail!("Unknown backend: {}", name),
    })
}

#[derive(Debug)]
pub struct Gamma {
    red: GammaValue,
    green: GammaValue,
    blue: GammaValue,
}
