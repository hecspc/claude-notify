# claude-notify

Notification bot for [Claude Code](https://docs.anthropic.com/en/docs/claude-code) hook events. Get Telegram or Slack notifications when Claude needs your input — permission prompts, questions, idle sessions, or task completions.

Built in Rust for a single native binary with no runtime dependencies.

## Why

When running Claude Code sessions (especially long-running or parallel ones), sessions sit idle waiting for attention. `claude-notify` sends notifications to Telegram or Slack so you can monitor from mobile or another screen.

## Quick Start

```bash
# Build and install
cargo build --release
cp target/release/claude-notify ~/.local/bin/

# One-command setup: configures credentials + hooks
claude-notify setup telegram YOUR_BOT_TOKEN YOUR_CHAT_ID
# Or for Slack:
claude-notify setup slack https://hooks.slack.com/services/T.../B.../xxx
```

This writes `~/.config/claude-notify/config.toml` with your credentials and adds hooks to `~/.claude/settings.json`.

## Supported Events

| Event | Icon | Description |
|---|---|---|
| Permission prompt | :bell: | Claude needs tool approval (Bash, Edit, etc.) |
| Idle prompt | :hourglass_flowing_sand: | Claude is waiting for your response |
| Elicitation dialog | :question: | Claude is asking a question |
| Response complete | :white_check_mark: | Claude finished responding (includes last message summary) |
| Task completed | :tada: | A background task finished (includes task subject, teammate, description) |

## Message Format

```
🔔 Permission Required
Session: safe-seal (66a021e0) | engineering-bot
─────────────────
Tool: Bash
Action: npm install express
```

```
✅ Response Complete
Session: safe-seal (66a021e0) | engineering-bot
─────────────────
I've updated the README.md with the new setup instructions and rebuilt the release binary.
```

```
🎉 Task Completed
Session: pink-swan (abc123) | engineering-bot
Task: Implement notification system
Teammate: implementer
─────────────────
Add Telegram notifications for Claude Code hook events
```

Sessions are identified by a friendly name derived from the session_id (e.g. `safe-seal`) plus the short UUID and project name, for quick identification across parallel sessions.

## CLI Usage

```
claude-notify                                                  # Normal: read hook JSON from stdin, notify
claude-notify setup telegram <BOT_TOKEN> <CHAT_ID>             # Configure credentials + hooks (user-level)
claude-notify setup telegram <BOT_TOKEN> <CHAT_ID> --project   # Configure hooks in current project
claude-notify setup slack <WEBHOOK_URL>                        # Configure Slack notifications
claude-notify mute                                             # Mute all notifications
claude-notify mute safe-seal                                   # Mute a specific session (friendly name or UUID)
claude-notify unmute                                           # Unmute all
claude-notify unmute safe-seal                                 # Unmute a specific session
claude-notify status                                           # Show mute status
claude-notify --dry-run                                        # Print formatted message to stdout, don't send
claude-notify --version                                        # Print version
```

### Muting

Mute notifications globally or per-session. Use the friendly name from the notification (e.g. `safe-seal`) or the raw session UUID.

```bash
claude-notify mute              # Silence everything
claude-notify mute safe-seal    # Silence one session
claude-notify status            # Check what's muted
claude-notify unmute            # Re-enable all
```

Mute state is stored as files in `~/.config/claude-notify/muted/`.

### Setup Scopes

- `--user` (default) — writes hooks to `~/.claude/settings.json`, applies to all projects
- `--project` — writes hooks to `.claude/settings.json` in the current directory, applies to this project only

Both scopes write backend credentials to `~/.config/claude-notify/config.toml`.

## Configuration

### Config File

`~/.config/claude-notify/config.toml`:

```toml
backends = ["telegram"]  # or ["slack"], or ["telegram", "slack"] for both

# Optional: filter which events trigger notifications (defaults to all)
# events = ["permission_prompt", "idle_prompt", "elicitation_dialog", "stop", "task_completed"]

[telegram]
bot_token = "123456:ABC-DEF..."
chat_id = "123456789"

[slack]
webhook_url = "https://hooks.slack.com/services/T.../B.../xxx"
```

### Environment Variables

Env vars override config file values.

| Variable | Purpose | Example |
|---|---|---|
| `NOTIFY_BACKEND` | Active backend(s), comma-separated | `telegram`, `slack`, `telegram,slack` |
| `NOTIFY_EVENTS` | Event filter, comma-separated | `permission_prompt,idle_prompt` |
| `TELEGRAM_BOT_TOKEN` | Token from @BotFather | `123456:ABC-DEF...` |
| `TELEGRAM_CHAT_ID` | User's chat ID | `123456789` |
| `SLACK_WEBHOOK_URL` | Slack Incoming Webhook URL | `https://hooks.slack.com/services/...` |

### Event Filtering

To silence noisy events like "Response Complete":

```toml
events = ["permission_prompt", "idle_prompt", "elicitation_dialog", "task_completed"]
```

## Slack Setup

1. Go to [Slack API: Incoming Webhooks](https://api.slack.com/messaging/webhooks) and create a new app (or use an existing one)
2. Enable Incoming Webhooks and add one to your desired channel
3. Copy the webhook URL
4. Run `claude-notify setup slack <WEBHOOK_URL>`

## Telegram Setup

1. Message [@BotFather](https://t.me/BotFather) on Telegram, send `/newbot`, and follow the prompts to get a bot token
2. Message [@userinfobot](https://t.me/userinfobot) to get your chat ID
3. Run `claude-notify setup telegram <BOT_TOKEN> <CHAT_ID>`

## Hook Configuration

Generated by `claude-notify setup`, or add manually to `~/.claude/settings.json`:

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

All hooks use `async: true` so they never block Claude Code.

## Architecture

```
Claude Code Event → Hook (async) → claude-notify → Notifier trait → Telegram / Slack
```

The notification backend is abstracted behind a `Notifier` trait. Adding new backends (WhatsApp, Discord, etc.) requires implementing a single trait:

```rust
pub trait Notifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn name(&self) -> &str;
}
```

## Testing

```bash
# Dry run a permission prompt
echo '{"session_id":"abc123","cwd":"/tmp/test","hook_event_name":"Notification","notification_type":"permission_prompt","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-notify --dry-run

# Dry run a stop event with last message
echo '{"session_id":"abc123","cwd":"/tmp/test","hook_event_name":"Stop","last_assistant_message":"I fixed the bug in the login handler."}' | claude-notify --dry-run

# Dry run a task completed event
echo '{"session_id":"abc123","cwd":"/tmp/test","hook_event_name":"TaskCompleted","task_subject":"Fix auth bug","teammate_name":"implementer"}' | claude-notify --dry-run
```

## License

MIT
