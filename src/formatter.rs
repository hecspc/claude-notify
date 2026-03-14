use crate::types::HookEvent;

const MAX_MESSAGE_LEN: usize = 4096;

pub fn format_message(event: &HookEvent) -> String {
    let session_name = friendly_name(&event.session_id);
    let session_short = if event.session_id.len() > 8 {
        &event.session_id[..8]
    } else {
        &event.session_id
    };

    let project = event
        .cwd
        .as_deref()
        .and_then(|p| p.rsplit('/').next())
        .unwrap_or("unknown");

    let session_line = format!("Session: {} ({}) | {}", session_name, session_short, project);

    let body = match event.hook_event_name.as_str() {
        "Notification" => format_notification(event),
        "Stop" => format_stop(event),
        "TaskCompleted" => format_task_completed(event),
        other => FormattedBody {
            header: format!("\u{2139}\u{fe0f} Event: {}", html_escape(other)),
            detail: String::new(),
        },
    };

    let msg = format!("{}\n{}\n{}", body.header, session_line, body.detail);
    truncate(&msg, MAX_MESSAGE_LEN)
}

struct FormattedBody {
    header: String,
    detail: String,
}

fn format_notification(event: &HookEvent) -> FormattedBody {
    let notification_type = event.notification_type.as_deref().unwrap_or("unknown");

    match notification_type {
        "permission_prompt" => {
            let tool = event.tool_name.as_deref().unwrap_or("unknown");
            let action = extract_action(event);
            FormattedBody {
                header: "\u{1f514} <b>Permission Required</b>".to_string(),
                detail: format!(
                    "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\nTool: {}\nAction: {}",
                    html_escape(tool),
                    html_escape(&action)
                ),
            }
        }
        "idle_prompt" => FormattedBody {
            header: "\u{23f3} <b>Waiting for Input</b>".to_string(),
            detail: "Claude is idle and waiting for your response.".to_string(),
        },
        "elicitation_dialog" => {
            let msg = event.message.as_deref().unwrap_or("Claude has a question");
            FormattedBody {
                header: "\u{2753} <b>Question</b>".to_string(),
                detail: html_escape(msg),
            }
        }
        other => FormattedBody {
            header: format!("\u{1f514} <b>Notification: {}</b>", html_escape(other)),
            detail: String::new(),
        },
    }
}

fn format_stop(event: &HookEvent) -> FormattedBody {
    let detail = match &event.last_assistant_message {
        Some(msg) if !msg.is_empty() => {
            let summary = truncate_lines(msg, 500);
            format!(
                "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n{}",
                html_escape(&summary)
            )
        }
        _ => "Claude has finished responding.".to_string(),
    };

    FormattedBody {
        header: "\u{2705} <b>Response Complete</b>".to_string(),
        detail,
    }
}

fn format_task_completed(event: &HookEvent) -> FormattedBody {
    let mut lines = Vec::new();

    if let Some(subject) = &event.task_subject {
        lines.push(format!("Task: {}", html_escape(subject)));
    }
    if let Some(teammate) = &event.teammate_name {
        lines.push(format!("Teammate: {}", html_escape(teammate)));
    }
    if matches!(&event.task_description, Some(desc) if !desc.is_empty()) {
        let desc = event.task_description.as_ref().unwrap();
        let short = truncate_lines(desc, 300);
        lines.push(format!(
            "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n{}",
            html_escape(&short)
        ));
    }

    if lines.is_empty() {
        lines.push("A background task has finished.".to_string());
    }

    FormattedBody {
        header: "\u{1f389} <b>Task Completed</b>".to_string(),
        detail: lines.join("\n"),
    }
}

/// Truncate to max_chars, cutting at the last newline or word boundary.
fn truncate_lines(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        return s.to_string();
    }
    let truncated = &s[..max_chars];
    // Try to cut at last newline
    if let Some(pos) = truncated.rfind('\n') {
        format!("{}...", &s[..pos])
    } else {
        format!("{}...", truncated.trim_end())
    }
}

fn extract_action(event: &HookEvent) -> String {
    let Some(input) = &event.tool_input else {
        return "—".to_string();
    };

    // For Bash tool, show the command
    if let Some(cmd) = input.get("command").and_then(|v| v.as_str()) {
        return cmd.to_string();
    }

    // For Edit/Write tools, show the file path
    if let Some(path) = input.get("file_path").and_then(|v| v.as_str()) {
        return path.to_string();
    }

    // For other tools, show a compact JSON summary
    let s = input.to_string();
    if s.len() > 200 {
        format!("{}...", &s[..200])
    } else {
        s
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

/// Maps a session_id to a deterministic human-friendly "adjective-noun" name.
pub fn friendly_name(session_id: &str) -> String {
    const ADJECTIVES: &[&str] = &[
        "bold", "calm", "cool", "dark", "deep", "dry", "fair", "fast",
        "fine", "free", "glad", "gold", "good", "gray", "keen", "kind",
        "late", "lean", "live", "long", "loud", "mild", "neat", "nice",
        "pale", "pure", "rare", "raw", "red", "rich", "safe", "slim",
        "slow", "soft", "tall", "tame", "thin", "true", "vast", "warm",
        "weak", "wide", "wild", "wise", "blue", "cold", "dull", "flat",
        "full", "grim", "high", "iron", "jade", "lazy", "mint", "opal",
        "pink", "plum", "ruby", "rust", "sage", "sand", "silk", "snow",
    ];

    const NOUNS: &[&str] = &[
        "ant", "ape", "bat", "bee", "bird", "boar", "bull", "cat",
        "colt", "crab", "crow", "deer", "dog", "dove", "duck", "elk",
        "fawn", "fish", "frog", "goat", "hare", "hawk", "ibis", "jay",
        "kite", "lark", "lion", "lynx", "mole", "moth", "newt", "oryx",
        "owl", "puma", "ram", "seal", "slug", "swan", "toad", "vole",
        "wasp", "wolf", "wren", "yak", "bear", "carp", "dodo", "ewe",
        "fox", "gnu", "hen", "imp", "koi", "lamb", "mink", "ox",
        "pug", "quail", "ray", "shrew", "tern", "urchin", "viper", "worm",
    ];

    // Simple hash: sum of bytes
    let hash: usize = session_id.bytes().map(|b| b as usize).sum();
    let adj = ADJECTIVES[hash % ADJECTIVES.len()];
    let noun = NOUNS[(hash / ADJECTIVES.len()) % NOUNS.len()];
    format!("{}-{}", adj, noun)
}
