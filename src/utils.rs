use std::f64;

use anyhow::Result;
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
    let r: f64;
    if temp <= 66.0 {
        r = 255.0;
    } else {
        let t = temp - 60.0;
        r = 329.698727446 * t.powf(-0.1332047592);
    }

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

use sunrise_sunset_calculator::SunriseSunsetParameters;

pub fn sunrise_and_set(latitude: f64, longitude: f64) -> Result<(i64, i64)> {
    let now = chrono::Local::now();
    let params = SunriseSunsetParameters::new(now.timestamp(), latitude, longitude);

    match params.calculate() {
        Ok(result) => Ok((result.set, result.rise)),
        Err(e) => anyhow::bail!("Error calculating sunrise and sunset: {}", e),
    }
}
