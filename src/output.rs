pub fn parse_command_output(sanitized: &str) -> (String, Vec<&str>) {
    let lines: Vec<&str> = sanitized
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let command = lines
        .iter()
        .find(|line| !line.starts_with('#'))
        .copied()
        .unwrap_or_default()
        .to_string();
    let notes: Vec<&str> = lines
        .iter()
        .filter(|line| line.starts_with('#'))
        .copied()
        .collect();
    (command, notes)
}

pub fn sanitize_output(raw: &str) -> String {
    let mut text = raw.to_string();
    for tok in [
        "<start_of_turn>",
        "<end_of_turn>",
        "</start_of_turn>",
        "</end_of_turn>",
        "<|start_of_turn|>",
        "<|end_of_turn|>",
        "</|start_of_turn|>",
        "</|end_of_turn|>",
    ] {
        text = text.replace(tok, "\n");
    }
    text.lines()
        .map(str::trim)
        .filter(|line| !matches!(*line, "user" | "model" | "assistant" | "system"))
        .collect::<Vec<_>>()
        .join("\n")
}
