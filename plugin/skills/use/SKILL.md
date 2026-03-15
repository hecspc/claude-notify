---
name: use
description: "Switch active claude-notify backends (e.g. desktop, slack, discord, ntfy, telegram). Pass comma-separated backend names."
---

# Switch Notification Backends

Run `claude-notify use <backends>` to switch which notification backends are active.

## Usage

The user provides one or more backend names (comma-separated). Valid backends: `desktop`, `telegram`, `slack`, `discord`, `ntfy`, `pushbullet`, `teams`, `webhook`, `email`.

Examples:
- `claude-notify use desktop` — desktop only
- `claude-notify use slack,discord` — Slack and Discord
- `claude-notify use desktop,telegram` — desktop and Telegram

Run the command with the user's chosen backend(s), then show the output.
