mod config;
mod formatter;
mod notifier;
mod notifiers;
mod setup;
mod types;

use clap::{Parser, Subcommand};
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "claude-notify", version, about = "Notification bot for Claude Code hook events")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Print formatted message to stdout without sending
    #[arg(long)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Configure hooks and notification backend
    Setup {
        #[command(subcommand)]
        backend: SetupBackend,

        /// Install hooks in ~/.claude/settings.json (default)
        #[arg(long, group = "scope")]
        user: bool,

        /// Install hooks in .claude/settings.json in current directory
        #[arg(long, group = "scope")]
        project: bool,
    },
    /// Mute notifications (globally or for a session)
    Mute {
        /// Session ID or friendly name to mute (omit to mute all)
        session: Option<String>,
    },
    /// Unmute notifications (globally or for a session)
    Unmute {
        /// Session ID or friendly name to unmute (omit to unmute all)
        session: Option<String>,
    },
    /// Show mute status
    Status,
    /// Switch active notification backend(s)
    Use {
        /// Backend name(s), comma-separated (e.g. "desktop", "slack,discord")
        backends: String,
    },
}

#[derive(Subcommand)]
pub enum SetupBackend {
    /// Configure Telegram notifications
    Telegram {
        /// Bot token from @BotFather
        bot_token: String,
        /// Chat ID from @userinfobot
        chat_id: String,
    },
    /// Configure Slack notifications via Incoming Webhook
    Slack {
        /// Webhook URL from Slack app configuration
        webhook_url: String,
    },
    /// Configure desktop notifications (zero-config, uses native OS)
    Desktop,
    /// Configure Discord notifications via webhook
    Discord {
        /// Webhook URL from Discord channel settings
        webhook_url: String,
    },
    /// Configure ntfy notifications
    Ntfy {
        /// Topic URL (e.g. https://ntfy.sh/my-topic)
        topic_url: String,
    },
}

fn mute_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("claude-notify")
        .join("muted")
}

fn is_muted(session_id: &str) -> bool {
    let dir = mute_dir();
    if dir.join("_global").exists() {
        return true;
    }
    // Check by exact session_id
    if dir.join(session_id).exists() {
        return true;
    }
    // Check by friendly name
    let friendly = formatter::friendly_name(session_id);
    if dir.join(&friendly).exists() {
        return true;
    }
    false
}

fn cmd_mute(session: Option<String>) {
    let dir = mute_dir();
    std::fs::create_dir_all(&dir).ok();

    let name = session.as_deref().unwrap_or("_global");
    let path = dir.join(name);
    std::fs::write(&path, "").ok();

    if name == "_global" {
        println!("All notifications muted.");
    } else {
        println!("Session '{}' muted.", name);
    }
}

fn cmd_unmute(session: Option<String>) {
    let dir = mute_dir();

    match session.as_deref() {
        Some(name) => {
            let path = dir.join(name);
            if path.exists() {
                std::fs::remove_file(&path).ok();
                println!("Session '{}' unmuted.", name);
            } else {
                println!("Session '{}' was not muted.", name);
            }
        }
        None => {
            // Remove everything in the muted dir
            if dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        std::fs::remove_file(entry.path()).ok();
                    }
                }
            }
            println!("All notifications unmuted.");
        }
    }
}

fn cmd_status() {
    let dir = mute_dir();

    if dir.join("_global").exists() {
        println!("Notifications: MUTED (all)");
        return;
    }

    let mut muted_sessions = Vec::new();
    if dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    muted_sessions.push(name.to_string());
                }
            }
        }
    }

    if muted_sessions.is_empty() {
        println!("Notifications: active");
    } else {
        println!("Notifications: active (except muted sessions)");
        for s in &muted_sessions {
            println!("  - {}", s);
        }
    }
}

fn cmd_use(backends: &str) {
    let path = config::Config::config_path();

    let mut config: toml::Table = if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        content.parse().unwrap_or_else(|_| toml::Table::new())
    } else {
        toml::Table::new()
    };

    let names: Vec<String> = backends
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let arr = names
        .iter()
        .map(|s| toml::Value::String(s.clone()))
        .collect();
    config.insert("backends".to_string(), toml::Value::Array(arr));

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if let Err(e) = std::fs::write(&path, config.to_string()) {
        eprintln!("Failed to write config: {}", e);
        std::process::exit(1);
    }

    println!("Active backends: {}", names.join(", "));
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Setup { backend, user: _, project }) => {
            let scope = if project {
                setup::Scope::Project
            } else {
                setup::Scope::User
            };
            if let Err(e) = setup::run_setup(&backend, scope) {
                eprintln!("Setup failed: {}", e);
                std::process::exit(1);
            }
            return;
        }
        Some(Command::Mute { session }) => {
            cmd_mute(session);
            return;
        }
        Some(Command::Unmute { session }) => {
            cmd_unmute(session);
            return;
        }
        Some(Command::Status) => {
            cmd_status();
            return;
        }
        Some(Command::Use { backends }) => {
            cmd_use(&backends);
            return;
        }
        None => {}
    }

    // Read hook event JSON from stdin
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("Failed to read stdin: {}", e);
        std::process::exit(1);
    }

    let event: types::HookEvent = match serde_json::from_str(&input) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to parse hook event: {}", e);
            std::process::exit(1);
        }
    };

    // Check mute status before anything else
    if is_muted(&event.session_id) {
        return;
    }

    let config = config::Config::load();

    // Determine the event key for filtering
    let event_key = match event.hook_event_name.as_str() {
        "Notification" => event
            .notification_type
            .as_deref()
            .unwrap_or("unknown")
            .to_string(),
        "Stop" => "stop".to_string(),
        "TaskCompleted" => "task_completed".to_string(),
        other => other.to_lowercase(),
    };

    if !config.should_notify(&event_key) {
        return;
    }

    let message = formatter::format_message(&event);

    if cli.dry_run {
        println!("{}", message);
        return;
    }

    let notifiers = notifiers::build_notifiers(&config);

    if notifiers.is_empty() {
        eprintln!("No notification backends configured. Run 'claude-notify setup' or set environment variables.");
        std::process::exit(1);
    }

    for n in &notifiers {
        if let Err(e) = n.send(&message) {
            eprintln!("Failed to send via {}: {}", n.name(), e);
        }
    }
}
