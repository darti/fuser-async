use std::{fs, path::PathBuf};

use config::{Config, Environment, File, FileFormat};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use log::{debug, info};
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new();
}

lazy_static! {
    static ref DIRS: ProjectDirs = ProjectDirs::from("", "", "rmk").unwrap();
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Configuration {
    pub device: DeviceConfiguration,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DeviceConfiguration {
    pub ip: String,
    pub port: u16,
    pub login: String,
    pub password: String,
}

pub struct Settings {
    config: Configuration,
    config_path: PathBuf,
}

impl Settings {
    pub fn new() -> Self {
        let config_path = DIRS.config_dir().join("config.toml");

        info!("Looking for config at: {}", config_path.to_string_lossy());

        if config_path.exists() {
            debug!("Found config at: {}", config_path.to_string_lossy());

            let config: Configuration = Config::builder()
                .add_source(File::from_str(
                    include_str!("config/defaults.toml"),
                    FileFormat::Toml,
                ))
                .add_source(File::from(config_path.clone()))
                .add_source(Environment::with_prefix("rmk"))
                .build()
                .unwrap()
                .try_deserialize()
                .expect("Failed to read config");

            Settings {
                config,
                config_path,
            }
        } else {
            info!(
                "No config at: {}, creating new",
                config_path.to_string_lossy()
            );

            Self::init(config_path)
        }
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
}
