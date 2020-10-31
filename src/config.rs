use std::fs;
use std::io::prelude::*;

use crate::DynResult;

// This current implementation is incredibly barebones.
// It literally just reads/writes the last selected mode from the file.
//
// No TOML, No JSON, just raw text.
//
// It shouldn't be too hard to get a proper serde-based implementation up and
// running, it's moreso that it'll bump compile times for no good reason. I'll
// set that all up once I need the complexity.

pub struct Config {
    pub last_mode: usize,
}

fn get_cfg_file() -> DynResult<fs::File> {
    let proj_dirs = directories::ProjectDirs::from("com", "prilik", "surface-dial-daemon")
        .ok_or("could not open config directory")?;
    let cfg_folder = proj_dirs.config_dir();
    let cfg_file_path = proj_dirs.config_dir().join("config.txt");

    fs::create_dir_all(cfg_folder).map_err(|_| "could not create config dir")?;

    if !cfg_file_path.exists() {
        fs::write(&cfg_file_path, "0")?;
    }

    let cfg_file = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(cfg_file_path)
        .map_err(|e| format!("could not open config file: {}", e))?;

    Ok(cfg_file)
}

impl Config {
    pub fn from_disk() -> DynResult<Config> {
        let mut cfg_file = get_cfg_file()?;

        let mut content = String::new();
        cfg_file.read_to_string(&mut content)?;

        let last_mode = content.parse()?;

        Ok(Config { last_mode })
    }

    pub fn to_disk(&self) -> DynResult<()> {
        let mut cfg_file = get_cfg_file()?;
        cfg_file.write_all(format!("{}", self.last_mode).as_bytes())?;

        Ok(())
    }
}
