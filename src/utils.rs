use crate::utils::{self};
use std::{f64, fs::File, path::PathBuf, str::FromStr};
use anyhow::Result;

pub fn get_data_path() -> PathBuf {
    let cache_dir = match std::env::var("XDG_CACHE_DIR") {
        Ok(path) => PathBuf::from_str(&path).expect("Valid path"),
        Err(_) => utils::home_dir().join(".cache"),
    };

    cache_dir.join("bluegone")
}

pub fn new_log_file() -> Result<File> {
    let timestamp = chrono::Local::now().to_rfc3339();
    let filename = format!("bluegone-log-{}.txt", timestamp);
    match File::create(filename) {
        Ok(file) => Ok(file),
        Err(e) => anyhow::bail!("Unable to create log file, {e}"),
    }
}

pub fn home_dir() -> PathBuf {
    #[allow(deprecated)] // deprecated because of windows support.
    match std::env::home_dir() {
        Some(path) => path,
        None => std::env::temp_dir(),
    }
}

/// Transforms temperature in Kelvin to Gamma values between 0 and 1.
/// Source: http://www.tannerhelland.com/4435/convert-temperature-rgb-algorithm-code/
pub fn temp_to_gamma(temp: f64) -> (f64, f64, f64) {
    fn rgb_to_gamma(color: f64) -> f64 {
        if color < 0.0 {
            0.0
        } else if color > 255.0 {
            1.0
        } else {
            color / 255.0
        }
    }

    let temp = temp / 100.0;

    // red
    let r: f64 = if temp <= 66.0 {
        255.0
    } else {
        let t = temp - 60.0;
        329.698727446 * t.powf(-0.1332047592)
    };

    // green
    let mut g: f64;
    if temp <= 66.0 {
        g = temp;
        g = 99.4708025861 * g.ln() - 161.1195681661;
    } else {
        let t = temp - 60.0;
        g = 288.1221695283 * t.powf(-0.0755148492);
    }

    // blue
    let b: f64;
    if temp <= 10.0 {
        b = 0.0;
    } else if temp >= 66.0 {
        b = 1.0;
    } else {
        let t = temp - 10.0;
        b = 138.5177312231 * t.ln() - 305.0447927307;
    }

    (rgb_to_gamma(r), rgb_to_gamma(g), rgb_to_gamma(b))
}

use chrono::Timelike;
pub trait RemoveSeconds<T>
where
    T: chrono::Timelike,
{
    /// Should panic if the time is invalid
    fn remove_seconds(&self) -> Self;
}

impl RemoveSeconds<chrono::NaiveTime> for chrono::NaiveTime {
    fn remove_seconds(&self) -> Self {
        self.with_second(0)
            .expect("time to be valid")
            .with_nanosecond(0)
            .expect("time to be valid")
    }
}

impl RemoveSeconds<chrono::DateTime<chrono::Local>> for chrono::DateTime<chrono::Local> {
    fn remove_seconds(&self) -> Self {
        self.with_second(0)
            .expect("time to be valid")
            .with_nanosecond(0)
            .expect("time to be valid")
    }
}
