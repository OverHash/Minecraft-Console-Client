use std::fs;

use serde::{Deserialize, Serialize};

const CONFIG_PATH: &str = "config.toml";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// If caching is enabled for disk storage
    pub cache_enabled: bool,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            cache_enabled: true,
        }
    }
}

pub fn get_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config = match fs::read_to_string(CONFIG_PATH) {
        Ok(config) => toml_edit::easy::from_str(&config)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // create config
            println!(
                "Existing config could not be found, creating new config at {}",
                CONFIG_PATH
            );
            let config = Config::default();

            let config_string = format!(
                "# The configuration file for the minecraft console client.\n{}",
                toml_edit::easy::to_string_pretty(&config)?
            );

            fs::write(CONFIG_PATH, config_string)?;
            config
        }
        Err(e) => return Err(Box::new(e)),
    };

    Ok(config)
}
