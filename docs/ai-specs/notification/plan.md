# claude-notify — Claude Code Notification Bot (Rust)

## Context

When running Claude Code sessions (especially long-running or parallel ones), the user needs to know when Claude requires input — permissions, questions, or task completion. Without notifications, sessions sit idle waiting for attention. `claude-notify` sends notifications to Telegram (and future backends) so the user can monitor from mobile. Built in Rust for a single distributable binary.

## Claude Code Events

| Hook Event | Matcher | What it means |
|---|---|---|
| `Notification` | `permission_prompt\|idle_prompt\|elicitation_dialog` | Claude needs user input |
| `Stop` | — | Claude finished responding |
| `TaskCompleted` | — | A task completed |

All hooks run with `async: true` so they never block Claude Code.

## Technology Stack

- **Rust** — compiles to a single native binary
- **`ureq`** — lightweight, blocking HTTP client (no async runtime needed)
- **`serde` + `serde_json`** — JSON deserialization of hook payloads
- **`toml`** — config file parsing
- **`clap`** — CLI argument parsing (for `--setup`, `--dry-run`, etc.)

## Architecture

```
Claude Code Event → Hook (async) → claude-notify (binary) → Notifier trait → Telegram / Slack / WhatsApp / ...
```

The hook invokes a native binary directly — no runtime needed on the target machine. The notification backend is abstracted behind a `Notifier` trait so new channels can be added without changing the core logic.

```rust
trait Notifier {
    fn send(&self, message: &str) -> Result<()>;
    fn name(&self) -> &str;
}
```

### Configuration

Config file at `~/.config/claude-notify/config.toml`:

```toml
# Which backends to use (comma-separated also works via NOTIFY_BACKEND env var)
backends = ["telegram"]

# Which events to notify on (defaults to all if omitted)
# events = ["permission_prompt", "idle_prompt", "elicitation_dialog", "stop", "task_completed"]

[telegram]
bot_token = "123456:ABC-DEF..."
chat_id = "123456789"

# Future backends:
# [slack]
# webhook_url = "https://hooks.slack.com/services/..."
#
# [whatsapp]
# api_token = "..."
# phone_number = "..."
```

Env vars override config file values (e.g. `TELEGRAM_BOT_TOKEN` overrides `[telegram].bot_token`).

### File Structure

```
Cargo.toml
src/
  main.rs              # Entry point: CLI parsing, read stdin → format → dispatch
  types.rs             # Structs for hook event payloads (serde)
  config.rs            # Config file + env var loading
  formatter.rs         # Maps events to human-readable messages
  notifier.rs          # Notifier trait definition
  notifiers/
    mod.rs             # Registry: returns active backends based on config
    telegram.rs        # Telegram Bot API via ureq
  setup.rs             # --setup: writes hook config into ~/.claude/settings.json
```

Future backends (not built now, just drop in):
```
  notifiers/
    slack.rs           # Slack webhook
    whatsapp.rs        # WhatsApp Business API / Twilio
```

### CLI Interface

```
claude-notify              # Normal mode: read hook JSON from stdin, notify
claude-notify --setup      # Auto-configure hooks in ~/.claude/settings.json
claude-notify --dry-run    # Print formatted message to stdout, don't send
claude-notify --version    # Print version
```

### Message Format

```
🔔 Permission Required
Session: a3f2b1c9 | engineering-bot
─────────────────
Tool: Bash
Action: npm install express
```

Each event type gets an icon and label. Session shows truncated `session_id` + `basename(cwd)` for quick identification across parallel sessions.

### Message Filtering

The `events` config key (or `NOTIFY_EVENTS` env var) controls which events trigger notifications. Defaults to all events. Users who find "Response Complete" noisy can exclude `stop`:

```toml
events = ["permission_prompt", "idle_prompt", "elicitation_dialog", "task_completed"]
```

## Environment Variables

All env vars are optional if config file is present. Env vars override config file values.

