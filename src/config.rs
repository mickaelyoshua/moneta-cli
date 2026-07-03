use anyhow::Context;
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
    pub fn load(cli_args: crate::cli::ConfigArgs) -> anyhow::Result<Self> {
        let config_path = Self::find_config_path();
        let file_config = Self::load_file_config(config_path)?;

        let database_url = cli_args
            .database_url
            .or(file_config.database_url)
            .context("DATABASE_URL is mandatory. Define via flag (--database-url), environment variable or at ~/.config/moneta/config.toml")?;

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

    fn load_file_config(path: Option<std::path::PathBuf>) -> anyhow::Result<FileConfig> {
        let Some(path) = path else {
            return Ok(FileConfig::default());
        };

        let content =
            fs::read_to_string(&path).with_context(|| format!("Error reading {:?}", path))?;

        toml::from_str(&content).with_context(|| format!("Error parsing TOML at {:?}", path))
    }
}
