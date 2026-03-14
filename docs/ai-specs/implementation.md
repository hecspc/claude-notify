# claude-notify — Implementation

## Build Status

Binary compiles to a 3.6 MB release binary. Installed to `~/.bin/claude-notify`.

## Project Structure

```
Cargo.toml
LICENSE
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
    email.rs           # Email via SMTP with STARTTLS (lettre), plain text
    ntfy.rs            # ntfy implementation (ureq), plain text POST with Title header
    pushbullet.rs      # Pushbullet implementation (ureq), POST to v2/pushes with Access-Token
    teams.rs           # Microsoft Teams (ureq), POST Adaptive Card to Workflows webhook
    webhook.rs         # Generic webhook (ureq), POST JSON {title, body, text} to any URL
  setup.rs             # setup subcommand: write backend config + hooks + skills (--user or --project)
.github/
  workflows/
    ci.yml             # Build + clippy on Ubuntu and macOS for PRs and pushes to main
    release.yml        # Detects version change, builds release binaries, creates tag + GitHub release
.claude/
  skills/
    release/SKILL.md   # /release: bump version, update changelog, commit, push
    dry-run/SKILL.md   # /dry-run: test notification formatting
    add-backend/SKILL.md # /add-backend: scaffold a new backend
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

Structs: `Config` (backends, events, telegram, slack, discord, ntfy, pushbullet, webhook, teams, email), `TelegramConfig` (bot_token, chat_id), `SlackConfig` (webhook_url), `DiscordConfig` (webhook_url), `NtfyConfig` (topic_url), `PushbulletConfig` (api_token), `WebhookConfig` (url), `TeamsConfig` (webhook_url), `EmailConfig` (from, to, smtp_host, smtp_port, smtp_username, smtp_password).

Env var overrides: `NOTIFY_BACKEND`, `NOTIFY_EVENTS`, `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID`, `SLACK_WEBHOOK_URL`, `DISCORD_WEBHOOK_URL`, `NTFY_TOPIC_URL`, `PUSHBULLET_API_TOKEN`, `TEAMS_WEBHOOK_URL`, `WEBHOOK_URL`, `EMAIL_FROM`, `EMAIL_TO`, `EMAIL_SMTP_HOST`, `EMAIL_SMTP_PORT`, `EMAIL_SMTP_USERNAME`, `EMAIL_SMTP_PASSWORD`.

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

### `src/notifiers/email.rs`

Email backend using `lettre` crate. Sends plain text emails via SMTP with STARTTLS on port 587. First line of message becomes the subject, rest becomes the body. Requires from, to, smtp_host, smtp_username, smtp_password config.

### `src/notifiers/ntfy.rs`

Ntfy backend for self-hosted push notifications. `html_to_plain()` strips tags and unescapes entities. POSTs plain text body with `Title` header to the topic URL.

### `src/notifiers/pushbullet.rs`

Pushbullet backend. `html_to_plain()` strips tags and unescapes entities. Splits message into title (first line) + body. POSTs `{"type": "note", "title": ..., "body": ...}` to `https://api.pushbullet.com/v2/pushes` with `Access-Token` header.

### `src/notifiers/teams.rs`

Microsoft Teams backend. Uses the Adaptive Card format required by Teams Workflows webhooks (legacy Office 365 connectors are deprecated). Converts `<b>` to `**` for bold. POSTs an Adaptive Card with a single TextBlock to the webhook URL.

### `src/notifiers/webhook.rs`

Generic webhook backend. POSTs JSON `{"title": ..., "body": ..., "text": ...}` to any URL. Accepts any 2xx response as success. Allows integration with arbitrary services (Home Assistant, Zapier, custom endpoints).

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
          → EmailNotifier.send() → SMTP server
          → NtfyNotifier.send() → ntfy topic URL
          → PushbulletNotifier.send() → Pushbullet API
          → TeamsNotifier.send() → Teams Workflows webhook
          → WebhookNotifier.send() → any HTTP endpoint
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
| `EMAIL_FROM` | `[email].from` |
| `EMAIL_TO` | `[email].to` |
| `EMAIL_SMTP_HOST` | `[email].smtp_host` |
| `EMAIL_SMTP_PORT` | `[email].smtp_port` |
| `EMAIL_SMTP_USERNAME` | `[email].smtp_username` |
| `EMAIL_SMTP_PASSWORD` | `[email].smtp_password` |
| `NTFY_TOPIC_URL` | `[ntfy].topic_url` |
| `PUSHBULLET_API_TOKEN` | `[pushbullet].api_token` |
| `TEAMS_WEBHOOK_URL` | `[teams].webhook_url` |
| `WEBHOOK_URL` | `[webhook].url` |

## Installation

```bash
cargo build --release
cp target/release/claude-notify ~/.bin/
claude-notify setup desktop                                    # zero-config
claude-notify setup telegram <BOT_TOKEN> <CHAT_ID>             # Telegram
claude-notify setup slack <WEBHOOK_URL>                        # Slack
claude-notify setup discord <WEBHOOK_URL>                      # Discord
claude-notify setup ntfy <TOPIC_URL>                           # ntfy
```

## CI/CD

### CI (`ci.yml`)

Runs on every PR and push to main. Matrix build on Ubuntu and macOS:
- `cargo build` — verify compilation
- `cargo clippy -- -D warnings` — lint with all warnings as errors

### Release (`release.yml`)

Triggered by pushes to main that modify `Cargo.toml`. Only proceeds if the `version` field actually changed (compares current vs previous commit).

Steps:
1. **check-version** — extracts version, compares with previous commit
2. **build** — matrix build for 3 targets: `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-apple-darwin`
3. **release** — extracts changelog section, creates/pushes git tag (`vX.Y.Z`), creates GitHub release with binaries and changelog as description

### `/release` Project Skill

The `/release` skill automates version bumps:
1. Reads current version from `Cargo.toml`
2. Bumps based on argument (`patch`, `minor`, `major`, or explicit version)
3. Updates `Cargo.toml` version
4. Renames `[Unreleased]` in `CHANGELOG.md` to `[X.Y.Z] - YYYY-MM-DD`, adds new empty `[Unreleased]`
5. Runs `cargo build` to verify
6. Commits and pushes to both `origin` and `upstream`

## Claude Code Skills (installed by setup)

`setup` installs these slash commands into `~/.claude/skills/` (or `.claude/skills/` with `--project`):

| Skill | File | Description |
|---|---|---|
| `/notify-mute` | `notify-mute/SKILL.md` | Runs `claude-notify mute [session]` |
| `/notify-unmute` | `notify-unmute/SKILL.md` | Runs `claude-notify unmute [session]` |
| `/notify-use` | `notify-use/SKILL.md` | Runs `claude-notify use <backends>` |
| `/notify-session` | `notify-session/SKILL.md` | Toggles mute for current session using `${CLAUDE_SESSION_ID}` |

The `/notify-session` skill uses Claude Code's `${CLAUDE_SESSION_ID}` string substitution to automatically target the active session without user input.

## Adding a New Backend

1. Create `src/notifiers/newbackend.rs` implementing `Notifier` trait
2. Add config struct fields to `Config` in `config.rs`
3. Add match arm in `notifiers/mod.rs` `build_notifiers()`
4. Add env var overrides in `config.rs` `apply_env_overrides()`
5. Add a variant to `SetupBackend` enum in `main.rs` for `setup` subcommand support
6. Add config writing logic in `setup.rs` `write_backend_config()`
