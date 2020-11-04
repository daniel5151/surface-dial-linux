use std::fs;
use std::io::prelude::*;

use crate::error::{Error, Result};

// This current implementation is incredibly barebones and brittle.
// It literally just reads/writes the last selected mode from the file as a raw
// index into the modes array. That's it.
//
// No TOML, No JSON, just raw text.
//
// While it wouldn't be too hard to get a proper serde-based implementation up
// and running, it'll just bump compile times up for no good reason, so I'll
// only set it up once I need the added complexity.

// TODO: stop using strings for errors lol

pub struct Config {
    pub last_mode: usize,
}

fn get_cfg_file() -> Result<fs::File> {
    let proj_dirs = directories::ProjectDirs::from("com", "prilik", "surface-dial-daemon")
        .ok_or_else(|| Error::ConfigFile("could not open config directory".into()))?;
    let cfg_folder = proj_dirs.config_dir();
    let cfg_file_path = proj_dirs.config_dir().join("config.txt");

    fs::create_dir_all(cfg_folder)
        .map_err(|e| Error::ConfigFile(format!("could not create config dir: {}", e)))?;

    if !cfg_file_path.exists() {
        fs::write(&cfg_file_path, "0")
            .map_err(|e| Error::ConfigFile(format!("could not write to config file: {}", e)))?;
    }

    let cfg_file = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(cfg_file_path)
        .map_err(|e| Error::ConfigFile(format!("could not open config file: {}", e)))?;

    Ok(cfg_file)
}

impl Config {
    pub fn from_disk() -> Result<Config> {
        let mut cfg_file = get_cfg_file()?;

        let mut content = String::new();
        cfg_file
            .read_to_string(&mut content)
            .map_err(|e| Error::ConfigFile(format!("could not read the config file: {}", e)))?;

        let last_mode = content
            .parse()
            .map_err(|e| Error::ConfigFile(format!("could not parse the config file: {}", e)))?;

        Ok(Config { last_mode })
    }

    pub fn to_disk(&self) -> Result<()> {
        let mut cfg_file = get_cfg_file()?;
        cfg_file
            .write_all(format!("{}", self.last_mode).as_bytes())
            .map_err(|e| Error::ConfigFile(format!("could not write to the config file: {}", e)))?;

        Ok(())
    }
}
