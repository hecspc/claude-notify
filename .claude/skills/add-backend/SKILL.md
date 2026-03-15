---
name: add-backend
description: "Scaffold a new notification backend following the project's 6-step checklist in CLAUDE.md. Takes a backend name as argument."
---

# Add Notification Backend

Scaffold a new notification backend by following the 6-step checklist from CLAUDE.md. The user will provide the backend name (e.g., "discord", "pagerduty", "teams").

## Steps

### 1. Create `src/notifiers/{backend}.rs`

Create a new file implementing the `Notifier` trait. Follow the pattern from `src/notifiers/telegram.rs` or `src/notifiers/slack.rs`:

- Define a `{Backend}Notifier` struct with required fields
- Implement `new(config: &{Backend}Config) -> Result<Self, Box<dyn std::error::Error>>` that validates config
- Implement `Notifier` trait: `send()` and `name()`
- Use `ureq` for HTTP requests (blocking, no async)
- Convert HTML message format to whatever the backend needs (like Slack's `html_to_mrkdwn`)

### 2. Add config struct to `src/config.rs`

Add after existing config structs:
```rust
#[derive(Debug, Deserialize, Clone, Default)]
pub struct {Backend}Config {
    // backend-specific fields as Option<String>
}
```

Add field to `Config` struct:
```rust
#[serde(default)]
pub {backend}: Option<{Backend}Config>,
```

### 3. Register in `src/notifiers/mod.rs`

- Add `pub mod {backend};`
- Add match arm for `"{backend}"` in `build_notifiers()` following the telegram/slack pattern

### 4. Add env var overrides in `src/config.rs`

In `apply_env_overrides()`, add environment variable overrides for the new backend's config fields. Use the pattern `{BACKEND}_FIELD_NAME`.

### 5. Add `SetupBackend` variant in `src/main.rs`

Add a new variant to the `SetupBackend` enum with clap doc comments and arguments for the required setup parameters.

### 6. Add config writing in `src/setup.rs`

Add a `SetupBackend::{Backend}` branch in `write_backend_config()` that:
- Adds the backend name to the `backends` array (if not already present)
- Writes the backend config section to TOML

### 7. Add plugin skill in `plugin/skills/setup-{backend}/SKILL.md`

Create a new skill directory and SKILL.md for the plugin. Follow the pattern from existing setup skills (e.g. `plugin/skills/setup-telegram/SKILL.md`):

- Frontmatter: `name: setup-{backend}`, `description` explaining what it configures
- Instructions for how to get credentials
- The `claude-notify setup {backend} <ARGS>` command to run
- Example usage

### 8. Update `plugin/README.md`

Add a row to the "Setup Commands" table:
```
| `/claude-notify:setup-{backend}` | Configure {Backend} notifications |
```

### 9. Update `.claude-plugin/plugin.json`

Add the new backend name to the `description` field so marketplace listings are accurate.

## After Scaffolding

- Run `cargo build` to verify compilation
- Run `/dry-run` to verify existing functionality isn't broken
- Update CLAUDE.md, README.md, and CHANGELOG.md with the new backend
- Update `docs/ai-specs/implementation.md`
