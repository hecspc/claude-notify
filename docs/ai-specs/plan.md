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
Claude Code Event → Hook (async) → claude-notify (binary) → Notifier trait → Desktop / Telegram / Slack / Discord / Ntfy
```

The hook invokes a native binary directly — no runtime needed on the target machine. The notification backend is abstracted behind a `Notifier` trait so new channels can be added without changing the core logic.

```rust
pub trait Notifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn name(&self) -> &str;
}
```

### Notification Backends

| Backend | Config Required | Transport | Success Status |
|---|---|---|---|
| Desktop | None (zero-config) | `osascript` (macOS) / `notify-send` (Linux) | exit code 0 |
| Telegram | `bot_token`, `chat_id` | ureq POST to Bot API | 200 |
| Slack | `webhook_url` | ureq POST to Incoming Webhook | 200 |
| Discord | `webhook_url` | ureq POST `{"content": text}` | 204 |
| Ntfy | `topic_url` | ureq POST plain text with `Title` header | 200 |

### Configuration

Config file at `~/.config/claude-notify/config.toml`:

```toml
backends = ["desktop"]

# Which events to notify on (defaults to all if omitted)
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

Env vars override config file values:

| Variable | Purpose | Example |
|---|---|---|
| `NOTIFY_BACKEND` | Active backend(s), comma-separated | `desktop`, `slack,discord` |
| `NOTIFY_EVENTS` | Event filter, comma-separated | `permission_prompt,idle_prompt` |
| `TELEGRAM_BOT_TOKEN` | Token from @BotFather | `123456:ABC-DEF...` |
| `TELEGRAM_CHAT_ID` | User's chat ID | `123456789` |
| `SLACK_WEBHOOK_URL` | Incoming Webhook URL | `https://hooks.slack.com/services/...` |
| `DISCORD_WEBHOOK_URL` | Discord webhook URL | `https://discord.com/api/webhooks/...` |
| `NTFY_TOPIC_URL` | ntfy topic URL | `https://ntfy.sh/my-topic` |

### File Structure

```
Cargo.toml
src/
  main.rs              # CLI entry point (clap subcommands): setup, use, mute/unmute/status, --dry-run, stdin→format→send
  types.rs             # HookEvent struct (serde). All optional fields use Option<T>
  config.rs            # Config + per-backend config structs. TOML file + env var overrides
  formatter.rs         # format_message() maps HookEvent → HTML string. friendly_name() hashes session_id
  notifier.rs          # Notifier trait definition
  notifiers/
    mod.rs             # build_notifiers() registry: config → Vec<Box<dyn Notifier>>
    telegram.rs        # TelegramNotifier: ureq POST to Telegram Bot API with HTML parse mode
    slack.rs           # SlackNotifier: ureq POST to Slack Incoming Webhook with mrkdwn conversion
    desktop.rs         # DesktopNotifier: osascript (macOS) / notify-send (Linux), no config needed
    discord.rs         # DiscordNotifier: ureq POST to Discord webhook, expects 204
    ntfy.rs            # NtfyNotifier: ureq POST plain text with Title header
  setup.rs             # run_setup() writes backend config + hooks + skills (--user or --project scope)
.github/
  workflows/
    ci.yml             # Build + clippy on Ubuntu and macOS for PRs and pushes to main
    release.yml        # Detects version change, builds release binaries, creates tag + GitHub release
.claude/
  skills/
    release/           # /release skill: bump version, update changelog, commit, push
    dry-run/           # /dry-run skill: test notification formatting
    add-backend/       # /add-backend skill: scaffold a new backend
```

### CLI Interface

```
claude-notify                                                  # Normal: read hook JSON from stdin, notify
claude-notify setup telegram <BOT_TOKEN> <CHAT_ID>             # Configure Telegram + hooks (user-level)
claude-notify setup telegram <BOT_TOKEN> <CHAT_ID> --project   # Configure hooks in current project
claude-notify setup slack <WEBHOOK_URL>                        # Configure Slack + hooks
claude-notify setup desktop                                    # Configure desktop notifications + hooks
claude-notify setup discord <WEBHOOK_URL>                      # Configure Discord + hooks
claude-notify setup ntfy <TOPIC_URL>                           # Configure ntfy + hooks
claude-notify use desktop                                      # Switch active backend
claude-notify use desktop,slack                                # Use multiple backends
claude-notify mute                                             # Mute all notifications
claude-notify mute <session>                                   # Mute a specific session (friendly name or UUID)
claude-notify unmute                                           # Unmute all
claude-notify unmute <session>                                 # Unmute a specific session
claude-notify status                                           # Show mute status
claude-notify --dry-run                                        # Print formatted message to stdout, don't send
claude-notify --version                                        # Print version
```

