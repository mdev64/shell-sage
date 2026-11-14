use anyhow::Result;
use clap::error::{ContextKind, ErrorKind};
use clap::{ArgAction, Parser};
use sage::config::{self, AppConfig};
use sage::{downloader, engine, output, prompt, setup, ui};

/// Upper bound on prompt + generated tokens for a single run.
const MAX_TOKENS: usize = engine::CONTEXT_SIZE as usize;

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

impl Cli {
    /// Parses argv, reporting unknown flags as "unknown command" and exiting.
    fn parse_or_exit() -> Self {
        match Self::try_parse() {
            Ok(cli) => cli,
            Err(err) if err.kind() == ErrorKind::UnknownArgument => {
                let bad = err
                    .get(ContextKind::InvalidArg)
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                eprintln!(
                    "{} unknown command or option {}\n",
                    ui::error("sage:"),
                    ui::bold(bad)
                );
                eprintln!(
                    "Run {} to see all available commands.",
                    ui::bold("sage --help")
                );
                std::process::exit(2);
            }
            Err(err) => err.exit(),
        }
    }

    /// The requested mode and its joined input text.
    fn into_mode(self) -> (Mode, String) {
        match (self.explain, self.answer) {
            (Some(parts), _) => (Mode::Explain, parts.join(" ")),
            (_, Some(parts)) => (Mode::Answer, parts.join(" ")),
            _ => (Mode::Command, self.query.join(" ")),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_or_exit();
    if cli.config {
        return setup::run_wizard().await;
    }
    let (mode, input) = cli.into_mode();

    let Some(app_config) = config::load_config()? else {
        // Not an error: just guide the user to the one-time setup.
        println!("{}", ui::warning("Sage isn't configured yet."));
        println!(
            "Run {} once to pick a model, shell, and preferences.",
            ui::bold("sage --config")
        );
        return Ok(());
    };

    let model_config = config::get_model_config(&app_config.model)?;
    let model_path = downloader::prepare_model(model_config).await?;

    let formatted_prompt = match mode {
        Mode::Command => prompt::format_prompt(&input, &app_config.shell),
        Mode::Explain => prompt::format_explain_prompt(&input, &app_config.shell),
        Mode::Answer => prompt::format_answer_prompt(&input),
    };
    let spinner = ui::spinner("Thinking...");
    let result = engine::InferenceEngine::new(&model_path)
        .and_then(|engine| engine.generate(&formatted_prompt, MAX_TOKENS));
    // Clear the spinner before reporting either the answer or an error.
    spinner.finish_and_clear();
    let raw_output = result?;

    let sanitized = output::sanitize_output(&raw_output);
    match mode {
        Mode::Command => print_command(&sanitized, &app_config),
        Mode::Explain | Mode::Answer => print_prose(&sanitized),
    }
    Ok(())
}

fn print_prose(sanitized: &str) {
    let text = sanitized.trim();
    if text.is_empty() {
        eprintln!("Model returned an empty response.");
    } else {
        println!("\n{text}");
    }
}

fn print_command(sanitized: &str, app_config: &AppConfig) {
    let (command, notes) = output::parse_command_output(sanitized);
    if command.is_empty() {
        eprintln!("Model returned an empty response.");
        return;
    }

    println!("\n{}", ui::success(format!("$ {command}")));
    if app_config.show_hints && !notes.is_empty() {
        println!("{}", ui::dim(notes.join("\n")));
    }

    if !app_config.auto_copy {
        println!(
            "\n{}",
            ui::dim("✓ Copy or type the command above, then press Enter to run it.")
        );
        return;
    }
    match ui::copy_to_clipboard(&command) {
        Ok(()) => println!(
            "\n{}",
            ui::dim("✓ Command copied to clipboard. Press '⌘ + V', then press Enter to run.")
        ),
        Err(err) => eprintln!(
            "\n{}",
            ui::dim(format!(
                "Could not copy to clipboard ({err}). Use the command above."
            ))
        ),
    }
}
