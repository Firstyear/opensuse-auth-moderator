use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tracing::*;

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub tls_bind_addr: SocketAddr,
    pub tls_pem_chain: PathBuf,
    pub tls_pem_key: PathBuf,
}

pub fn parse(config_path: &Path) -> Result<Config> {
    let mut file = File::open(config_path)
        .with_context(|| format!("Failed to open config from {}", config_path.display()))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .with_context(|| format!("Failed to read config from {}", config_path.display()))?;

    toml::from_str(&contents).context("Invalid configuration content")
}
