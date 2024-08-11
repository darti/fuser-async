use std::{fs, path::PathBuf};

use config::{builder::DefaultState, Config, ConfigBuilder, Environment, File, FileFormat};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use tracing::{debug, info};

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new().unwrap();
}

lazy_static! {
    static ref DIRS: ProjectDirs = ProjectDirs::from("", "", "rmk").unwrap();
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error loading config: {}", source))]
    LoadConfig { source: config::ConfigError },

    #[snafu(display("Error parsing config: {}", source))]
    ParseConfig { source: config::ConfigError },
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Configuration {
    pub cache: CacheConfiguration,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CacheConfiguration {
    pub root: String,
    pub mount_point: Option<PathBuf>,
}

pub struct Settings {
    config: Configuration,
    config_path: PathBuf,
}

impl Settings {
    fn default_config() -> Result<ConfigBuilder<DefaultState>, Error> {
        let config = Config::builder().add_source(File::from_str(
            include_str!("config/defaults.toml"),
            FileFormat::Toml,
        ));
        // .set_default("local.root", DIRS.data_local_dir().display().to_string())?;

        Ok(config)
    }
    pub fn new() -> Result<Self, Error> {
        let config_path = DIRS.config_dir().join("config.toml");

        debug!("Looking for config at: {}", config_path.to_string_lossy());

        let mut config = Settings::default_config()?;

        if config_path.exists() {
            info!("Config found: {}", config_path.to_string_lossy());
            config = config.add_source(File::from(config_path.clone()));
        }

        let config = config
            .add_source(Environment::with_prefix("rmk"))
            .build()
            .context(LoadConfigSnafu)?
            .try_deserialize()
            .context(ParseConfigSnafu)?;

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
}
