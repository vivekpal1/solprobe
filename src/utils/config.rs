use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_url: String,
    pub update_interval: u64,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Config::get_config_path()?;
        if config_path.exists() {
            let config_str = fs::read_to_string(config_path)?;
            Ok(toml::from_str(&config_str)?)
        } else {
            let default_config = Config {
                default_url: "https://api.mainnet-beta.solana.com".to_string(),
                update_interval: 5,
            };
            let toml_str = toml::to_string(&default_config)?;
            fs::write(config_path, toml_str)?;
            Ok(default_config)
        }
    }

    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let mut path = dirs::config_dir().ok_or("Failed to get config directory")?;
        path.push("solprobe");
        fs::create_dir_all(&path)?;
        path.push("config.toml");
        Ok(path)
    }
}