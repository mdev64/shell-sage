/// Gemma turn markers the model sometimes echoes back.
const TURN_TOKENS: [&str; 8] = [
    "<start_of_turn>",
    "<end_of_turn>",
    "</start_of_turn>",
    "</end_of_turn>",
    "<|start_of_turn|>",
    "<|end_of_turn|>",
    "</|start_of_turn|>",
    "</|end_of_turn|>",
];

/// Bare role labels left behind once the turn markers are stripped.
const ROLE_MARKERS: [&str; 4] = ["user", "model", "assistant", "system"];

/// Splits model output into the command (first non-`#` line) and its
/// `#`-prefixed notes. Blank lines are ignored.
pub fn parse_command_output(sanitized: &str) -> (String, Vec<&str>) {
    let (notes, rest): (Vec<&str>, Vec<&str>) = sanitized
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .partition(|line| line.starts_with('#'));
    let command = rest.first().copied().unwrap_or_default().to_string();
    (command, notes)
}

/// Removes echoed Gemma turn tokens and role labels, trimming every line.
pub fn sanitize_output(raw: &str) -> String {
    let text = TURN_TOKENS
        .iter()
        .fold(raw.to_string(), |text, tok| text.replace(tok, "\n"));
    text.lines()
        .map(str::trim)
        .filter(|line| !ROLE_MARKERS.contains(line))
        .collect::<Vec<_>>()
        .join("\n")
}
