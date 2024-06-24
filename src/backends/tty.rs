use super::*;

static TTY_COLOR_TABLE: &'static [&str] = &[
    "000000", "aa0000", "00aa00", "aa5500", "0000aa", "aa00aa", "00aaaa", "aaaaaa", "555555",
    "ff5555", "55ff55", "ffff55", "5555ff", "ff55ff", "55ffff", "ffffff",
];

pub struct TTY;

impl GammaSetter for TTY {
    fn set_temperature(&self, temp: Temperature) -> Result<()> {
        assert!(temp >= 0.0 && temp <= 6500.0);
        let (gamma_r, gamma_g, gamma_b) = temp_to_gamma(temp);
        Self::set_gamma(self, gamma_r, gamma_g, gamma_b)
    }

    fn set_gamma(&self, gamma_r: f64, gamma_g: f64, gamma_b: f64) -> Result<()> {
        for i in 0..16 {
            let color = TTY_COLOR_TABLE[i];
            let flt_to_hex = |flt: f64| -> String {
                let flt = if flt > 255.0 { 255.0 } else { flt };
                format!("{:02x}", flt as u8)
            };

            let hex_r = flt_to_hex(gamma_r * u8::from_str_radix(&color[0..2], 16)? as f64);
            let hex_g = flt_to_hex(gamma_g * u8::from_str_radix(&color[2..4], 16)? as f64);
            let hex_b = flt_to_hex(gamma_b * u8::from_str_radix(&color[4..6], 16)? as f64);

            let string = format!("{:X}{}{}{}", i, hex_r, hex_g, hex_b);

            println!("\x1B]P{}", string);
        }

        Ok(())
    }
}
