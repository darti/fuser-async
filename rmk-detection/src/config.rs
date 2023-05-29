use std::{fs, path::PathBuf};

use config::{Config, Environment, File, FileFormat};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::errors::RmkDetectionError;

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new().unwrap();
}

lazy_static! {
    static ref DIRS: ProjectDirs = ProjectDirs::from("", "", "rmk").unwrap();
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Configuration {
    pub device: DeviceConfiguration,
    pub remarkable: RemarkableConfiguration,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DeviceConfiguration {
    pub endpoint: String,
    pub user: String,
    pub key_file: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RemarkableConfiguration {
    pub base: String,
}

pub struct Settings {
    config: Configuration,
    config_path: PathBuf,
}

impl Settings {
    pub fn new() -> Result<Self, RmkDetectionError> {
        let config_path = DIRS.config_dir().join("config.toml");

        debug!("Looking for config at: {}", config_path.to_string_lossy());

        let mut config = Config::builder().add_source(File::from_str(
            include_str!("config/defaults.toml"),
            FileFormat::Toml,
        ));

        if config_path.exists() {
            info!("Config found: {}", config_path.to_string_lossy());
            config = config.add_source(File::from(config_path.clone()));
        }

        let config = config
            .add_source(Environment::with_prefix("rmk"))
            .build()?
            .try_deserialize()?;

        Ok(Settings {
            config,
            config_path,
        })
    }

    pub fn init(config_path: PathBuf) -> Settings {
        let config: Configuration = Config::builder()
            .add_source(File::from_str(
                include_str!("config/defaults.toml"),
                FileFormat::Toml,
            ))
            .build()
            .unwrap()
            .try_deserialize()
            .expect("Failed to read config");

        fs::create_dir_all(DIRS.config_dir()).unwrap();

        fs::write(
            &config_path.clone(),
            toml::to_string_pretty(&config).unwrap(),
        )
        .expect("Failed to write config");

        let settings = Settings {
            config,
            config_path,
        };

        settings.save();

        settings
    }

    pub fn save(&self) {
        fs::write(
            &self.config_path.clone(),
            toml::to_string_pretty(&self.config).unwrap(),
        )
        .expect("Failed to write config");
    }

    pub fn config(&self) -> &Configuration {
        &self.config
    }

    pub fn remarkable(&self) -> &RemarkableConfiguration {
        &self.config.remarkable
    }
}
