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
        let mut file_config = FileConfig::default();

        let local_path = std::path::PathBuf::from("config.toml");

        let global_path = dirs::config_dir().map(|mut p| {
            p.push("moneta");
            p.push("config.toml");
            p
        });

        let config_path = if local_path.exists() {
            Some(local_path)
        } else {
            global_path.filter(|p| p.exists())
        };

        if let Some(path) = config_path {
            let content =
                fs::read_to_string(&path).with_context(|| format!("Error reading {:?}", path))?;
            file_config = toml::from_str(&content)
                .with_context(|| format!("Error parsing TOML at {:?}", path))?;
        }

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
}