| Variable | Purpose | Example |
|---|---|---|
| `NOTIFY_BACKEND` | Active backend(s), comma-separated | `telegram` (default) |
| `NOTIFY_EVENTS` | Event filter, comma-separated | `permission_prompt,idle_prompt` |
| `TELEGRAM_BOT_TOKEN` | Token from @BotFather | `123456:ABC-DEF...` |
| `TELEGRAM_CHAT_ID` | User's chat ID (from @userinfobot) | `123456789` |

## Hook Configuration (added to `~/.claude/settings.json`)

Generated automatically by `claude-notify --setup`, or added manually:

```json
{
  "hooks": {
    "Notification": [{
      "matcher": "permission_prompt|idle_prompt|elicitation_dialog",
      "hooks": [{ "type": "command", "command": "claude-notify", "async": true }]
    }],
    "Stop": [{
      "hooks": [{ "type": "command", "command": "claude-notify", "async": true }]
    }],
    "TaskCompleted": [{
      "hooks": [{ "type": "command", "command": "claude-notify", "async": true }]
    }]
  }
}
```

Note: uses bare `claude-notify` (assumes installed to PATH via `~/.local/bin/` or `/usr/local/bin/`).

## Installation

```bash
# Build
cargo build --release

# Install to PATH
cp target/release/claude-notify ~/.local/bin/

# Auto-configure hooks
claude-notify --setup
```

## Implementation Steps

1. **Init project** — `cargo init --name claude-notify`, add `serde`, `serde_json`, `ureq`, `toml`, `clap` to `Cargo.toml`
2. **`src/types.rs`** — Define `HookEvent` struct with `#[derive(Deserialize)]`, use `Option<T>` for event-specific fields
3. **`src/config.rs`** — Load `~/.config/claude-notify/config.toml`, merge with env var overrides. Define `Config` struct with backends, events filter, and per-backend settings
4. **`src/notifier.rs`** — Define `Notifier` trait with `send(&self, message: &str) -> Result<()>` and `name(&self) -> &str`
5. **`src/notifiers/telegram.rs`** — Implement `Notifier` for `TelegramNotifier`, using `ureq::post()` to Telegram Bot API
6. **`src/notifiers/mod.rs`** — Registry: read config, return `Vec<Box<dyn Notifier>>` of active backends
7. **`src/formatter.rs`** — `format_message(event: &HookEvent) -> String`, match on `hook_event_name` and `notification_type`. Truncate to 4096 chars
8. **`src/setup.rs`** — Read existing `~/.claude/settings.json`, merge hook config, write back. Warn if hooks already configured
9. **`src/main.rs`** — Parse CLI args with clap. Route to `--setup`, `--dry-run`, or normal stdin→format→send flow
10. **Build & install** — `cargo build --release && cp target/release/claude-notify ~/.local/bin/`
11. **Configure** — `claude-notify --setup` or manually edit `~/.claude/settings.json`

## Key Design Decisions

- **HTML parse mode** over MarkdownV2 — only need to escape `< > &` vs 15+ special chars
- **async hooks** — notifications must never block Claude Code
- **Single binary for all events** — routes internally by `hook_event_name`, keeps config simple
- **`ureq` over `reqwest`** — no async runtime needed, smaller binary, faster compile
- **`Notifier` trait** — pluggable backends; adding Slack/WhatsApp later is just a new struct implementing the trait
- **Config file + env var layering** — config file for persistent setup, env vars for overrides and CI
- **`--setup` command** — zero-friction installation, no manual JSON editing
- **Event filtering** — users control notification noise level

## Verification

1. **Manual test**: `echo '{"session_id":"abc123","cwd":"/tmp/test","hook_event_name":"Notification","notification_type":"permission_prompt","tool_name":"Bash","tool_input":{"command":"rm -rf /"}}' | claude-notify`
2. **Dry run**: `echo '...' | claude-notify --dry-run` — prints formatted message to stdout
3. **Setup test**: `claude-notify --setup` — verify hooks appear in `~/.claude/settings.json`
4. **End-to-end**: Trigger a permission prompt in Claude Code, verify Telegram notification arrives
