// use anyhow::Result;
use bluegone::StateFileName;
use std::{fmt::Display, path::PathBuf, str::FromStr};

use crate::utils::{self};

pub fn write<T>(value: T) -> anyhow::Result<()>
where
    T: Display + StateFileName,
{
    let path = get_data_path().join(T::name());
    std::fs::create_dir_all(get_data_path())?;
    std::fs::write(path, value.to_string())?;
    Ok(())
}

pub fn read<T>() -> anyhow::Result<T>
where
    T: Sized + StateFileName + TryFrom<String>,
    <T as std::convert::TryFrom<std::string::String>>::Error: std::fmt::Debug,
{
    let path = get_data_path().join(T::name());
    let content = std::fs::read(path)?;
    let string = String::from_utf8(content)?;
    let result: T = string.try_into().unwrap();
    Ok(result)
}

fn get_data_path() -> PathBuf {
    let cache_dir = match std::env::var("XDG_CACHE_DIR") {
        Ok(path) => PathBuf::from_str(&path).expect("Valid path"),
        Err(_) => utils::home_dir().join(".cache"),
    };

    cache_dir.join("bluegone")
}
