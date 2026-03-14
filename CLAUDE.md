# claude-notify

## Project Overview

`claude-notify` is a Rust CLI that receives Claude Code hook events via stdin and dispatches notifications to configurable backends (Telegram, Slack, Desktop, Discord, ntfy). It compiles to a single native binary with no runtime dependencies.

Requires Rust edition 2024 (rustc 1.85+).

## Build & Run

```bash
cargo build                                                     # dev build
cargo build --release                                           # release build (output: target/release/claude-notify)
cargo run -- --dry-run                                          # run with dry-run flag (pipe JSON to stdin)
cargo run -- setup telegram <BOT_TOKEN> <CHAT_ID>               # configure credentials + hooks (user-level)
cargo run -- setup telegram <BOT_TOKEN> <CHAT_ID> --project     # configure hooks in current project
cargo run -- setup slack <WEBHOOK_URL>                          # configure Slack notifications
cargo run -- setup desktop                                      # configure native OS notifications
cargo run -- setup discord <WEBHOOK_URL>                        # configure Discord notifications
cargo run -- setup ntfy <TOPIC_URL>                             # configure ntfy notifications
cargo run -- use desktop                                        # switch active backend(s)
cargo run -- use desktop,slack                                  # multiple backends
```

There are no tests yet. Verify changes with `cargo build` and manual dry-run testing:

```bash
echo '{"session_id":"abc","cwd":"/tmp","hook_event_name":"Notification","notification_type":"permission_prompt","tool_name":"Bash","tool_input":{"command":"ls"}}' | cargo run -- --dry-run
```

## Code Structure

```
src/
  main.rs           — CLI entry point (clap subcommands). Routes to setup, mute/unmute/status, --dry-run, or stdin→format→send
  types.rs          — HookEvent struct (serde). All optional fields use Option<T>
  config.rs         — Config + per-backend config structs. Loads ~/.config/claude-notify/config.toml, env vars override
  formatter.rs      — format_message() maps HookEvent → HTML string. friendly_name() hashes session_id to adjective-noun pair
  notifier.rs       — Notifier trait (send + name)
  notifiers/
    mod.rs          — build_notifiers() registry: config → Vec<Box<dyn Notifier>>
    telegram.rs     — TelegramNotifier: ureq POST to Telegram Bot API with HTML parse mode
    slack.rs        — SlackNotifier: ureq POST to Slack Incoming Webhook, converts HTML→mrkdwn
    desktop.rs      — DesktopNotifier: osascript (macOS) / notify-send (Linux), zero-config
    discord.rs      — DiscordNotifier: ureq POST to Discord webhook, expects 204
    ntfy.rs         — NtfyNotifier: ureq POST plain text with Title header
  setup.rs          — run_setup() writes backend config + merges hooks into settings.json (--user or --project scope)
```

## Runtime File Paths

- `~/.config/claude-notify/config.toml` — backend credentials + event filter (written by `setup`, read at runtime)
- `~/.config/claude-notify/muted/` — mute state: `_global` file = all muted, session name/UUID files = per-session mute
- `~/.claude/settings.json` — user-level hooks (`--user` scope, default)
- `.claude/settings.json` — project-level hooks (`--project` scope)

## Key Conventions

- **`SetupBackend` enum** is defined in `main.rs` and imported by `setup.rs` via `crate::SetupBackend` — new backends need a variant here
- **HTML parse mode** for Telegram — escape only `< > &` (not MarkdownV2)
- **Blocking HTTP** via `ureq` — no async runtime, keeps binary small
- **Errors to stderr** — hooks are async so stderr is invisible to Claude Code users
- **Event filtering** uses lowercase keys: `permission_prompt`, `idle_prompt`, `elicitation_dialog`, `stop`, `task_completed`
- **Config layering**: TOML file first, then env var overrides

## Adding a New Notification Backend

1. Create `src/notifiers/newbackend.rs` implementing `Notifier` trait
2. Add config fields to `Config` in `config.rs`
3. Add `pub mod newbackend;` and a match arm in `notifiers/mod.rs`
4. Add env var overrides in `config.rs` `apply_env_overrides()`
5. Add a variant to `SetupBackend` enum in `main.rs` for `setup` subcommand support
6. Add config writing logic in `setup.rs` `write_backend_config()`

## Design Docs

- `docs/ai-specs/notification/design.md` — original requirements
- `docs/ai-specs/notification/plan.md` — architecture and implementation plan
- `docs/ai-specs/notification/implementation.md` — full source reference
