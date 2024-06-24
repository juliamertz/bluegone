use super::*;

pub struct X11;

impl GammaSetter for X11 {
    fn set_temperature(&self, temp: Temperature) -> Result<()> {
        if (temp < 0.0) || (temp > 6500.0) {
            anyhow::bail!("Temperature must be between 0 and 6500 Kelvin");
        }

        let (gamma_r, gamma_g, gamma_b) = temp_to_gamma(temp);
        Self::set_gamma(self, gamma_r, gamma_g, gamma_b)
    }

    fn set_gamma(&self, gamma_r: f64, gamma_g: f64, gamma_b: f64) -> Result<()> {
        let (conn, _) = RustConnection::connect(None)?;

        let screen = &conn.setup().roots[0];
        let res = conn
            .randr_get_screen_resources_current(screen.root)?
            .reply()?;

        for &crtc in &res.crtcs {
            let size = conn.randr_get_crtc_gamma_size(crtc)?.reply()?.size as usize;

            let start = 0 as u16;
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
}
