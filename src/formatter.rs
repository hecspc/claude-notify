use crate::types::HookEvent;

const MAX_MESSAGE_LEN: usize = 4096;

pub fn format_message(event: &HookEvent) -> String {
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

    let session_line = format!("Session: {} | {}", session_short, project);

    let body = match event.hook_event_name.as_str() {
        "Notification" => format_notification(event),
        "Stop" => format_stop(),
        "TaskCompleted" => format_task_completed(),
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

fn format_stop() -> FormattedBody {
    FormattedBody {
        header: "\u{2705} <b>Response Complete</b>".to_string(),
        detail: "Claude has finished responding.".to_string(),
    }
}

fn format_task_completed() -> FormattedBody {
    FormattedBody {
        header: "\u{1f389} <b>Task Completed</b>".to_string(),
        detail: "A background task has finished.".to_string(),
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
