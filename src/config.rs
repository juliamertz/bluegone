use crate::{
    backends::{Backend, Temperature},
    utils::RemoveSeconds,
};
use anyhow::Result;
use chrono::{prelude as crono, DateTime};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
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

// Root is considered to be the users home directory
static CONFIG_PATHS: [&str; 3] = [
    "/.config/bluegone/config.toml",
    "/.config/bluegone.toml",
    "/.bluegone.toml",
];

impl Configuration {
    pub fn get_config() -> Result<Self> {
        #[allow(deprecated)] // deprecated because of windows support.
        let home = match std::env::home_dir() {
            Some(path) => path,
            None => return Ok(Configuration::default()),
        };

        for path in CONFIG_PATHS.iter() {
            let path = path.strip_prefix("/").expect("Path to be valid");
            let path = home.join(path);

            if !std::fs::metadata(&path).is_ok() {
                continue;
            }

            let content = std::fs::read_to_string(&path).expect("Could not read file");
            let config = match toml::from_str::<Self>(&content) {
                Ok(config) => config,
                Err(err) => {
                    eprintln!("Error parsing config file: {}", err);
                    return Ok(Configuration::default());
                }
            };

            return Ok(config);
        }

        Ok(Configuration::default())
    }
}
#[derive(Deserialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Static,
    Dynamic,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Preset {
    pub name: String,
    pub temperature: Temperature,
}

#[derive(Debug, Clone)]
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

impl ScheduleLightTrigger {
    pub fn get_time(&self, location: Location) -> Result<chrono::NaiveTime> {
        let now = crono::Local::now();
        let params = sunrise_sunset_calculator::SunriseSunsetParameters::new(
            now.timestamp(),
            location.latitude,
            location.longitude,
        );

        let time_from_millis = |millis: i64| {
            let as_time = DateTime::from_timestamp(millis, 0).unwrap().time();
            as_time
        };

        let result = params.calculate()?;

        match self {
            ScheduleLightTrigger::Sunset => Ok(time_from_millis(result.set).remove_seconds()),
            ScheduleLightTrigger::Sunrise => Ok(time_from_millis(result.rise).remove_seconds()),
        }
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

#[derive(Debug, Clone)]
pub enum ScheduleTrigger {
    Time(crono::NaiveTime),
    Light(ScheduleLightTrigger),
}

#[derive(Deserialize, Debug, Clone)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

impl Schedule {
    pub fn get_time(&self, location: &Option<Location>) -> Result<chrono::NaiveTime> {
        self.get_trigger().get_time(location)
    }
    pub fn get_trigger(&self) -> &ScheduleTrigger {
        match self {
            Schedule::Temperature { trigger, .. } => trigger,
            Schedule::Preset { trigger, .. } => trigger,
        }
    }
}

impl ScheduleTrigger {
    pub fn get_time(&self, location: &Option<Location>) -> Result<chrono::NaiveTime> {
        match self {
            ScheduleTrigger::Time(time) => Ok(*time),
            ScheduleTrigger::Light(state) => state.get_time(
                location
                    .clone()
                    .expect("Location should be set, unreachable state."),
            ),
        }
    }
}

pub fn get_next_scheduled_event(
    schedules: Vec<Schedule>,
    location: Option<Location>,
) -> Result<Schedule> {
    let now = chrono::Local::now();
    let mut result = schedules
        .clone()
        .into_iter()
        .filter(|schedule| {
            let time = schedule.get_time(&location).expect("No time found");
            return now.time() < time;
        })
        .collect::<Vec<_>>();

    result.sort_by(|a, b| {
        let a = a.get_time(&location).expect("No time found");
        let b = b.get_time(&location).expect("No time found");
        a.cmp(&b)
    });

    dbg!(&result);

    if result.is_empty() || result.len() == 0 {
        anyhow::bail!("No events found");
    }

    Ok(result[0].clone())
}

impl<'de> Deserialize<'de> for ScheduleTrigger {
    fn deserialize<D>(deserializer: D) -> Result<ScheduleTrigger, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let time_regex = regex::Regex::new(r"^\d{2}:\d{2}$").unwrap();
        if let Some(time) = time_regex.captures(&s) {
            let parsed_time = crono::NaiveTime::parse_from_str(&time[0], "%H:%M");
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
