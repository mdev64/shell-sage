use crate::config;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub async fn ensure_model_exists(url: &str, target_path: &Path) -> Result<()> {
    if target_path.exists() {
        match head_content_length(url).await {
            Ok(expected) => {
                let local = std::fs::metadata(target_path).map(|m| m.len()).unwrap_or(0);
                if local == expected {
                    return Ok(());
                }
                println!(
                    "\x1b[1;33mLocal model is incomplete ({} bytes, expected {}) — re-downloading.\x1b[0m",
                    local, expected
                );
                std::fs::remove_file(target_path)?;
            }
            Err(_) => return Ok(()),
        }
    }

    println!("\x1b[1;33mModel not found locally.\x1b[0m");
    println!(
        "Downloading from: {}\nTarget: {}\n",
        url,
        target_path.display()
    );

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .send()
        .await
        .context("Failed to initiate model download")?;

    let total_size = res
        .content_length()
        .context("Failed to get content length from server")?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{msg}\n[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("Downloading model weights...");

    let mut file = File::create(target_path).context("Failed to create local model file")?;
    let mut stream = res.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Error reading byte chunk from stream")?;
        file.write_all(&chunk)?;
        pb.inc(chunk.len() as u64);
    }

    pb.finish_with_message("Download complete! Model cached.");
    Ok(())
}

async fn head_content_length(url: &str) -> Result<u64> {
    let client = reqwest::Client::new();
    let res = client
        .head(url)
        .send()
        .await
        .context("Failed to HEAD model file")?;
    res.content_length()
        .context("No content-length in HEAD response")
}

pub fn remove_other_models(keep: &config::ModelConfig) -> Result<()> {
    for cfg in [config::GEMMA_E2B_CONFIG, config::GEMMA_E4B_CONFIG] {
        if cfg.expected_filename != keep.expected_filename {
            let path = config::get_model_path(&cfg)?;
            if path.exists() {
                std::fs::remove_file(&path)
                    .with_context(|| format!("Failed to remove {}", path.display()))?;
                println!("Removed old model to save space: {}", path.display());
            }
        }
    }
    Ok(())
}
