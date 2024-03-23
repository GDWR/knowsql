use serde::Deserialize;
use tracing::{debug, warn};
use std::fs::read_to_string;
use toml;

const DEFAULT_CONFIG_PATH: &'static str = "/etc/knowsql/config.toml";

#[derive(Debug, Deserialize)]
pub struct Config {
    pub data_dir: String,
    pub port: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            data_dir: "./data".to_string(),
            port: 2288,
        }
    }
}

/// Use environment variable KNOWSQL_CONFIG or DEFAULT_CONFIG_PATH to load the config
///   if it doesn't exist, use default config
pub fn get_config() -> Config {
    let config_path = std::env::var("KNOWSQL_CONFIG").unwrap_or(DEFAULT_CONFIG_PATH.to_string());

    if let Ok(config) = read_to_string(config_path.clone()) {
        debug!(path = config_path, "loading configuration");
        toml::from_str(&config).unwrap()
    } else {
        warn!(path = config_path, "unable to read configuration. using defaults.");
        Config::default()
    }
}
