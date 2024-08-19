use crate::{
    state::{self},
    utils::temp_to_gamma,
};
use anyhow::Result;
use bluegone::StateFileName;
use serde::Deserialize;
use x11rb::connection::Connection;
use x11rb::protocol::randr::*;
use x11rb::rust_connection::RustConnection;

pub type GammaValue = Vec<u16>;
// pub type Temperature = f64;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Temperature(f64);

impl Temperature {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn as_f64(&self) -> f64 {
        self.0
    }
}

impl std::fmt::Display for Temperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StateFileName for Temperature {
    fn name() -> String {
        "temperature".into()
    }
}

#[derive(Debug, Default, clap::ValueEnum, Clone)]
pub enum Backend {
    Tty,
    #[default]
    X11,
}

impl<'a> Deserialize<'a> for Backend {
    fn deserialize<D>(deserializer: D) -> Result<Backend, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let s = String::deserialize(deserializer)?;
        Backend::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<&str> for Backend {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "TTY" => Ok(Backend::Tty),
            "X11" => Ok(Backend::X11),
            _ => anyhow::bail!("Invalid backend"),
        }
    }
}

impl Backend {
    pub fn set_gamma(&self, gamma_r: f64, gamma_g: f64, gamma_b: f64) -> Result<()> {
        match self {
            Backend::Tty => set_gamma_for_tty(gamma_r, gamma_g, gamma_b),
            Backend::X11 => set_gamma_for_x11(gamma_r, gamma_g, gamma_b),
        }
    }

    pub fn set_temperature(&self, temp: Temperature) -> Result<()> {
        state::write(temp)?;
        let gamma = temp_to_gamma(temp.as_f64());
        self.set_gamma(gamma.0, gamma.1, gamma.2)
    }
}

pub fn set_gamma_for_x11(gamma_r: f64, gamma_g: f64, gamma_b: f64) -> Result<()> {
    let (conn, _) = RustConnection::connect(None)?;

    let screen = &conn.setup().roots[0];
    let res = conn
        .randr_get_screen_resources_current(screen.root)?
        .reply()?;

    for &crtc in &res.crtcs {
        let size = conn.randr_get_crtc_gamma_size(crtc)?.reply()?.size as usize;

        let start = 0_u16;
        let mut gamma = Gamma {
            red: vec![start; size],
            green: vec![start; size],
            blue: vec![start; size],
        };

        for i in 0..size {
            let g = 65535.0 * (i as f64) / (size as f64);
            gamma.red[i] = (g * gamma_r) as u16;
            gamma.green[i] = (g * gamma_g) as u16;
            gamma.blue[i] = (g * gamma_b) as u16;
        }

        conn.randr_set_crtc_gamma(crtc, &gamma.red, &gamma.green, &gamma.blue)?;
    }

    conn.flush()?;
    Ok(())
}

// TTY

static TTY_COLOR_TABLE: &[&str] = &[
    "000000", "aa0000", "00aa00", "aa5500", "0000aa", "aa00aa", "00aaaa", "aaaaaa", "555555",
    "ff5555", "55ff55", "ffff55", "5555ff", "ff55ff", "55ffff", "ffffff",
];

pub fn set_gamma_for_tty(gamma_r: f64, gamma_g: f64, gamma_b: f64) -> Result<()> {
    #[allow(clippy::needless_range_loop)]
    for i in 0..TTY_COLOR_TABLE.len() {
        let color = TTY_COLOR_TABLE[i];
        let flt_to_hex = |flt: f64| -> String {
            let flt = if flt > 255.0 { 255.0 } else { flt };
            format!("{:02x}", flt as u8)
        };

        let hex_r = flt_to_hex(gamma_r * u8::from_str_radix(&color[0..2], 16)? as f64);
        let hex_g = flt_to_hex(gamma_g * u8::from_str_radix(&color[2..4], 16)? as f64);
        let hex_b = flt_to_hex(gamma_b * u8::from_str_radix(&color[4..6], 16)? as f64);

        let string = format!("{:X}{}{}{}", i, hex_r, hex_g, hex_b);

        print!("\x1B]P{}", string);
    }

    Ok(())
}

#[derive(Debug)]
pub struct Gamma {
    red: GammaValue,
    green: GammaValue,
    blue: GammaValue,
}
