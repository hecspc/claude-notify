# claude-notify Plugin for Claude Code

This plugin auto-registers hooks and provides slash commands for [claude-notify](https://github.com/hecspc/claude-notify) — a notification system for Claude Code.

## Prerequisites

The `claude-notify` binary must be installed and available in your `$PATH`:

```bash
curl -sSL https://raw.githubusercontent.com/hecspc/claude-notify/main/install.sh | sh
```

## Install Plugin

```bash
claude plugin install claude-notify
```

## What the Plugin Does

1. **Auto-registers hooks** for `Notification`, `Stop`, and `TaskCompleted` events — no manual `settings.json` editing needed
2. **Provides slash commands** for configuring and managing notifications from within Claude Code

## Available Skills

### Setup Commands

| Skill | Description |
|---|---|
| `/claude-notify:setup-desktop` | Enable native OS notifications (zero-config) |
| `/claude-notify:setup-telegram` | Configure Telegram bot notifications |
| `/claude-notify:setup-slack` | Configure Slack webhook notifications |
| `/claude-notify:setup-discord` | Configure Discord webhook notifications |
| `/claude-notify:setup-ntfy` | Configure ntfy push notifications |
| `/claude-notify:setup-pushbullet` | Configure Pushbullet notifications |
| `/claude-notify:setup-teams` | Configure Microsoft Teams notifications |
| `/claude-notify:setup-webhook` | Configure generic webhook notifications |
| `/claude-notify:setup-email` | Configure email (SMTP) notifications |
| `/claude-notify:setup-whatsapp` | Configure WhatsApp notifications via Meta Cloud API |
| `/claude-notify:setup-openclaw` | Configure OpenClaw Gateway notifications |

### Control Commands

| Skill | Description |
|---|---|
| `/claude-notify:use` | Switch active backends (e.g. `desktop,slack`) |
| `/claude-notify:mute` | Mute notifications globally or for a session |
| `/claude-notify:unmute` | Unmute notifications |
| `/claude-notify:status` | Show current mute status |
| `/claude-notify:session` | Toggle mute for the current session |

## Plugin vs `setup` Command

If you install the plugin, you **don't need to run `claude-notify setup`** for hook wiring — the plugin handles that automatically. You only need `setup` (or the setup skills) to configure backend credentials.
