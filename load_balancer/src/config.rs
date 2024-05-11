use std::{fs, path::PathBuf};

use serde_derive::Deserialize;

use crate::LBError;

#[derive(Clone, Debug, Deserialize)]
pub struct Backend {
    pub url: String,
    pub healthcheck_path: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub healthcheck_interval_secs: usize,
    pub backends: Vec<Backend>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            healthcheck_interval_secs: 30,
            backends: Default::default(),
        }
    }
}

impl Config {
    pub fn new(path: &PathBuf) -> Result<Self, LBError> {
        let content = fs::read_to_string(path).map_err(|e| LBError::MissingConfigurationFile {
            config_file_path: path.clone(),
            source: e,
        })?;
        toml::from_str(&content).map_err(LBError::InvalidConfig)
    }
}
