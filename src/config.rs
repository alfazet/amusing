use anyhow::{Result, anyhow, bail};
use clap::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
};
use toml::{Table, Value as TomlValue};

use crate::{constants, model::theme::Theme};

#[derive(Parser, Debug)]
#[command(version, about, author, long_about = None)]
pub struct CliOptions {
    /// Path to the config file (default: <config_dir>/amusing/amusing.toml).
    #[arg(short = 'c', long = "config")]
    pub config_file: Option<PathBuf>,

    /// Port on which your musing instance is listening (default: 2137).
    #[arg(short = 'p', long = "port")]
    pub port: Option<u16>,
}

pub struct Config {
    pub port: u16,
    pub volume_step: i8,
    pub speed_step: i16,
    pub theme: Theme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: constants::DEFAULT_PORT,
            volume_step: constants::DEFAULT_VOLUME_STEP,
            speed_step: constants::DEFAULT_SPEED_STEP,
            theme: Theme::default(),
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
                (other, _) => bail!("invalid config key `{}`", other),
            }
        }

        Ok(config)
    }

    pub fn merge_with_cli(self, cli_opts: CliOptions) -> Self {
        Self {
            port: cli_opts.port.unwrap_or(self.port),
            ..self
        }
    }
}
