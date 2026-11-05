use crate::config::{self, ModelConfig};
use crate::ui;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Makes sure `model` is fully downloaded, removes any other cached model,
/// and returns the local path to the weights.
pub async fn prepare_model(model: &ModelConfig) -> Result<PathBuf> {
    let path = config::get_model_path(model)?;
    ensure_model_exists(model.download_url, &path).await?;
    remove_other_models(model)?;
    Ok(path)
}

/// Downloads `url` to `target_path` unless a complete copy is already there.
///
/// An existing file is checked against the server's content-length; a size
/// mismatch (e.g. an interrupted download) triggers a re-download. If the
/// server can't be reached, an existing file is trusted so Sage stays usable
/// offline.
pub async fn ensure_model_exists(url: &str, target_path: &Path) -> Result<()> {
    let client = reqwest::Client::new();

    if target_path.exists() {
        let Ok(expected) = head_content_length(&client, url).await else {
            return Ok(());
        };
        let local = fs::metadata(target_path).map(|m| m.len()).unwrap_or(0);
        if local == expected {
            return Ok(());
        }
        println!(
            "{}",
            ui::warning(format!(
                "Local model is incomplete ({local} bytes, expected {expected}) — re-downloading."
            ))
        );
        fs::remove_file(target_path)?;
    } else {
        println!("{}", ui::warning("Model not found locally."));
    }

    println!(
        "Downloading from: {url}\nTarget: {}\n",
        target_path.display()
    );
    download(&client, url, target_path).await
}

async fn download(client: &reqwest::Client, url: &str, target_path: &Path) -> Result<()> {
    let res = client
        .get(url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
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
            .expect("valid progress template")
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
    file.flush()?;

    pb.finish_with_message("Download complete! Model cached.");
    Ok(())
}

async fn head_content_length(client: &reqwest::Client, url: &str) -> Result<u64> {
    client
        .head(url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .context("Failed to HEAD model file")?
        .content_length()
        .context("No content-length in HEAD response")
}

/// Deletes every cached model except `keep` to save disk space.
pub fn remove_other_models(keep: &ModelConfig) -> Result<()> {
    for model in config::ALL_MODELS {
        if model.name == keep.name {
            continue;
        }
        let path = config::get_model_path(model)?;
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to remove {}", path.display()))?;
            println!("Removed old model to save space: {}", path.display());
        }
    }
    Ok(())
}
