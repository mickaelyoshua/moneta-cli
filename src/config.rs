#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("DATABASE_URL is mandatory. Define via flag (--database-url), environment variable or at ~/.config/moneta/config.toml")]
    MissingDatabaseUrl,
    #[error("Error reading {path:?}: {source}")]
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("Error parsing TOML at {path:?}: {source}")]
    Parse {
        path: std::path::PathBuf,
        source: toml::de::Error,
    },
}
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Default)]
struct FileConfig {
    database_url: Option<String>,
    max_connections: Option<u32>,
}

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub max_connections: u32,
}

impl Config {
    pub fn load(cli_args: crate::cli::ConfigArgs) -> Result<Self, ConfigError> {
        let config_path = Self::find_config_path();
        let file_config = Self::load_file_config(config_path)?;

        let database_url = cli_args
            .database_url
            .or(file_config.database_url)
            .ok_or(ConfigError::MissingDatabaseUrl)?;

        let max_connections = cli_args
            .max_connections
            .or(file_config.max_connections)
            .unwrap_or(10);

        Ok(Self {
            database_url,
            max_connections,
        })
    }

    fn find_config_path() -> Option<std::path::PathBuf> {
        let local_path = std::path::PathBuf::from("config.toml");
        if local_path.exists() {
            return Some(local_path);
        }

        dirs::config_dir()
            .map(|mut p| {
                p.push("moneta");
                p.push("config.toml");
                p
            })
            .filter(|p| p.exists())
    }

    fn load_file_config(path: Option<std::path::PathBuf>) -> Result<FileConfig, ConfigError> {
        let Some(path) = path else {
            return Ok(FileConfig::default());
        };

        let content = fs::read_to_string(&path)
            .map_err(|source| ConfigError::Io { path: path.clone(), source })?;

        toml::from_str(&content)
            .map_err(|source| ConfigError::Parse { path, source })
    }
}
