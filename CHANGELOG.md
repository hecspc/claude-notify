# Changelog

All notable changes to claude-notify are documented here.

## [Unreleased]

## [0.1.1] - 2026-03-14

### Added
- Slack notification backend via Incoming Webhooks (`claude-notify setup slack <WEBHOOK_URL>`)
- Desktop notification backend — zero-config, uses native OS (`osascript` on macOS, `notify-send` on Linux)
- Discord notification backend via webhooks (`claude-notify setup discord <WEBHOOK_URL>`)
- Ntfy notification backend for self-hosted push (`claude-notify setup ntfy <TOPIC_URL>`)
- `use` command to switch active backends without editing config (`claude-notify use desktop,slack`)
- Claude Code slash commands installed by `setup`: `/notify-mute`, `/notify-unmute`, `/notify-use`, `/notify-session`
- `/notify-session` skill uses `${CLAUDE_SESSION_ID}` to toggle mute for the current session
- `SLACK_WEBHOOK_URL`, `DISCORD_WEBHOOK_URL`, `NTFY_TOPIC_URL` environment variable overrides
- HTML-to-mrkdwn conversion for Slack message formatting

## [0.1.0] - 2026-03-14

### Added
- Initial release of claude-notify
- Telegram notification backend via Bot API (HTML parse mode)
- Hook event support: `Notification` (permission_prompt, idle_prompt, elicitation_dialog), `Stop`, `TaskCompleted`
- `setup` subcommand with inline backend configuration: `claude-notify setup telegram <token> <chat_id>`
- `--user` and `--project` scope flags for hook installation
- Config file at `~/.config/claude-notify/config.toml` with env var overrides
- Event filtering via `events` config key or `NOTIFY_EVENTS` env var
- `--dry-run` flag for testing message formatting without sending
- Rich context in Stop notifications (`last_assistant_message` summary)
- Rich context in TaskCompleted notifications (`task_subject`, `teammate_name`, `task_description`)
- Friendly session names derived from session_id hash (e.g. `safe-seal`) with short UUID in parentheses
- `mute` / `unmute` commands for global or per-session notification control
- `status` command to check mute state
- Pluggable `Notifier` trait for future backends
