use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ModelConfig {
    pub download_url: &'static str,
    pub expected_filename: &'static str,
}

pub const GEMMA_E2B_CONFIG: ModelConfig = ModelConfig {
    download_url: "https://models.r2-cf.mdev64.com/gemma4/gemma-4-E2B-it-Q4_K_M.gguf",
    expected_filename: "gemma-4-E2B-it-Q4_K_M.gguf",
};

pub const GEMMA_E4B_CONFIG: ModelConfig = ModelConfig {
    download_url: "https://models.r2-cf.mdev64.com/gemma4/gemma-4-E4B-it-Q4_K_M.gguf",
    expected_filename: "gemma-4-E4B-it-Q4_K_M.gguf",
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub model: String,
    pub shell: String,
    pub show_hints: bool,
    pub auto_copy: bool,
}

pub fn get_model_config(name: &str) -> Result<&'static ModelConfig> {
    match name {
        "E2B" => Ok(&GEMMA_E2B_CONFIG),
        "E4B" => Ok(&GEMMA_E4B_CONFIG),
        other => bail!("Unknown model '{other}' in configuration. Run `sage --config` to fix it."),
    }
}

pub fn get_cache_dir() -> Result<PathBuf> {
    let path = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".cache/sage");
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}

pub fn get_model_path(config: &ModelConfig) -> Result<PathBuf> {
    let cache_dir = get_cache_dir()?;
    Ok(cache_dir.join(config.expected_filename))
}

pub fn config_file_path() -> Result<PathBuf> {
    Ok(get_cache_dir()?.join("config.json"))
}

pub fn load_config() -> Result<Option<AppConfig>> {
    load_config_from(&config_file_path()?)
}

pub fn load_config_from(path: &Path) -> Result<Option<AppConfig>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(e) => {
            eprintln!("\x1b[90mWarning: could not read config ({}).\x1b[0m", e);
            return Ok(None);
        }
    };
    match serde_json::from_str(&raw) {
        Ok(cfg) => Ok(Some(cfg)),
        Err(e) => {
            eprintln!(
                "\x1b[90mWarning: config is invalid ({}). Run `sage --config` to recreate it.\x1b[0m",
                e
            );
            Ok(None)
        }
    }
}

pub fn save_config(cfg: &AppConfig) -> Result<()> {
    save_config_to(cfg, &config_file_path()?)
}

pub fn save_config_to(cfg: &AppConfig, path: &Path) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(cfg)?)
        .with_context(|| format!("Failed to write {}", path.display()))
}
