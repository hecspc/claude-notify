# Changelog

All notable changes to claude-notify are documented here.

## [Unreleased]

## [1.2.0] - 2026-03-15

### Added
- WhatsApp notification backend via Meta Cloud API (`claude-notify setup whatsapp <PHONE_ID> <TOKEN> <RECIPIENT>`)
- OpenClaw notification backend via Gateway API (`claude-notify setup openclaw <URL> <TOKEN> <TARGET>`)
- Claude Code plugin (`plugin/`) for auto-hook registration and namespaced skills
- Plugin manifest at repo root (`.claude-plugin/plugin.json`) for marketplace discovery
- 16 plugin skills: setup for all 11 backends + mute, unmute, use, status, session
- `WHATSAPP_PHONE_NUMBER_ID`, `WHATSAPP_ACCESS_TOKEN`, `WHATSAPP_RECIPIENT` environment variable overrides
- `OPENCLAW_GATEWAY_URL`, `OPENCLAW_TOKEN`, `OPENCLAW_TARGET`, `OPENCLAW_CHANNEL` environment variable overrides
- Default install directory changed from `~/.bin` to `~/.local/bin`

## [1.1.3] - 2026-03-15

### Added
- Claude Code plugin (`plugin/`) for auto-hook registration and namespaced skills
- 14 plugin skills: setup for all 9 backends + mute, unmute, use, status, session
- Plugin README with installation and usage guide
- Named webhook instances (`webhook.name`) for multiple webhook targets
- Custom headers support for webhooks (auth tokens, etc.)
- Default install directory changed from `~/.bin` to `~/.local/bin`

### Changed
- `claude-notify setup webhook` now supports `<NAME> <URL>` for named instances

## [1.1.2] - 2026-03-14

### Added
- Windows desktop notifications via PowerShell toast (Windows.UI.Notifications)
- Windows x86_64 release binary and CI support

## [1.1.1] - 2026-03-14

### Added
- Email notification backend via SMTP with STARTTLS (`claude-notify setup email <FROM> <TO> <HOST> <USER> <PASS>`)
- Microsoft Teams notification backend via Workflows webhook (`claude-notify setup teams <WEBHOOK_URL>`)
- Generic webhook notification backend (`claude-notify setup webhook <URL>`)
- `EMAIL_FROM`, `EMAIL_TO`, `EMAIL_SMTP_HOST`, `EMAIL_SMTP_PORT`, `EMAIL_SMTP_USERNAME`, `EMAIL_SMTP_PASSWORD` environment variable overrides
- `TEAMS_WEBHOOK_URL`, `WEBHOOK_URL` environment variable overrides

## [1.1.0] - 2026-03-14

### Added
- Pushbullet notification backend (`claude-notify setup pushbullet <API_TOKEN>`)
- `PUSHBULLET_API_TOKEN` environment variable override
- Install script for curl-based installation (`curl -sSL .../install.sh | sh`)

## [1.0.1] - 2026-03-14

### Fixed
- Fix clippy collapsible_if warnings for CI compatibility
- Fix release workflow: replace deprecated macos-13 with macos-latest for x86_64 cross-compilation

## [1.0.0] - 2026-03-14

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
