# claude-notify — Implementation

## Build Status

Binary compiles to a 3.6 MB release binary. Installed to `~/.local/bin/claude-notify`.

## Project Structure

```
Cargo.toml
src/
  main.rs              # CLI entry point: clap subcommands, stdin → format → dispatch, cmd_use()
  types.rs             # HookEvent struct (serde deserialization)
  config.rs            # Config loading: TOML file + env var overrides
  formatter.rs         # Event → human-readable HTML message
  notifier.rs          # Notifier trait definition
  notifiers/
    mod.rs             # Backend registry: config → Vec<Box<dyn Notifier>>
    telegram.rs        # Telegram Bot API implementation (ureq)
    slack.rs           # Slack Incoming Webhook implementation (ureq), HTML→mrkdwn conversion
    desktop.rs         # Native OS notifications: osascript (macOS) / notify-send (Linux)
    discord.rs         # Discord webhook implementation (ureq), HTML→Discord markdown
    ntfy.rs            # ntfy implementation (ureq), plain text POST with Title header
  setup.rs             # setup subcommand: write backend config + hooks + skills (--user or --project)
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

- **serde + serde_json** — deserialize hook JSON payloads, serialize API requests
- **ureq** — lightweight blocking HTTP client, no async runtime needed
- **toml** — parse `~/.config/claude-notify/config.toml`
- **clap** — CLI argument parsing with derive macros

## Source Files

### `src/types.rs`

All fields except `session_id` and `hook_event_name` are `Option<T>` because different event types carry different payloads. Uses `serde_json::Value` for `tool_input` since its shape varies by tool.

### `src/config.rs`

Loading order: TOML file → env var overrides. If no backends specified, defaults to `["telegram"]`. Event filtering uses the `events` list — `None` means all events pass through.

Structs: `Config` (backends, events, telegram, slack, discord, ntfy), `TelegramConfig` (bot_token, chat_id), `SlackConfig` (webhook_url), `DiscordConfig` (webhook_url), `NtfyConfig` (topic_url).

Env var overrides: `NOTIFY_BACKEND`, `NOTIFY_EVENTS`, `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID`, `SLACK_WEBHOOK_URL`, `DISCORD_WEBHOOK_URL`, `NTFY_TOPIC_URL`.

### `src/notifier.rs`

Trait object interface for pluggable backends. `send()` dispatches a message, `name()` is used in error messages.

### `src/notifiers/telegram.rs`

Uses HTML parse mode (only need to escape `< > &`). `disable_web_page_preview` prevents link previews cluttering the notification. POSTs to Telegram Bot API via `ureq`.

### `src/notifiers/slack.rs`

Uses Slack Incoming Webhooks — simplest integration, no OAuth needed. `html_to_mrkdwn()` converts the HTML-formatted message (designed for Telegram) to Slack's mrkdwn format: `<b>`→`*` for bold, and unescapes HTML entities (`&amp;`, `&lt;`, `&gt;`). POSTs `{"text": ...}` to the webhook URL.

### `src/notifiers/desktop.rs`

Zero-config backend. `html_to_plain()` strips HTML tags and unescapes entities. Splits message into title (first line) + body, then dispatches via `osascript` on macOS or `notify-send` on Linux.

### `src/notifiers/discord.rs`

Discord webhook backend. `html_to_discord()` converts `<b>` to `**` (Discord bold) and unescapes entities. POSTs `{"content": text}` to the webhook URL. Success is 204 (not 200).

### `src/notifiers/ntfy.rs`

Ntfy backend for self-hosted push notifications. `html_to_plain()` strips tags and unescapes entities. POSTs plain text body with `Title` header to the topic URL.

### `src/notifiers/mod.rs`

Registry pattern: reads `config.backends` and constructs the corresponding `Notifier` implementations. Adding a new backend means adding a match arm and a new module. Desktop requires no config check.

### `src/formatter.rs`

Message format: `header + session_line + detail`. Each event type maps to an icon and label. `extract_action` smart-extracts the most useful field from `tool_input` (command for Bash, file_path for Edit/Write, truncated JSON for everything else). Truncated to 4096 chars (Telegram message limit). `friendly_name()` hashes session_id to an adjective-noun pair.

### `src/setup.rs`

Setup does three things:
1. **`write_backend_config()`** — writes `~/.config/claude-notify/config.toml` with backend credentials (always user-level)
2. **`write_hooks()`** — merges hook entries into `settings.json` at the chosen scope (`--user` → `~/.claude/settings.json`, `--project` → `.claude/settings.json`)
3. **`write_skills()`** — installs Claude Code slash commands (`/notify-mute`, `/notify-unmute`, `/notify-use`, `/notify-session`) as SKILL.md files

Detects if hooks are already configured to avoid duplicates. All hooks use `"async": true` so they never block Claude Code. The `/notify-session` skill uses `${CLAUDE_SESSION_ID}` substitution to target the active session.

### `src/main.rs`

CLI uses clap subcommands. `setup` is a subcommand with a nested backend subcommand (`Telegram`, `Slack`, `Desktop`, `Discord`, `Ntfy`) and `--user`/`--project` scope flags. Also has `mute`, `unmute`, `status`, `use` subcommands. Normal mode (no subcommand) reads from stdin as before.

Usage:
```
claude-notify setup telegram <BOT_TOKEN> <CHAT_ID>             # user-level (default)
claude-notify setup telegram <BOT_TOKEN> <CHAT_ID> --project   # project-level
claude-notify setup slack <WEBHOOK_URL>                        # Slack notifications
claude-notify setup desktop                                    # Desktop notifications (zero-config)
claude-notify setup discord <WEBHOOK_URL>                      # Discord notifications
claude-notify setup ntfy <TOPIC_URL>                           # ntfy notifications
claude-notify use desktop                                      # Switch active backend(s)
claude-notify use desktop,slack                                # Multiple backends
claude-notify --dry-run                                        # test formatting
```

`cmd_use()` loads config.toml, replaces the `backends` array, and writes it back — no other config is touched.

Flow: parse CLI → route to `setup`/`use`/`mute`/`unmute`/`status` subcommand or stdin mode → read JSON → check mute status → check event filter → format → send via all active backends. Errors go to stderr (invisible to Claude Code since hooks are async).

## Message Output Examples

Sessions are identified by a friendly name derived from the session_id hash (e.g. `safe-seal`) plus the short UUID and project name.

**Permission prompt:**
```
🔔 Permission Required
Session: safe-seal (a3f2b1c9) | engineering-bot
─────────────────
Tool: Bash
Action: npm install express
```

**Stop (response complete) — includes last assistant message:**
```
✅ Response Complete
Session: safe-seal (66a021e0) | engineering-bot
─────────────────
I've updated the README.md with the new setup instructions and rebuilt the release binary.
```

**Task completed — includes task subject, teammate, and description:**
```
🎉 Task Completed
Session: pink-swan (abc123) | engineering-bot
Task: Implement notification system
Teammate: implementer
─────────────────
Add Telegram notifications for Claude Code hook events
```

**Idle prompt:**
```
⏳ Waiting for Input
Session: calm-fox (abc123) | test
Claude is idle and waiting for your response.
```

**Elicitation dialog:**
```
❓ Question
Session: calm-fox (abc123) | test
Which database should I use?
```

## Data Flow

```
Claude Code Event
  → Hook (async: true, never blocks)
    → stdin: JSON payload
      → claude-notify binary
        → deserialize HookEvent (types.rs)
        → check mute status (muted/ dir)
        → load Config (config.rs) — TOML + env vars
        → check event filter (config.should_notify)
        → format message (formatter.rs)
        → dispatch to backends (notifiers/mod.rs)
          → DesktopNotifier.send() → osascript / notify-send
          → TelegramNotifier.send() → Telegram Bot API
          → SlackNotifier.send() → Slack Incoming Webhook
          → DiscordNotifier.send() → Discord Webhook API
          → NtfyNotifier.send() → ntfy topic URL
