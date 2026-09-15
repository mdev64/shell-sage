use crate::config;
use crate::downloader;
use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};
use std::env;
use std::process::Command;

pub async fn run_wizard() -> Result<()> {
    let theme = ColorfulTheme::default();

    println!("\n\x1b[1;36m=== Shell Sage Config ===\x1b[0m");

    println!("\n\x1b[1mStep 1/4 — Select a model\x1b[0m");
    let ram_gb = detect_ram_gb();
    let recommended = if ram_gb >= 16 { "E4B" } else { "E2B" };
    println!("Detected RAM: \x1b[1m{} GB\x1b[0m", ram_gb);
    println!(
        "Recommended model for {} GB: \x1b[1m{}\x1b[0m ({})",
        ram_gb,
        recommended,
        if recommended == "E4B" {
            "more capable, higher memory"
        } else {
            "faster, lower memory"
        }
    );

    let model_options = [
        "E2B — faster, lower RAM footprint",
        "E4B — more capable, higher RAM",
    ];
    let model_default = if recommended == "E4B" { 1 } else { 0 };
    let model_sel = Select::with_theme(&theme)
        .with_prompt("Which model should Sage use?")
        .default(model_default)
        .items(model_options)
        .interact()?;
    let model = if model_sel == 0 { "E2B" } else { "E4B" };

    let model_config = config::get_model_config(model)?;
    let model_path = config::get_model_path(model_config)?;
    if model_path.exists() {
        println!("Model already cached: {}", model_path.display());
    } else {
        println!(
            "Model not found locally — downloading {} ...",
            model_config.expected_filename
        );
    }
    downloader::ensure_model_exists(model_config.download_url, &model_path).await?;
    downloader::remove_other_models(model_config)?;

    println!("\n\x1b[1mStep 2/4 — Default shell\x1b[0m");
    let detected_shell = detect_shell();
    let shells = [
        "/bin/zsh",
        "/bin/bash",
        "/bin/sh",
        "/bin/tcsh",
        "/bin/ksh",
        "/opt/homebrew/bin/fish",
        "/usr/local/bin/fish",
        "Other (enter a custom shell path)",
    ];
    let default_shell_idx = shells
        .iter()
        .position(|s| *s == detected_shell)
        .unwrap_or(7);
    let shell_sel = Select::with_theme(&theme)
        .with_prompt(format!(
            "Detected shell: {} — confirm or pick another",
            detected_shell
        ))
        .default(default_shell_idx)
        .items(shells)
        .interact()?;
    let shell = if shell_sel == 7 {
        Input::<String>::with_theme(&theme)
            .with_prompt("Enter the full path to your shell")
            .default(detected_shell.clone())
            .interact()?
    } else {
        shells[shell_sel].to_string()
    };

    println!("\n\x1b[1mStep 3/4 — Code hints\x1b[0m");
    println!("Example of the hints Sage prints after the command:");
    println!();
    println!("\x1b[1;32m$ find . -name \"*.mp4\" -type f\x1b[0m");
    println!("\x1b[90m# find: Searches the filesystem for files.\x1b[0m");
    println!("\x1b[90m# -name \"*.mp4\": Match files ending in .mp4.\x1b[0m");
    println!("\x1b[90m# -type f: Only match regular files, not directories.\x1b[0m");
    println!();
    let show_hints = Confirm::with_theme(&theme)
        .with_prompt("Show code hints like these after the command?")
        .default(true)
        .interact()?;

    println!("\n\x1b[1mStep 4/4 — Clipboard\x1b[0m");
    let auto_copy = Confirm::with_theme(&theme)
        .with_prompt("Automatically copy the command to the clipboard?")
        .default(true)
        .interact()?;

    config::save_config(&config::AppConfig {
        model: model.to_string(),
        shell,
        show_hints,
        auto_copy,
    })?;

    println!();
    println!(
        "\x1b[1;32m✓ Configuration saved\x1b[0m to {}",
        config::config_file_path()?.display()
    );
    println!("Run `sage \"<your query>\"` to start using it.");
    Ok(())
}

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

pub fn detect_shell() -> String {
    env::var("SHELL")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "/bin/zsh".to_string())
}
