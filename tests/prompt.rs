use sage::prompt::{format_answer_prompt, format_explain_prompt, format_prompt};
use std::env;

#[test]
fn command_prompt_embeds_query_shell_and_os() {
    let prompt = format_prompt("find all mp4 files", "/bin/zsh");
    assert!(prompt.contains("find all mp4 files"));
    assert!(prompt.contains("/bin/zsh"));
    assert!(prompt.contains(env::consts::OS));
    assert!(prompt.contains("<start_of_turn>user"));
    assert!(prompt.ends_with("<start_of_turn>model\n"));
}

#[test]
fn explain_prompt_embeds_command_and_shell() {
    let prompt = format_explain_prompt("ls -la", "/bin/bash");
    assert!(prompt.contains("ls -la"));
    assert!(prompt.contains("/bin/bash"));
    assert!(prompt.contains(env::consts::OS));
}

#[test]
fn answer_prompt_embeds_question() {
    let prompt = format_answer_prompt("why is the sky blue?");
    assert!(prompt.contains("why is the sky blue?"));
    assert!(prompt.contains("<start_of_turn>user"));
}
