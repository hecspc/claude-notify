# claude-notify — Implementation

## Build Status

Binary compiles to a 3.6 MB release binary. Installed to `~/.local/bin/claude-notify`.

## Project Structure

```
Cargo.toml
src/
  main.rs              # CLI entry point: arg parsing, stdin → format → dispatch
  types.rs             # HookEvent struct (serde deserialization)
  config.rs            # Config loading: TOML file + env var overrides
  formatter.rs         # Event → human-readable HTML message
  notifier.rs          # Notifier trait definition
  notifiers/
    mod.rs             # Backend registry: config → Vec<Box<dyn Notifier>>
    telegram.rs        # Telegram Bot API implementation (ureq)
  setup.rs             # --setup: auto-configure hooks in ~/.claude/settings.json
```

## Dependencies

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ureq = { version = "3", features = ["json"] }
toml = "0.8"
clap = { version = "4", features = ["derive"] }
```

- **serde + serde_json** — deserialize hook JSON payloads, serialize Telegram API requests
- **ureq** — lightweight blocking HTTP client, no async runtime needed
- **toml** — parse `~/.config/claude-notify/config.toml`
- **clap** — CLI argument parsing with derive macros

## Source Files

### `src/types.rs`

```rust
use serde::Deserialize;

/// Represents the JSON payload Claude Code sends to hooks via stdin.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct HookEvent {
    pub session_id: String,
    pub cwd: Option<String>,
    pub hook_event_name: String,
    /// Present for Notification events
    pub notification_type: Option<String>,
    /// Present for permission_prompt notifications
    pub tool_name: Option<String>,
    /// Present for permission_prompt notifications
    pub tool_input: Option<serde_json::Value>,
    /// Present for Stop / TaskCompleted events
    pub stop_hook_active: Option<bool>,
    /// Present for elicitation_dialog notifications
    pub message: Option<String>,
}
```

All fields except `session_id` and `hook_event_name` are `Option<T>` because different event types carry different payloads. Uses `serde_json::Value` for `tool_input` since its shape varies by tool.

### `src/config.rs`

```rust
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub backends: Vec<String>,
    pub events: Option<Vec<String>>,
    #[serde(default)]
    pub telegram: Option<TelegramConfig>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TelegramConfig {
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        let mut config = Self::load_from_file().unwrap_or_default();
        config.apply_env_overrides();

        if config.backends.is_empty() {
            config.backends = vec!["telegram".to_string()];
        }

        config
    }

    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("claude-notify")
            .join("config.toml")
    }

    fn load_from_file() -> Option<Self> {
        let path = Self::config_path();
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("NOTIFY_BACKEND") {
            self.backends = val.split(',').map(|s| s.trim().to_string()).collect();
        }

        if let Ok(val) = std::env::var("NOTIFY_EVENTS") {
            self.events = Some(val.split(',').map(|s| s.trim().to_string()).collect());
        }

        let tg = self.telegram.get_or_insert_with(TelegramConfig::default);
        if let Ok(val) = std::env::var("TELEGRAM_BOT_TOKEN") {
            tg.bot_token = Some(val);
        }
        if let Ok(val) = std::env::var("TELEGRAM_CHAT_ID") {
            tg.chat_id = Some(val);
        }
    }

    /// Returns true if the given event name should trigger a notification.
    pub fn should_notify(&self, event: &str) -> bool {
        match &self.events {
            None => true,
            Some(events) => events.iter().any(|e| e == event),
        }
    }
}
```

Loading order: TOML file → env var overrides. If no backends specified, defaults to `["telegram"]`. Event filtering uses the `events` list — `None` means all events pass through.

### `src/notifier.rs`

```rust
pub trait Notifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn name(&self) -> &str;
}
```

Trait object interface for pluggable backends. `name()` is used in error messages.

### `src/notifiers/telegram.rs`

```rust
use crate::config::TelegramConfig;
use crate::notifier::Notifier;

