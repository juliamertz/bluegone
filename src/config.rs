use crate::backends::{Backend, Temperature};
use anyhow::Result;
use chrono::prelude as crono;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Configuration {
    #[serde(default)]
    pub backend: Backend,
    #[serde(default)]
    pub mode: Mode,
    pub location: Option<Location>,
    pub presets: Vec<Preset>,
    #[serde(default)]
    pub schedule: Vec<Schedule>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Static,
    Dynamic,
}

#[derive(Deserialize, Debug)]
pub struct Preset {
    pub name: String,
    pub temperature: Temperature,
}

#[derive(Debug)]
pub enum Schedule {
    Temperature {
        trigger: ScheduleTrigger,
        temperature: Temperature,
    },
    Preset {
        trigger: ScheduleTrigger,
        preset: String,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub enum ScheduleLightTrigger {
    Sunset,
    Sunrise,
}

#[derive(Debug, Clone)]
pub enum ScheduleTrigger {
    Time(crono::NaiveTime),
    Light(ScheduleLightTrigger),
}

#[derive(Deserialize, Debug)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

impl<'de> Deserialize<'de> for ScheduleTrigger {
    fn deserialize<D>(deserializer: D) -> Result<ScheduleTrigger, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let time_regex = regex::Regex::new(r"^\d{2}:\d{2}:\d{2}$").unwrap();
        if let Some(time) = time_regex.captures(&s) {
            let parsed_time = crono::NaiveTime::parse_from_str(&time[0], "%H:%M:%S");
            if let Ok(t) = parsed_time {
                return Ok(ScheduleTrigger::Time(t));
            }
            return Err(serde::de::Error::custom("Invalid time format"));
        }

        match s.as_str() {
            "sunset" => Ok(ScheduleTrigger::Light(ScheduleLightTrigger::Sunset)),
            "sunrise" => Ok(ScheduleTrigger::Light(ScheduleLightTrigger::Sunrise)),
            _ => Err(serde::de::Error::custom("Invalid trigger")),
        }
    }
}

impl<'a> Deserialize<'a> for Schedule {
    fn deserialize<D>(deserializer: D) -> Result<Schedule, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let value = toml::Value::deserialize(deserializer)?;
        let table = value.as_table().unwrap();
        // deserialize trigger field
        let trigger = table.get("trigger").unwrap();
        let trigger = ScheduleTrigger::deserialize(trigger.clone()).unwrap();

        let has_temperature = table.contains_key("temperature");
        let has_preset = table.contains_key("preset");

        match (has_temperature, has_preset) {
            (true, true) => {
                return Err(serde::de::Error::custom(
                    "Cannot have both temperature and preset fields",
                ))
            }
            (false, false) => {
                return Err(serde::de::Error::custom(
                    "Must have either temperature or preset field",
                ))
            }
            (true, false) => {
                let temperature = table.get("temperature").unwrap();
                let temperature = Temperature::deserialize(temperature.clone()).unwrap();
                Ok(Schedule::Temperature {
                    trigger: trigger.clone(),
                    temperature,
                })
            }
            (false, true) => {
                let preset = table.get("preset").unwrap();
                let preset = preset.as_str().unwrap();
                Ok(Schedule::Preset {
                    trigger: trigger.clone(),
                    preset: preset.to_string(),
                })
            }
        }
    }
}

impl Configuration {
    pub fn parse_from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            location: None,
            backend: Backend::default(),
            mode: Mode::default(),
            schedule: vec![],
            presets: vec![
                Preset {
                    name: "day".to_string(),
                    temperature: 6500.0,
                },
                Preset {
                    name: "night".to_string(),
                    temperature: 4000.0,
                },
            ],
        }
    }
}
