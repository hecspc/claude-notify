# claude-notify — Architecture & Implementation Plan

## Context

When running Claude Code sessions (especially long-running or parallel ones), the user needs to know when Claude requires input — permissions, questions, or task completion. Without notifications, sessions sit idle waiting for attention. `claude-notify` sends notifications to configurable backends so the user can monitor from mobile or another screen. Built in Rust for a single distributable binary.

## Claude Code Events

| Hook Event | Matcher | What it means |
|---|---|---|
| `Notification` | `permission_prompt\|idle_prompt\|elicitation_dialog` | Claude needs user input |
| `Stop` | — | Claude finished responding (includes `last_assistant_message`) |
| `TaskCompleted` | — | A task completed (includes `task_subject`, `teammate_name`, `task_description`) |

All hooks run with `async: true` so they never block Claude Code.

## Technology Stack

- **Rust** (edition 2024, rustc 1.85+) — compiles to a single native binary
- **`ureq`** — lightweight, blocking HTTP client (no async runtime needed)
- **`serde` + `serde_json`** — JSON deserialization of hook payloads
- **`toml`** — config file parsing
- **`clap`** — CLI argument parsing with derive macros and subcommands

## Architecture

```
Claude Code Event → Hook (async) → claude-notify (binary) → Notifier trait → Telegram / Slack / ...
```

The hook invokes a native binary directly — no runtime needed on the target machine. The notification backend is abstracted behind a `Notifier` trait so new channels can be added without changing the core logic.

```rust
pub trait Notifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn name(&self) -> &str;
}
```

### Configuration

Config file at `~/.config/claude-notify/config.toml`:

```toml
backends = ["telegram", "slack"]

# Which events to notify on (defaults to all if omitted)
# events = ["permission_prompt", "idle_prompt", "elicitation_dialog", "stop", "task_completed"]

[telegram]
bot_token = "123456:ABC-DEF..."
chat_id = "123456789"

[slack]
webhook_url = "https://hooks.slack.com/services/T.../B.../xxx"
```

Env vars override config file values:

| Variable | Purpose | Example |
|---|---|---|
| `NOTIFY_BACKEND` | Active backend(s), comma-separated | `telegram,slack` |
| `NOTIFY_EVENTS` | Event filter, comma-separated | `permission_prompt,idle_prompt` |
| `TELEGRAM_BOT_TOKEN` | Token from @BotFather | `123456:ABC-DEF...` |
| `TELEGRAM_CHAT_ID` | User's chat ID | `123456789` |
| `SLACK_WEBHOOK_URL` | Incoming Webhook URL | `https://hooks.slack.com/services/...` |

### File Structure

```
Cargo.toml
src/
  main.rs              # CLI entry point (clap subcommands): setup, mute/unmute/status, --dry-run, stdin→format→send
  types.rs             # HookEvent struct (serde). All optional fields use Option<T>
  config.rs            # Config + per-backend config structs. TOML file + env var overrides
  formatter.rs         # format_message() maps HookEvent → HTML string. friendly_name() hashes session_id
  notifier.rs          # Notifier trait definition
  notifiers/
    mod.rs             # build_notifiers() registry: config → Vec<Box<dyn Notifier>>
    telegram.rs        # TelegramNotifier: ureq POST to Telegram Bot API with HTML parse mode
    slack.rs           # SlackNotifier: ureq POST to Slack Incoming Webhook with mrkdwn conversion
  setup.rs             # run_setup() writes backend config + merges hooks into settings.json
```

### CLI Interface

```
claude-notify                                                  # Normal: read hook JSON from stdin, notify
claude-notify setup telegram <BOT_TOKEN> <CHAT_ID>             # Configure Telegram + hooks (user-level)
claude-notify setup telegram <BOT_TOKEN> <CHAT_ID> --project   # Configure hooks in current project
claude-notify setup slack <WEBHOOK_URL>                        # Configure Slack + hooks (user-level)
claude-notify setup slack <WEBHOOK_URL> --project              # Configure hooks in current project
claude-notify mute                                             # Mute all notifications
claude-notify mute <session>                                   # Mute a specific session (friendly name or UUID)
claude-notify unmute                                           # Unmute all
claude-notify unmute <session>                                 # Unmute a specific session
claude-notify status                                           # Show mute status
claude-notify --dry-run                                        # Print formatted message to stdout, don't send
claude-notify --version                                        # Print version
```

### Message Format

Sessions identified by friendly name + short UUID + project:

```
🔔 Permission Required
Session: safe-seal (a3f2b1c9) | engineering-bot
─────────────────
Tool: Bash
Action: npm install express
```

Telegram receives HTML (`<b>` tags). Slack receives mrkdwn (`*` for bold), converted from HTML in the Slack notifier.

### Message Filtering

The `events` config key (or `NOTIFY_EVENTS` env var) controls which events trigger notifications. Defaults to all events.

### Mute/Unmute

Mute state stored as files in `~/.config/claude-notify/muted/`. `_global` file = all muted. Session name/UUID files = per-session mute.

## Runtime File Paths

- `~/.config/claude-notify/config.toml` — backend credentials + event filter
- `~/.config/claude-notify/muted/` — mute state files
- `~/.claude/settings.json` — user-level hooks (`--user` scope, default)
- `.claude/settings.json` — project-level hooks (`--project` scope)

## Hook Configuration

Generated by `claude-notify setup`, or added manually to `~/.claude/settings.json`:

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

## Key Design Decisions

- **HTML as internal format** — formatter produces HTML, each backend converts as needed (Telegram uses natively, Slack converts to mrkdwn)
- **async hooks** — notifications must never block Claude Code
- **Single binary for all events** — routes internally by `hook_event_name`, keeps config simple
- **`ureq` over `reqwest`** — no async runtime needed, smaller binary, faster compile
- **`Notifier` trait** — pluggable backends; adding a new one follows a 6-step checklist
- **Config file + env var layering** — config file for persistent setup, env vars for overrides
- **`setup` subcommand** — zero-friction installation with inline credentials
- **Friendly session names** — deterministic adjective-noun hash of session_id for readability
- **Per-session muting** — file-based mute state, supports friendly names and UUIDs

## Adding a New Notification Backend

1. Create `src/notifiers/newbackend.rs` implementing `Notifier` trait
2. Add config struct + fields to `Config` in `config.rs`
3. Add `pub mod newbackend;` and match arm in `notifiers/mod.rs`
4. Add env var overrides in `config.rs` `apply_env_overrides()`
5. Add variant to `SetupBackend` enum in `main.rs`
6. Add config writing logic in `setup.rs` `write_backend_config()`

## Verification

1. **Build**: `cargo build`
2. **Dry run**: `echo '{"session_id":"abc123","cwd":"/tmp/test","hook_event_name":"Notification","notification_type":"permission_prompt","tool_name":"Bash","tool_input":{"command":"ls"}}' | claude-notify --dry-run`
3. **Setup test**: `claude-notify setup telegram <token> <chat_id>` / `claude-notify setup slack <webhook_url>`
4. **Mute test**: `claude-notify mute` → `claude-notify status` → `claude-notify unmute`
5. **End-to-end**: Trigger a permission prompt in Claude Code, verify notification arrives on configured backends
