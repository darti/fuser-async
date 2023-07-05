use std::{fs, path::PathBuf};

use config::{builder::DefaultState, Config, ConfigBuilder, Environment, File, FileFormat};
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
    pub local: LocalConfiguration,
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

#[derive(Deserialize, Serialize, Clone)]
pub struct LocalConfiguration {
    pub root: String,
    pub base: String,
    pub overlay: String,
}

impl LocalConfiguration {
    pub fn overlay(&self) -> PathBuf {
        PathBuf::from(&self.root).join(&self.overlay)
    }

    pub fn base(&self) -> PathBuf {
        PathBuf::from(&self.root).join(&self.base)
    }
}

pub struct Settings {
    config: Configuration,
    config_path: PathBuf,
}

impl Settings {
    fn default_config() -> Result<ConfigBuilder<DefaultState>, RmkDetectionError> {
        let config = Config::builder()
            .add_source(File::from_str(
                include_str!("config/defaults.toml"),
                FileFormat::Toml,
            ))
            .set_default("local.root", DIRS.data_local_dir().display().to_string())?;

        Ok(config)
    }
    pub fn new() -> Result<Self, RmkDetectionError> {
        let config_path = DIRS.config_dir().join("config.toml");

        debug!("Looking for config at: {}", config_path.to_string_lossy());

        let mut config = Settings::default_config()?;

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

    pub fn save(&self) {
        fs::create_dir_all(DIRS.config_dir()).unwrap();

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

    pub fn local(&self) -> &LocalConfiguration {
        &self.config.local
    }
}
