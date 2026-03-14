use crate::SetupBackend;
use std::path::PathBuf;

pub enum Scope {
    User,
    Project,
}

fn settings_path(scope: &Scope) -> PathBuf {
    match scope {
        Scope::User => {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".claude").join("settings.json")
        }
        Scope::Project => PathBuf::from(".claude").join("settings.json"),
    }
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("claude-notify")
        .join("config.toml")
}

fn write_backend_config(backend: &SetupBackend) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path();

    // Load existing config or start fresh
    let mut config: toml::Table = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        content.parse()?
    } else {
        toml::Table::new()
    };

    match backend {
        SetupBackend::Telegram { bot_token, chat_id } => {
            // Set backends to include telegram
            let backends = config
                .entry("backends")
                .or_insert(toml::Value::Array(vec![]));
            if let toml::Value::Array(arr) = backends {
                let tg = toml::Value::String("telegram".to_string());
                if !arr.contains(&tg) {
                    arr.push(tg);
                }
            }

            // Set telegram config
            let mut tg_table = toml::Table::new();
            tg_table.insert(
                "bot_token".to_string(),
                toml::Value::String(bot_token.clone()),
            );
            tg_table.insert(
                "chat_id".to_string(),
                toml::Value::String(chat_id.clone()),
            );
            config.insert("telegram".to_string(), toml::Value::Table(tg_table));
        }
        SetupBackend::Slack { webhook_url } => {
            // Set backends to include slack
            let backends = config
                .entry("backends")
                .or_insert(toml::Value::Array(vec![]));
            if let toml::Value::Array(arr) = backends {
                let slack = toml::Value::String("slack".to_string());
                if !arr.contains(&slack) {
                    arr.push(slack);
                }
            }

            // Set slack config
            let mut slack_table = toml::Table::new();
            slack_table.insert(
                "webhook_url".to_string(),
                toml::Value::String(webhook_url.clone()),
            );
            config.insert("slack".to_string(), toml::Value::Table(slack_table));
        }
        SetupBackend::Desktop => {
            let backends = config
                .entry("backends")
                .or_insert(toml::Value::Array(vec![]));
            if let toml::Value::Array(arr) = backends {
                let desktop = toml::Value::String("desktop".to_string());
                if !arr.contains(&desktop) {
                    arr.push(desktop);
                }
            }
        }
        SetupBackend::Discord { webhook_url } => {
            let backends = config
                .entry("backends")
                .or_insert(toml::Value::Array(vec![]));
            if let toml::Value::Array(arr) = backends {
                let discord = toml::Value::String("discord".to_string());
                if !arr.contains(&discord) {
                    arr.push(discord);
                }
            }

            let mut discord_table = toml::Table::new();
            discord_table.insert(
                "webhook_url".to_string(),
                toml::Value::String(webhook_url.clone()),
            );
            config.insert("discord".to_string(), toml::Value::Table(discord_table));
        }
        SetupBackend::Ntfy { topic_url } => {
            let backends = config
                .entry("backends")
                .or_insert(toml::Value::Array(vec![]));
            if let toml::Value::Array(arr) = backends {
                let ntfy = toml::Value::String("ntfy".to_string());
                if !arr.contains(&ntfy) {
                    arr.push(ntfy);
                }
            }

            let mut ntfy_table = toml::Table::new();
            ntfy_table.insert(
                "topic_url".to_string(),
                toml::Value::String(topic_url.clone()),
            );
            config.insert("ntfy".to_string(), toml::Value::Table(ntfy_table));
        }
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, config.to_string())?;
    println!("Config written to {}", path.display());

    Ok(())
}

fn write_hooks(scope: &Scope) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings_path(scope);

    let mut settings: serde_json::Value = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    let obj = settings
        .as_object_mut()
        .ok_or("settings.json is not an object")?;

    if obj.contains_key("hooks") {
        let hooks = obj.get("hooks").unwrap();
        let has_notify = hooks.to_string().contains("claude-notify");
        if has_notify {
            println!(
                "claude-notify hooks already configured in {}",
                path.display()
            );
            return Ok(());
        }
    }

    let hooks = obj
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let hooks_obj = hooks
        .as_object_mut()
        .ok_or("hooks is not an object")?;

    hooks_obj.insert(
        "Notification".to_string(),
        serde_json::json!([{
            "matcher": "permission_prompt|idle_prompt|elicitation_dialog",
            "hooks": [{ "type": "command", "command": "claude-notify", "async": true }]
        }]),
    );

    hooks_obj.insert(
        "Stop".to_string(),
        serde_json::json!([{
            "hooks": [{ "type": "command", "command": "claude-notify", "async": true }]
        }]),
    );

    hooks_obj.insert(
        "TaskCompleted".to_string(),
        serde_json::json!([{
            "hooks": [{ "type": "command", "command": "claude-notify", "async": true }]
        }]),
    );

    let content = serde_json::to_string_pretty(&settings)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)?;
    println!("Hooks configured in {}", path.display());

    Ok(())
}

pub fn run_setup(
    backend: &SetupBackend,
    scope: Scope,
) -> Result<(), Box<dyn std::error::Error>> {
    write_backend_config(backend)?;
    write_hooks(&scope)?;

    let scope_label = match scope {
        Scope::User => "user (~/.claude/settings.json)",
        Scope::Project => "project (.claude/settings.json)",
    };
    println!("\nSetup complete ({}).", scope_label);
    println!("Make sure claude-notify is in your PATH (e.g. ~/.local/bin/).");

    Ok(())
}
