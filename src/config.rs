use anyhow::{Result, anyhow, bail};
use clap::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
};
use toml::{Table, Value as TomlValue};

use crate::{
    constants,
    model::{keybind::Keybind, theme::Theme},
};

#[derive(Parser, Debug)]
#[command(version, about, author, long_about = None)]
pub struct CliOptions {
    /// Path to the config file (default: <config_dir>/amusing/amusing.toml).
    #[arg(short = 'c', long = "config")]
    pub config_file: Option<PathBuf>,
}

pub struct Config {
    pub port: u16,
    pub theme: Theme,
    pub keybind: Keybind,
    pub seek_step: i64,
    pub volume_step: i8,
    pub speed_step: i16,
    pub library_group_by: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: constants::DEFAULT_PORT,
            theme: Theme::default(),
            keybind: Keybind::default(),
            seek_step: constants::DEFAULT_SEEK_STEP,
            volume_step: constants::DEFAULT_VOLUME_STEP,
            speed_step: constants::DEFAULT_SPEED_STEP,
            library_group_by: constants::DEFAULT_GROUP_BY
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

impl Config {
    pub fn try_from_file(path: Option<&Path>) -> Result<Self> {
        let default_path = dirs::config_dir()
            .ok_or(anyhow!("no config dir found on the system"))?
            .join(constants::DEFAULT_CONFIG_DIR)
            .join(constants::DEFAULT_CONFIG_FILE);
        let path = path.unwrap_or(&default_path);
        let content = fs::read_to_string(path)?;

        let mut config = Self::default();
        let table = content.parse::<Table>()?;
        for (key, val) in table {
            match (key.as_str(), val) {
                ("port", TomlValue::Integer(port)) => {
                    config.port = u16::try_from(port)?;
                }
                ("theme", TomlValue::Table(theme)) => {
                    config.theme = Theme::try_from(theme)?;
                }
                ("keybind", TomlValue::Table(keybind)) => {
                    config.keybind = Keybind::try_from(keybind)?;
                }
                ("seek_step", TomlValue::Integer(seek_step)) => {
                    config.seek_step = seek_step;
                }
                ("volume_step", TomlValue::Integer(volume_step)) => {
                    config.volume_step = i8::try_from(volume_step)?;
                }
                ("speed_step", TomlValue::Integer(speed_step)) => {
                    config.speed_step = i16::try_from(speed_step)?;
                }
                ("library_group_by", TomlValue::Array(library_group_by)) => {
                    config.library_group_by = library_group_by
                        .iter()
                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                        .collect();
                }
                (other, _) => bail!("invalid config key `{}`", other),
            }
        }

        Ok(config)
    }
}
