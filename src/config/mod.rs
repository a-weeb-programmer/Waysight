use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};
use thiserror::Error;
use toml::de::Error as DeserializeError;

use crate::USER_DATA;

#[derive(Serialize, Deserialize)]
pub struct WaysightConfig {
    #[serde(default = "default_num_workspaces")]
    pub workspaces: u32,
    pub input: InputConfig,
}

#[derive(Serialize, Deserialize)]
pub struct InputConfig {
    #[serde(default = "default_layout")]
    pub keyboard_layout: String,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not find a valid config file")]
    InvalidPath,
    #[error("Malformed config file: `{0}`")]
    MalformedConfig(DeserializeError),
    #[error("IO error occurred: `{0}`")]
    IOError(io::Error),
}

pub fn parse(user_set_path: Option<PathBuf>) -> Result<WaysightConfig, ConfigError> {
    let path = match user_set_path {
        Some(path) => path,
        None => {
            if let Ok(config_dir) = env::var("XDG_CONFIG_HOME") {
                let mut path = PathBuf::new();
                path.push(config_dir);
                path.push("waysight/config.toml");
                path
            } else {
                let home_dir = env::var("HOME").map_err(|_| ConfigError::InvalidPath)?;
                let mut path = PathBuf::new();
                path.push(home_dir);
                path.push(".config/waysight/config.toml");
                path
            }
        }
    };

    if !path.exists() {
        return Err(ConfigError::InvalidPath);
    }

    let file_content = fs::read_to_string(&path).map_err(|err| ConfigError::IOError(err))?;

    toml::from_str::<WaysightConfig>(&file_content).map_err(|err| ConfigError::MalformedConfig(err))
}

pub fn generate_config(user_path: Option<PathBuf>) -> Result<PathBuf, ConfigError> {
    let mut path = match user_path {
        Some(path) => path,
        None => {
            if let Ok(config_dir) = env::var("XDG_CONFIG_HOME") {
                let mut path = PathBuf::new();
                path.push(config_dir);
                path.push("waysight/config.toml");
                path
            } else {
                let home_dir = env::var("HOME").map_err(|_| ConfigError::InvalidPath)?;
                let mut path = PathBuf::new();
                path.push(home_dir);
                path.push(".config/waysight/config.toml");
                path
            }
        }
    };
    if !path.exists() {
        // I am so sorry
        let og_path = path.clone();
        let file_name = og_path.file_name().unwrap();
        path.pop();
        fs::create_dir_all(&path).map_err(|err| ConfigError::IOError(err))?;
        path.push(file_name);
        tracing::info!("passed directory creation");
    };

    let input_config = InputConfig {
        keyboard_layout: "us".to_owned(),
    };

    let config = WaysightConfig {
        workspaces: 10,
        input: input_config,
    };

    let config_str = toml::to_string_pretty::<WaysightConfig>(&config).unwrap();
    let mut config_file = File::create(path.clone()).map_err(|err| ConfigError::IOError(err))?;
    config_file
        .write(config_str.as_bytes())
        .map_err(|err| ConfigError::IOError(err))?;

    Ok(path)
}

impl WaysightConfig {
    pub fn load_config() -> WaysightConfig {
        let mutex_data = USER_DATA.lock().unwrap();
        match parse(mutex_data.config_path.clone()) {
            Ok(config) => {
                drop(mutex_data);
                return config;
            }
            Err(error) => match error {
                ConfigError::InvalidPath => {
                    tracing::error!(
                        "Cannot find a valid config file. Generating one automatically."
                    );
                    let config_path = generate_config(mutex_data.config_path.clone()).unwrap();
                    drop(mutex_data);
                    parse(Some(config_path)).unwrap()
                }
                ConfigError::MalformedConfig(err) => {
                    tracing::error!("Malformed config: {}", err);
                    panic!("Malformed config");
                }
                ConfigError::IOError(err) => {
                    tracing::error!(
                        "IO error encountered while loading waysight config: {}",
                        err
                    );
                    panic!("IO error");
                }
            },
        }
    }
}

pub fn default_layout() -> String {
    "us".to_owned()
}

pub fn default_num_workspaces() -> u32 {
    10
}