pub struct TelegramNotifier {
    bot_token: String,
    chat_id: String,
}

impl TelegramNotifier {
    pub fn new(config: &TelegramConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let bot_token = config
            .bot_token
            .clone()
            .ok_or("telegram bot_token not configured")?;
        let chat_id = config
            .chat_id
            .clone()
            .ok_or("telegram chat_id not configured")?;
        Ok(Self { bot_token, chat_id })
    }
}

impl Notifier for TelegramNotifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );

        let body = serde_json::json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": "HTML",
            "disable_web_page_preview": true,
        });

        let response = ureq::post(&url).send_json(&body)?;

        if response.status() != 200 {
            let status = response.status();
            let body = response.into_body().read_to_string()?;
            return Err(format!("Telegram API error {}: {}", status, body).into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "telegram"
    }
}
```

Uses HTML parse mode (only need to escape `< > &`). `disable_web_page_preview` prevents link previews cluttering the notification.

### `src/notifiers/mod.rs`

```rust
pub mod telegram;

use crate::config::Config;
use crate::notifier::Notifier;

pub fn build_notifiers(config: &Config) -> Vec<Box<dyn Notifier>> {
    let mut notifiers: Vec<Box<dyn Notifier>> = Vec::new();

    for backend in &config.backends {
        match backend.as_str() {
            "telegram" => {
                if let Some(tg_config) = &config.telegram {
                    match telegram::TelegramNotifier::new(tg_config) {
                        Ok(n) => notifiers.push(Box::new(n)),
                        Err(e) => eprintln!("Warning: failed to init telegram: {}", e),
                    }
                } else {
                    eprintln!("Warning: telegram backend enabled but not configured");
                }
            }
            other => {
                eprintln!("Warning: unknown backend '{}'", other);
            }
        }
    }

    notifiers
}
```

Registry pattern: reads `config.backends` and constructs the corresponding `Notifier` implementations. Adding a new backend means adding a match arm and a new module.

### `src/formatter.rs`

```rust
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
            header: format!("ℹ️ Event: {}", html_escape(other)),
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
                header: "🔔 <b>Permission Required</b>".to_string(),
                detail: format!(
                    "─────────────────\nTool: {}\nAction: {}",
                    html_escape(tool),
                    html_escape(&action)
                ),
            }
        }
        "idle_prompt" => FormattedBody {
            header: "⏳ <b>Waiting for Input</b>".to_string(),
            detail: "Claude is idle and waiting for your response.".to_string(),
        },
        "elicitation_dialog" => {
            let msg = event.message.as_deref().unwrap_or("Claude has a question");
            FormattedBody {
                header: "❓ <b>Question</b>".to_string(),
                detail: html_escape(msg),
            }
        }
        other => FormattedBody {
            header: format!("🔔 <b>Notification: {}</b>", html_escape(other)),
            detail: String::new(),
        },
    }
}

fn format_stop() -> FormattedBody {
    FormattedBody {
        header: "✅ <b>Response Complete</b>".to_string(),
        detail: "Claude has finished responding.".to_string(),
    }
}

fn format_task_completed() -> FormattedBody {
    FormattedBody {
        header: "🎉 <b>Task Completed</b>".to_string(),
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
```

Message format: `header + session_line + detail`. Each event type maps to an icon and label. `extract_action` smart-extracts the most useful field from `tool_input` (command for Bash, file_path for Edit/Write, truncated JSON for everything else). Truncated to 4096 chars (Telegram message limit).

### `src/setup.rs`

```rust
use std::path::PathBuf;

fn settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".claude").join("settings.json")
}

