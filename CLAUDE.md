# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Sage is an offline CLI assistant for macOS Apple Silicon: it takes a natural-language query (e.g. `sage "find all files modified today"`), runs a Gemma 4 model locally via `llama-cpp-2` (Metal backend) to produce a shell command, prints the command to the terminal, and (optionally) copies it to the system clipboard so the user can paste it at the shell prompt (`⌘ + V`) and press Enter. Sage never executes anything itself. The model weights (a single GGUF file) are downloaded on first use and cached at `~/.cache/sage/`.

Before first use, the user runs `sage --config`, a terminal wizard that picks which model to use (Gemma 4 **E2B** or **E4B**, guided by detected RAM via `sysctl`), confirms the user's shell, and sets two preferences (show `#` hints, auto-copy to clipboard). Preferences are persisted to `~/.cache/sage/config.json` and respected on every run; if the file is missing, sage tells the user to run `sage --config`. The wizard downloads the chosen model if needed and deletes the other cached model to save space. Model downloads verify against the server's content-length so interrupted downloads are re-downloaded.

## Commands

- Build: `cargo build`
- Run with a query: `cargo run -- "<natural language query>"` — generates a shell command
- Explain a terminal command: `cargo run -- -e "ls -la"` (or `--explain`) — explains what the command does
- Ask a general question: `cargo run -- -a "why is the sky blue?"` (or `--ask`) — answers from general knowledge
- Configure: `cargo run -- --config` (interactive wizard; writes `~/.cache/sage/config.json`)
- Help: `sage --help` — lists every command and usage examples
- Version: `sage -v` / `sage --version`
- Any other `-flag` (e.g. `sage -x`) is reported as an unknown command
- Check: `cargo check` (faster iteration than full build)
- Lint: `cargo clippy`

Note: `engine.rs` uses the `llama-cpp-2` (locked at 0.1.154) sampling/token APIs as of that version — `LlamaTokenDataArray::sample_token_greedy()` for greedy sampling (the old `ctx.sample_token_greedy` was removed) and `LlamaModel::token_to_piece(token, &mut decoder, false, None)` for decoding (the old `token_to_str` is deprecated and needs a `Special` arg). `encoding_rs` is a direct dependency solely to provide the `UTF_8` decoder for `token_to_piece`. There is no test suite.

## Architecture

Flow in `main.rs`: parse args with clap (`Cli::try_parse`; `--config` short-circuits into `setup::run_wizard`; `-v`/`--version` prints the version; unknown `-` flags are caught via `ErrorKind::UnknownArgument` and reported as "unknown command"; bare `sage` prints `--help` via `arg_required_else_help`) → pick a `Mode` (`Command` for `sage <query>`, `Explain` for `sage -e <cmd>`, `Answer` for `sage -a <question>`) → load `config::AppConfig` from `~/.cache/sage/config.json` (error out with a hint to run `sage --config` if missing) → `config::get_model_config` maps the configured model name to its `ModelConfig` → `downloader::ensure_model_exists` (downloads if absent, re-downloads on size mismatch, deletes the other cached model) → `engine::InferenceEngine::new` loads model into memory → build the mode-specific prompt (`prompt::format_prompt` for commands, `format_explain_prompt` for `-e`, `format_answer_prompt` for `-a`) → `engine.generate` (greedy sampling behind a repetition-penalty sampler, up to 1024 tokens) → sanitize the raw output (strip echoed Gemma `<start_of_turn>`/`<end_of_turn>` tokens) → in Explain/Answer mode print the prose directly; in Command mode parse output (`parse_command_output`: first non-`#` line = command, `# `-prefixed lines = notes) → print the command (`$ <cmd>`) once, then print notes if `show_hints`, then optionally copy to clipboard if `auto_copy` (`copy_to_clipboard`, via `arboard`), then print the confirmation guiding the user to paste and press Enter. The command appears exactly once on stdout.

- `src/config.rs` — `ModelConfig` struct, the `GEMMA_E2B_CONFIG`/`GEMMA_E4B_CONFIG` constants (GGUF download URLs + filenames on a private R2 bucket), and the persisted `AppConfig` (model, shell, show_hints, auto_copy) with `load_config`/`save_config` against `~/.cache/sage/config.json`. Resolves `~/.cache/sage/` as the cache dir.
- `src/setup.rs` — the interactive `sage --config` wizard (dialoguer): detects RAM via `sysctl`, recommends E2B (<16 GB) or E4B (≥16 GB), confirms shell against `$SHELL` with a pick-list, and asks about hints + clipboard, then saves the config. Also triggers model download and removes the old model.
- `src/downloader.rs` — `ensure_model_exists` streams the model download with a progress bar; verifies the local file size against the server's content-length (HEAD) so interrupted downloads are re-downloaded. `remove_other_models` deletes whichever of E2B/E4B isn't in use.
- `src/engine.rs` — `InferenceEngine` wraps `LlamaBackend` + `LlamaModel`. `new()` loads the model; `generate()` decodes the prompt into a batch, then loops sampling until `max_tokens` or an end-of-generation token. A persistent `LlamaSampler::penalties` (repeat 1.1, last 64, freq/present 0) is seeded with the prompt tokens and fed each generated token via `accept` to suppress the small model's repetition loops before the greedy pick.
- `src/prompt.rs` — Gemma-format (`<start_of_turn>...`) instruction prompts. `format_prompt` embeds the current OS and the *configured* shell and asks for a raw shell command (contract: "raw shell command on line 1, `# `-prefixed explanation on later lines"); `format_explain_prompt` asks for a plain-language breakdown of a given command; `format_answer_prompt` asks for a general-knowledge answer to a question.

## Key dependencies

- `llama-cpp-2` 0.1 with only the `metal` feature (default features disabled) — macOS Apple Silicon only.
- `clap` (derive), `anyhow` (errors), `dirs` (home dir resolution), `futures-util` (download streaming), `tokio`/`reqwest` (async download), `indicatif` (progress bar), `encoding_rs` (UTF-8 decoder for `token_to_piece`), `arboard` (system clipboard), `serde`/`serde_json` (config persistence), `dialoguer` (interactive `--config` wizard).