### Claude Code Skills

`setup` installs these slash commands into `~/.claude/skills/` (or `.claude/skills/` with `--project`):

| Skill | Description |
|---|---|
| `/notify-mute` | Mute all notifications, or pass a session name to mute one |
| `/notify-unmute` | Unmute all notifications, or pass a session name to unmute one |
| `/notify-use` | Switch active backends (e.g. `/notify-use desktop,slack`) |
| `/notify-session` | Toggle mute for the current session using `${CLAUDE_SESSION_ID}` |

### Message Format

Sessions identified by friendly name + short UUID + project:

```
🔔 Permission Required
Session: safe-seal (a3f2b1c9) | engineering-bot
─────────────────
Tool: Bash
Action: npm install express
```

Telegram receives HTML (`<b>` tags). Slack receives mrkdwn (`*` for bold). Discord receives Discord markdown (`**` for bold). Desktop and ntfy receive plain text (HTML tags stripped, entities unescaped).

### Message Filtering

The `events` config key (or `NOTIFY_EVENTS` env var) controls which events trigger notifications. Defaults to all events.

### Mute/Unmute

Mute state stored as files in `~/.config/claude-notify/muted/`. `_global` file = all muted. Session name/UUID files = per-session mute.

## Runtime File Paths

- `~/.config/claude-notify/config.toml` — backend credentials + event filter
- `~/.config/claude-notify/muted/` — mute state files
- `~/.claude/settings.json` — user-level hooks (`--user` scope, default)
- `.claude/settings.json` — project-level hooks (`--project` scope)
- `~/.claude/skills/{notify-mute,notify-unmute,notify-use,notify-session}/SKILL.md` — Claude Code slash commands

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

## CI/CD

### CI (`ci.yml`)

Runs on PRs and pushes to main. Builds and runs clippy on both Ubuntu and macOS.

### Release (`release.yml`)

Triggered by pushes to main that modify `Cargo.toml`. Detects if the `version` field actually changed, then:
1. Builds release binaries for `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-apple-darwin`
2. Creates and pushes a git tag (`vX.Y.Z`)
3. Extracts the changelog section for the version from `CHANGELOG.md`
4. Creates a GitHub release with binaries and changelog as description

## Key Design Decisions

- **HTML as internal format** — formatter produces HTML, each backend converts as needed
- **async hooks** — notifications must never block Claude Code
- **Single binary for all events** — routes internally by `hook_event_name`, keeps config simple
- **`ureq` over `reqwest`** — no async runtime needed, smaller binary, faster compile
- **`Notifier` trait** — pluggable backends; adding a new one follows a 6-step checklist
- **Config file + env var layering** — config file for persistent setup, env vars for overrides
- **`setup` subcommand** — zero-friction installation with inline credentials + hooks + skills
- **Friendly session names** — deterministic adjective-noun hash of session_id for readability
- **Per-session muting** — file-based mute state, supports friendly names and UUIDs
- **`use` subcommand** — quick backend switching without editing config file
- **Desktop backend** — zero-config native notifications via osascript/notify-send
- **Skills** — Claude Code slash commands installed by setup for in-session control

## Adding a New Notification Backend

1. Create `src/notifiers/newbackend.rs` implementing `Notifier` trait
2. Add config struct + fields to `Config` in `config.rs`
3. Add `pub mod newbackend;` and match arm in `notifiers/mod.rs`
4. Add env var overrides in `config.rs` `apply_env_overrides()`
5. Add variant to `SetupBackend` enum in `main.rs`
6. Add config writing logic in `setup.rs` `write_backend_config()`

## Verification

1. **Build**: `cargo build`
2. **Clippy**: `cargo clippy -- -D warnings`
3. **Dry run**: `echo '...' | claude-notify --dry-run`
4. **Setup test**: `claude-notify setup desktop` / `telegram` / `slack` / `discord` / `ntfy`
5. **Use test**: `claude-notify use desktop` → `claude-notify use slack` → `claude-notify use desktop,slack`
6. **Mute test**: `claude-notify mute` → `claude-notify status` → `claude-notify unmute`
7. **End-to-end**: Trigger a permission prompt in Claude Code, verify notification arrives