```

## Configuration

Config file: `~/.config/claude-notify/config.toml`

```toml
backends = ["desktop"]  # or ["telegram"], ["slack"], ["desktop", "slack"], etc.

# Optional: filter which events trigger notifications
# events = ["permission_prompt", "idle_prompt", "elicitation_dialog", "stop", "task_completed"]

[telegram]
bot_token = "123456:ABC-DEF..."
chat_id = "123456789"

[slack]
webhook_url = "https://hooks.slack.com/services/T.../B.../xxx"

[discord]
webhook_url = "https://discord.com/api/webhooks/123/abc"

[ntfy]
topic_url = "https://ntfy.sh/my-claude-topic"
```

Environment variables override config file values:

| Variable | Overrides |
|---|---|
| `NOTIFY_BACKEND` | `backends` |
| `NOTIFY_EVENTS` | `events` |
| `TELEGRAM_BOT_TOKEN` | `[telegram].bot_token` |
| `TELEGRAM_CHAT_ID` | `[telegram].chat_id` |
| `SLACK_WEBHOOK_URL` | `[slack].webhook_url` |
| `DISCORD_WEBHOOK_URL` | `[discord].webhook_url` |
| `NTFY_TOPIC_URL` | `[ntfy].topic_url` |

## Installation

```bash
cargo build --release
cp target/release/claude-notify ~/.local/bin/
claude-notify setup desktop                                    # zero-config
claude-notify setup telegram <BOT_TOKEN> <CHAT_ID>             # Telegram
claude-notify setup slack <WEBHOOK_URL>                        # Slack
claude-notify setup discord <WEBHOOK_URL>                      # Discord
claude-notify setup ntfy <TOPIC_URL>                           # ntfy
```

## Adding a New Backend

1. Create `src/notifiers/newbackend.rs` implementing `Notifier` trait
2. Add config struct fields to `Config` in `config.rs`
3. Add match arm in `notifiers/mod.rs` `build_notifiers()`
4. Add env var overrides in `config.rs` `apply_env_overrides()`
5. Add a variant to `SetupBackend` enum in `main.rs` for `setup` subcommand support
6. Add config writing logic in `setup.rs` `write_backend_config()`
