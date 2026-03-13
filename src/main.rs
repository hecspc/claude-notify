mod config;
mod formatter;
mod notifier;
mod notifiers;
mod setup;
mod types;

use clap::{Parser, Subcommand};
use std::io::Read;

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
}

#[derive(Subcommand)]
enum SetupBackend {
    /// Configure Telegram notifications
    Telegram {
        /// Bot token from @BotFather
        bot_token: String,
        /// Chat ID from @userinfobot
        chat_id: String,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Some(Command::Setup { backend, user, project }) = cli.command {
        let scope = if project {
            setup::Scope::Project
        } else if user {
            setup::Scope::User
        } else {
            setup::Scope::User
        };

        if let Err(e) = setup::run_setup(&backend, scope) {
            eprintln!("Setup failed: {}", e);
            std::process::exit(1);
        }
        return;
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
