use std::env;

pub fn format_prompt(query: &str, shell: &str) -> String {
    let os = env::consts::OS;

    format!(
        "<start_of_turn>user\n\
You are an expert CLI assistant. Generate ONLY the executable shell command matching the user's intent on {os} using {shell}.\n\
Rules:\n\
1. Output raw shell syntax without markdown formatting or code blocks.\n\
2. Explain important flags on subsequent lines starting with '# '.\n\n\
Intent: {query}<end_of_turn>\n\
<start_of_turn>model\n"
    )
}

pub fn format_explain_prompt(command: &str, shell: &str) -> String {
    let os = env::consts::OS;
    format!(
        "<start_of_turn>user\n\
You are a helpful CLI tutor. Explain the following terminal command and what it does on {os} using {shell}.\n\
Rules:\n\
1. Start with a one-line summary of what the command does.\n\
2. Then explain each flag, option, and argument in plain language.\n\
3. If useful, give a short example of when you would use it.\n\
4. Use plain text; do not use markdown code blocks.\n\n\
Command: {command}<end_of_turn>\n\
<start_of_turn>model\n"
    )
}

pub fn format_answer_prompt(question: &str) -> String {
    format!(
        "<start_of_turn>user\n\
You are Sage, a helpful assistant. Answer the user's question using your general knowledge.\n\
Rules:\n\
1. Answer clearly, accurately, and concisely.\n\
2. If you are not sure, say so instead of guessing.\n\
3. Use plain text; do not use markdown code blocks or shell commands.\n\n\
Question: {question}<end_of_turn>\n\
<start_of_turn>model\n"
    )
}
