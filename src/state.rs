use anyhow::Result;
use bluegone::Pid;
use std::{fmt::Display, path::PathBuf, str::FromStr};

use crate::{
    config::Mode,
    utils::{self},
};

fn get_data_path() -> PathBuf {
    let cache_dir = match std::env::var("XDG_CACHE_DIR") {
        Ok(path) => PathBuf::from_str(&path).expect("Valid path"),
        Err(_) => utils::home_dir().join(".cache"),
    };

    cache_dir.join("bluegone")
}

pub trait StateFile {
    fn write_state(value: Self) -> Result<()>
    where
        Self: Display;
    fn read_state() -> Result<Self>
    where
        Self: Sized + TryFrom<String>;
}


#[macro_export]
macro_rules! impl_state_file {
    ( $x:ident, $name:literal ) => {
        impl StateFile for $x {
            fn write_state(value: Self) -> Result<()> {
                let path = get_data_path().join($name);
                std::fs::create_dir_all(get_data_path())?;
                std::fs::write(path, value.to_string())?;
                Ok(())
            }

            fn read_state() -> Result<Self>
            where
                Self: Sized,
            {
                let path = get_data_path().join($name);
                let content = std::fs::read(path)?;
                Self::try_from(String::from_utf8(content)?)
            }
        }
    };
}

impl_state_file!(Mode, "mode");
impl_state_file!(Pid, "pid");

// impl StateFile for Mode {
//     fn state_path() -> PathBuf {
//         get_data_path().join("mode")
//     }
//
//     fn write_state(value: Self) -> Result<()> {
//         let path = Self::state_path();
//         std::fs::create_dir_all(get_data_path())?;
//         std::fs::write(path, value.to_string())?;
//         Ok(())
//     }
//
//     fn read_state() -> Result<Self>
//     where
//         Self: Sized,
//     {
//         let content = std::fs::read(Self::state_path())?;
//         Self::try_from(String::from_utf8(content)?)
//     }
// }
//
// impl StateFile for Pid {
//     fn state_path() -> PathBuf {
//         get_data_path().join("pid")
//     }
//
//     fn write_state(value: Self) -> Result<()> {
//         let path = Self::state_path();
//         std::fs::create_dir_all(get_data_path())?;
//         std::fs::write(path, value.to_string())?;
//         Ok(())
//     }
//
//     fn read_state() -> Result<Self>
//     where
//         Self: Sized,
//     {
//         let content = std::fs::read(Self::state_path())?;
//         Self::try_from(String::from_utf8(content)?)
//     }
// }