pub fn run_setup() -> Result<(), Box<dyn std::error::Error>> {
    let path = settings_path();

    let mut settings: serde_json::Value = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    let obj = settings
        .as_object_mut()
        .ok_or("settings.json is not an object")?;

    if obj.contains_key("hooks") {
        let hooks = obj.get("hooks").unwrap();
        let has_notify = hooks.to_string().contains("claude-notify");
        if has_notify {
            println!("claude-notify hooks are already configured in {}", path.display());
            println!("Remove the existing hooks first if you want to reconfigure.");
            return Ok(());
        }
    }

    let hooks = obj
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let hooks_obj = hooks
        .as_object_mut()
        .ok_or("hooks is not an object")?;

    // Notification hook with matcher
    hooks_obj.insert(
        "Notification".to_string(),
        serde_json::json!([{
            "matcher": "permission_prompt|idle_prompt|elicitation_dialog",
            "hooks": [{ "type": "command", "command": "claude-notify", "async": true }]
        }]),
    );

    // Stop hook
    hooks_obj.insert(
        "Stop".to_string(),
        serde_json::json!([{
            "hooks": [{ "type": "command", "command": "claude-notify", "async": true }]
        }]),
    );

    // TaskCompleted hook
    hooks_obj.insert(
        "TaskCompleted".to_string(),
        serde_json::json!([{
            "hooks": [{ "type": "command", "command": "claude-notify", "async": true }]
        }]),
    );

    // Write back with pretty formatting
    let content = serde_json::to_string_pretty(&settings)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)?;

    println!("Hooks configured in {}", path.display());
    println!("\nMake sure claude-notify is in your PATH (e.g. ~/.local/bin/)");
    println!("and that ~/.config/claude-notify/config.toml has your Telegram credentials.");

    Ok(())
}
```

Reads existing `settings.json` (or starts from `{}`), merges hook config, writes back. Detects if hooks are already configured to avoid duplicates. All hooks use `"async": true` so they never block Claude Code.

### `src/main.rs`

```rust
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
```

Flow: parse CLI → route to `--setup` or stdin mode → read JSON → check event filter → format → send via all active backends. Errors go to stderr (invisible to Claude Code since hooks are async).

## Message Output Examples

**Permission prompt:**
```
🔔 Permission Required
Session: a3f2b1c9 | engineering-bot
─────────────────
Tool: Bash
Action: npm install express
```

**Stop (response complete):**
```
✅ Response Complete
Session: abc123 | test
Claude has finished responding.
```

**Task completed:**
```
🎉 Task Completed
Session: abc123 | test
A background task has finished.
```

**Idle prompt:**
```
⏳ Waiting for Input
Session: abc123 | test
Claude is idle and waiting for your response.
```

**Elicitation dialog:**
```
❓ Question
Session: abc123 | test
Which database should I use?
```

## Data Flow

```
Claude Code Event
  → Hook (async: true, never blocks)
    → stdin: JSON payload
      → claude-notify binary
        → deserialize HookEvent (types.rs)
        → load Config (config.rs) — TOML + env vars
        → check event filter (config.should_notify)
        → format message (formatter.rs)
        → dispatch to backends (notifiers/mod.rs)
          → TelegramNotifier.send() → Telegram Bot API
```

## Configuration

Config file: `~/.config/claude-notify/config.toml`

```toml
backends = ["telegram"]

# Optional: filter which events trigger notifications
# events = ["permission_prompt", "idle_prompt", "elicitation_dialog", "stop", "task_completed"]

[telegram]
bot_token = "123456:ABC-DEF..."
chat_id = "123456789"
```

Environment variables override config file values:

| Variable | Overrides |
|---|---|
| `NOTIFY_BACKEND` | `backends` |
| `NOTIFY_EVENTS` | `events` |
| `TELEGRAM_BOT_TOKEN` | `[telegram].bot_token` |
| `TELEGRAM_CHAT_ID` | `[telegram].chat_id` |

## Installation

```bash
cargo build --release
cp target/release/claude-notify ~/.local/bin/
claude-notify --setup
```

## Adding a New Backend

1. Create `src/notifiers/newbackend.rs` implementing `Notifier` trait
2. Add config struct fields to `Config` in `config.rs`
3. Add match arm in `notifiers/mod.rs` `build_notifiers()`
4. Add env var overrides in `config.rs` `apply_env_overrides()`
