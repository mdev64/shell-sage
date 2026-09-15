use anyhow::{Context, Result};
use clap::error::{ContextKind, ErrorKind};
use clap::{ArgAction, Parser};
use indicatif::{ProgressBar, ProgressStyle};
use sage::{config, downloader, engine, output, prompt, setup};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = "sage",
    version,
    about = "Offline AI CLI Assistant",
    disable_version_flag = true,
    arg_required_else_help = true,
    after_help = "EXAMPLES:\n  sage \"find all mp4 files\"         Generate a shell command\n  sage -e \"ls -la\"                   Explain a terminal command\n  sage -a \"why is the sky blue?\"      Answer a question\n  sage --config                       Configure model, shell, hints, clipboard"
)]
struct Cli {
    /// Explain a terminal command and what it does (e.g. `sage -e "ls -la"`).
    #[arg(
        short = 'e',
        long = "explain",
        value_name = "COMMAND",
        num_args = 1..,
        allow_hyphen_values = true,
        conflicts_with_all = ["answer", "config", "query"]
    )]
    explain: Option<Vec<String>>,

    /// Answer any question using general knowledge (e.g. `sage -a "why is the sky blue?"`).
    #[arg(
        short = 'a',
        long = "ask",
        value_name = "QUESTION",
        num_args = 1..,
        allow_hyphen_values = true,
        conflicts_with_all = ["explain", "config", "query"]
    )]
    answer: Option<Vec<String>>,

    /// Run the interactive configuration wizard (model, shell, hints, clipboard).
    #[arg(short, long, conflicts_with_all = ["explain", "answer", "query"])]
    config: bool,

    /// Show the version number.
    #[arg(short = 'v', long = "version", action = ArgAction::Version)]
    version: (),

    /// Natural-language query to turn into a shell command.
    #[arg(value_name = "QUERY", required_unless_present_any = ["explain", "answer", "config"])]
    query: Vec<String>,
}

enum Mode {
    Command,
    Explain,
    Answer,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) if err.kind() == ErrorKind::UnknownArgument => {
            let bad = err
                .get(ContextKind::InvalidArg)
                .map(|v| v.to_string())
                .unwrap_or_default();
            eprintln!(
                "\x1b[1;31msage:\x1b[0m unknown command or option \x1b[1m{}\x1b[0m",
                bad
            );
            eprintln!();
            eprintln!("Run \x1b[1msage --help\x1b[0m to see all available commands.");
            std::process::exit(2);
        }
        Err(err) => err.exit(),
    };

    if cli.config {
        setup::run_wizard().await?;
        return Ok(());
    }

    let (mode, input) = if let Some(parts) = cli.explain {
        (Mode::Explain, parts.join(" "))
    } else if let Some(parts) = cli.answer {
        (Mode::Answer, parts.join(" "))
    } else {
        (Mode::Command, cli.query.join(" "))
    };

    let Some(app_config) = config::load_config()? else {
        eprintln!("\x1b[1;33mNo configuration found.\x1b[0m");
        eprintln!("Please run `sage --config` once to pick a model, shell, and preferences.");
        return Ok(());
    };

    let model_config = config::get_model_config(&app_config.model)?;
    let model_path = config::get_model_path(model_config)?;
    downloader::ensure_model_exists(model_config.download_url, &model_path).await?;
    downloader::remove_other_models(model_config)?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message("Thinking...");
    spinner.enable_steady_tick(Duration::from_millis(80));
    let engine = engine::InferenceEngine::new(&model_path)?;

    let formatted_prompt = match mode {
        Mode::Command => prompt::format_prompt(&input, &app_config.shell),
        Mode::Explain => prompt::format_explain_prompt(&input, &app_config.shell),
        Mode::Answer => prompt::format_answer_prompt(&input),
    };

    let raw_output = engine.generate(&formatted_prompt, 1024)?;
    spinner.finish_and_clear();

    let sanitized = output::sanitize_output(&raw_output);

    match mode {
        Mode::Explain | Mode::Answer => {
            let text = sanitized.trim();
            if text.is_empty() {
                eprintln!("Model returned an empty response.");
                return Ok(());
            }
            println!("\n{}", text);
        }
        Mode::Command => {
            let (suggested_cmd, notes) = output::parse_command_output(&sanitized);
            if suggested_cmd.is_empty() {
                eprintln!("Model returned an empty response.");
                return Ok(());
            }

            println!("\n\x1b[1;32m$ {}\x1b[0m", suggested_cmd);

            if app_config.show_hints && !notes.is_empty() {
                println!("\x1b[90m{}\x1b[0m", notes.join("\n"));
            }

            if app_config.auto_copy {
                match copy_to_clipboard(&suggested_cmd) {
                    Ok(()) => println!(
                        "\n\x1b[90m✓ Command copied to clipboard. Press '⌘ + V', then press Enter to run.\x1b[0m"
                    ),
                    Err(err) => eprintln!(
                        "\n\x1b[90mCould not copy to clipboard ({err}). Use the command above.\x1b[0m"
                    ),
                }
            } else {
                println!(
                    "\n\x1b[90m✓ Copy or type the command above, then press Enter to run it.\x1b[0m"
                );
            }
        }
    }

    Ok(())
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().context("failed to access clipboard")?;
    clipboard
        .set_text(text.to_owned())
        .context("failed to copy to clipboard")?;
    Ok(())
}
