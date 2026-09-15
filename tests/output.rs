use sage::output::{parse_command_output, sanitize_output};

#[test]
fn parse_command_with_notes() {
    let (cmd, notes) = parse_command_output("ls -la\n# -l: long format\n# -a: show all");
    assert_eq!(cmd, "ls -la");
    assert_eq!(notes, vec!["# -l: long format", "# -a: show all"]);
}

#[test]
fn parse_command_without_notes() {
    let (cmd, notes) = parse_command_output("rm -rf foo");
    assert_eq!(cmd, "rm -rf foo");
    assert!(notes.is_empty());
}

#[test]
fn parse_command_skips_leading_notes() {
    let (cmd, notes) = parse_command_output("# first a note\nls -la\n# trailing");
    assert_eq!(cmd, "ls -la");
    assert_eq!(notes, vec!["# first a note", "# trailing"]);
}

#[test]
fn parse_command_ignores_blank_lines() {
    let (cmd, _) = parse_command_output("  \n\n\t\nls -la\n\n");
    assert_eq!(cmd, "ls -la");
}

#[test]
fn parse_command_empty_input() {
    let (cmd, notes) = parse_command_output("   \n\n");
    assert_eq!(cmd, "");
    assert!(notes.is_empty());
}

#[test]
fn sanitize_strips_turn_tokens() {
    let out = sanitize_output("<start_of_turn>model\nls -la\n<end_of_turn>");
    assert_eq!(out.trim(), "ls -la");
}

#[test]
fn sanitize_strips_pipe_turn_tokens() {
    let out = sanitize_output("<|start_of_turn|>model\nls -la\n<|end_of_turn|>");
    assert_eq!(out.trim(), "ls -la");
}

#[test]
fn sanitize_removes_role_markers() {
    let out = sanitize_output("<start_of_turn>user\n<start_of_turn>model\nfind . -name '*.mp4'");
    assert!(!out.contains("user"));
    assert!(!out.contains("model"));
    assert!(out.contains("find . -name '*.mp4'"));
}

#[test]
fn sanitize_preserves_real_content() {
    let out = sanitize_output("find . -name '*.mp4'\n# matches mp4 files");
    assert!(out.contains("find . -name '*.mp4'"));
    assert!(out.contains("# matches mp4 files"));
}
