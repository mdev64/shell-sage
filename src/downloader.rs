use crate::config::{self, ModelConfig};
use crate::ui;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How long to wait for a TCP/TLS connection before giving up.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Abort a download that stops sending data for this long.
const STALL_TIMEOUT: Duration = Duration::from_secs(60);

/// Makes sure `model` is fully downloaded, removes any other cached model,
/// and returns the local path to the weights.
pub async fn prepare_model(model: &ModelConfig) -> Result<PathBuf> {
    let path = config::get_model_path(model)?;
    ensure_model_exists(model, &path).await?;
    remove_other_models(model)?;
    Ok(path)
}

/// Downloads `model` to `target_path` unless a complete copy is already there.
///
/// The check is fully offline: the cached file's size is compared with the
/// model's known `expected_size`. The network is only touched when the model
/// is missing or incomplete and has to be downloaded.
pub async fn ensure_model_exists(model: &ModelConfig, target_path: &Path) -> Result<()> {
    let expected = model.expected_size;
    match fs::metadata(target_path) {
        Ok(meta) if meta.len() == expected => return Ok(()),
        Ok(meta) => {
            println!(
                "{}",
                ui::warning(format!(
                    "Local model is incomplete ({} bytes, expected {expected}) — re-downloading.",
                    meta.len()
                ))
            );
            fs::remove_file(target_path)
                .with_context(|| format!("Failed to remove {}", target_path.display()))?;
        }
        Err(_) => println!("{}", ui::warning("Model not found locally.")),
    }

    println!(
        "Downloading from: {}\nTarget: {}\n",
        model.download_url,
        target_path.display()
    );
    let client = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(STALL_TIMEOUT)
        .build()
        .context("Failed to create HTTP client")?;
    download(&client, model.download_url, expected, target_path).await
}

async fn download(
    client: &reqwest::Client,
    url: &str,
    total_size: u64,
    target_path: &Path,
) -> Result<()> {
    let res = client
        .get(url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .context("Failed to initiate model download")?;

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

    // Stream into a sibling `.part` file and rename it into place only once
    // it is complete, so an interrupted download can never be mistaken for a
    // usable model (e.g. on a later offline run).
    let part_path = part_path(target_path);
    let written = match write_stream(res, &part_path, &pb).await {
        Ok(written) => written,
        Err(err) => {
            pb.abandon();
            let _ = fs::remove_file(&part_path);
            return Err(err);
        }
    };
    if written != total_size {
        pb.abandon();
        let _ = fs::remove_file(&part_path);
        anyhow::bail!("Download incomplete ({written} of {total_size} bytes). Please try again.");
    }
    fs::rename(&part_path, target_path).context("Failed to move downloaded model into place")?;

    pb.finish_with_message("Download complete! Model cached.");
    Ok(())
}

/// Path of the temporary file a download is streamed into.
fn part_path(target_path: &Path) -> PathBuf {
    let mut name = target_path.as_os_str().to_owned();
    name.push(".part");
    PathBuf::from(name)
}

/// Writes the response body to `path`, returning the number of bytes written.
async fn write_stream(res: reqwest::Response, path: &Path, pb: &ProgressBar) -> Result<u64> {
    let mut file = File::create(path).context("Failed to create local model file")?;
    let mut stream = res.bytes_stream();
    let mut written = 0u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Error reading byte chunk from stream")?;
        file.write_all(&chunk)
            .context("Failed to write model data to disk")?;
        written += chunk.len() as u64;
        pb.inc(chunk.len() as u64);
    }
    file.sync_all().context("Failed to flush model to disk")?;
    Ok(written)
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
