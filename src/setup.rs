use crate::config::{self, AppConfig, ModelConfig};
use crate::{downloader, ui};
use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};
use std::env;
use std::process::Command;

/// Machines with at least this much RAM get the larger model recommended.
const LARGE_MODEL_MIN_RAM_GB: u64 = 16;

const KNOWN_SHELLS: [&str; 7] = [
    "/bin/zsh",
    "/bin/bash",
    "/bin/sh",
    "/bin/tcsh",
    "/bin/ksh",
    "/opt/homebrew/bin/fish",
    "/usr/local/bin/fish",
];

/// Runs the interactive `sage --config` wizard and saves the result.
pub async fn run_wizard() -> Result<()> {
    let theme = ColorfulTheme::default();
    println!("\n{}", ui::heading("=== Shell Sage Config ==="));

    let model = select_model(&theme)?;
    let model_path = config::get_model_path(model)?;
    downloader::ensure_model_exists(model, &model_path).await?;

    let shell = select_shell(&theme)?;
    let show_hints = ask_show_hints(&theme)?;

    step(4, "Clipboard");
    let auto_copy = Confirm::with_theme(&theme)
        .with_prompt("Automatically copy the command to the clipboard?")
        .default(true)
        .interact()?;

    config::save_config(&AppConfig {
        model: model.name.to_string(),
        shell,
        show_hints,
        auto_copy,
    })?;
    // Only now that the config points at the new model is it safe to delete
    // the other one; aborting the wizard earlier must not strand the old config.
    downloader::remove_other_models(model)?;

    println!(
        "\n{} to {}",
        ui::success("✓ Configuration saved"),
        config::config_file_path()?.display()
    );
    println!("Run `sage \"<your query>\"` to start using it.");
    Ok(())
}

fn step(n: u8, title: &str) {
    println!("\n{}", ui::bold(format!("Step {n}/4 — {title}")));
}

fn select_model(theme: &ColorfulTheme) -> Result<&'static ModelConfig> {
    step(1, "Select a model");
    let ram_gb = detect_ram_gb();
    let recommended = recommended_model(ram_gb);
    println!("Detected RAM: {}", ui::bold(format!("{ram_gb} GB")));
    println!(
        "Recommended model for {ram_gb} GB: {} ({})",
        ui::bold(recommended.name),
        recommended.summary
    );

    let items: Vec<String> = config::ALL_MODELS
        .iter()
        .map(|m| format!("{} — {}", m.name, m.summary))
        .collect();
    let default = config::ALL_MODELS
        .iter()
        .position(|m| m.name == recommended.name)
        .unwrap_or(0);
    let selected = Select::with_theme(theme)
        .with_prompt("Which model should Sage use?")
        .default(default)
        .items(&items)
        .interact()?;
    let model = config::ALL_MODELS[selected];

    let path = config::get_model_path(model)?;
    if path.exists() {
        println!("Model already cached: {}", path.display());
    }
    Ok(model)
}

fn select_shell(theme: &ColorfulTheme) -> Result<String> {
    step(2, "Default shell");
    let detected = detect_shell();

    let mut items: Vec<&str> = KNOWN_SHELLS.to_vec();
    items.push("Other (enter a custom shell path)");
    let other_idx = KNOWN_SHELLS.len();

    let default = KNOWN_SHELLS
        .iter()
        .position(|s| *s == detected)
        .unwrap_or(other_idx);
    let selected = Select::with_theme(theme)
        .with_prompt(format!(
            "Detected shell: {detected} — confirm or pick another"
        ))
        .default(default)
        .items(&items)
        .interact()?;

    if selected == other_idx {
        Ok(Input::<String>::with_theme(theme)
            .with_prompt("Enter the full path to your shell")
            .default(detected)
            .interact()?)
    } else {
        Ok(KNOWN_SHELLS[selected].to_string())
    }
}

fn ask_show_hints(theme: &ColorfulTheme) -> Result<bool> {
    step(3, "Code hints");
    println!("Example of the hints Sage prints after the command:\n");
    println!("{}", ui::success("$ find . -name \"*.mp4\" -type f"));
    println!("{}", ui::dim("# find: Searches the filesystem for files."));
    println!(
        "{}",
        ui::dim("# -name \"*.mp4\": Match files ending in .mp4.")
    );
    println!(
        "{}",
        ui::dim("# -type f: Only match regular files, not directories.")
    );
    println!();
    Ok(Confirm::with_theme(theme)
        .with_prompt("Show code hints like these after the command?")
        .default(true)
        .interact()?)
}

/// Picks the model suited to a machine with `ram_gb` of memory.
pub fn recommended_model(ram_gb: u64) -> &'static ModelConfig {
    if ram_gb >= LARGE_MODEL_MIN_RAM_GB {
        &config::GEMMA_E4B_CONFIG
    } else {
        &config::GEMMA_E2B_CONFIG
    }
}

/// Total RAM in whole GB via `sysctl`, falling back to 8 if unavailable.
pub fn detect_ram_gb() -> u64 {
    Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|bytes| bytes / (1024 * 1024 * 1024))
        .unwrap_or(8)
}

/// The user's login shell from `$SHELL`, defaulting to `/bin/zsh`.
pub fn detect_shell() -> String {
    env::var("SHELL")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "/bin/zsh".to_string())
}
