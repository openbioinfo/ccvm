use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const DEFAULT_REGISTRY: &str = "https://registry.npmmirror.com";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub registry: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            registry: DEFAULT_REGISTRY.to_string(),
        }
    }
}

fn ccvm_dir() -> PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join(".ccvm")
}

pub fn config_path() -> PathBuf {
    ccvm_dir().join("config.toml")
}

pub fn versions_dir() -> PathBuf {
    ccvm_dir().join("versions")
}

pub fn bin_dir() -> PathBuf {
    ccvm_dir().join("bin")
}

pub fn cache_dir() -> PathBuf {
    ccvm_dir().join("cache")
}

pub fn codex_dir() -> PathBuf {
    ccvm_dir().join("codex")
}

pub fn codex_versions_dir() -> PathBuf {
    codex_dir().join("versions")
}

pub fn codex_cache_dir() -> PathBuf {
    codex_dir().join("cache")
}

#[allow(dead_code)]
pub fn current_file() -> PathBuf {
    ccvm_dir().join("current")
}

pub fn codex_current_file() -> PathBuf {
    codex_dir().join("current")
}

pub fn ensure_dirs() -> Result<()> {
    let dirs = [
        versions_dir(),
        bin_dir(),
        cache_dir(),
        codex_versions_dir(),
        codex_cache_dir(),
    ];
    for d in &dirs {
        fs::create_dir_all(d)
            .with_context(|| format!("failed to create directory: {}", d.display()))?;
    }
    Ok(())
}

pub fn load_config() -> Result<Config> {
    let path = config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).with_context(|| "failed to read config.toml")?;
        toml::from_str(&content).with_context(|| "failed to parse config.toml")
    } else {
        Ok(Config::default())
    }
}

pub fn save_config(config: &Config) -> Result<()> {
    ensure_dirs()?;
    let content = toml::to_string_pretty(config).with_context(|| "failed to serialize config")?;
    fs::write(config_path(), content).with_context(|| "failed to write config.toml")?;
    Ok(())
}

pub fn get_registry() -> String {
    load_config()
        .map(|c| c.registry)
        .unwrap_or_else(|_| DEFAULT_REGISTRY.to_string())
}

pub fn set_registry(url: &str) -> Result<()> {
    let mut config = load_config()?;
    config.registry = url.to_string();
    save_config(&config)?;
    Ok(())
}

#[allow(dead_code)]
pub fn get_versions_dir() -> PathBuf {
    versions_dir()
}
