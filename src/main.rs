mod config;
mod formatter;
mod notifier;
mod notifiers;
mod setup;
mod types;

use clap::Parser;
use std::io::Read;

#[derive(Parser)]
#[command(name = "claude-notify", version, about = "Notification bot for Claude Code hook events")]
struct Cli {
    /// Auto-configure hooks in ~/.claude/settings.json
    #[arg(long)]
    setup: bool,

    /// Print formatted message to stdout without sending
    #[arg(long)]
    dry_run: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.setup {
        if let Err(e) = setup::run_setup() {
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
        eprintln!("No notification backends configured. Run 'claude-notify --setup' or set environment variables.");
        std::process::exit(1);
    }

    for n in &notifiers {
        if let Err(e) = n.send(&message) {
            eprintln!("Failed to send via {}: {}", n.name(), e);
        }
    }
}
