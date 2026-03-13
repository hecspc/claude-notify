use std::path::PathBuf;

fn settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".claude").join("settings.json")
}

pub fn run_setup() -> Result<(), Box<dyn std::error::Error>> {
    let path = settings_path();

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
            println!("claude-notify hooks are already configured in {}", path.display());
            println!("Remove the existing hooks first if you want to reconfigure.");
            return Ok(());
        }
    }

    let hooks = obj
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let hooks_obj = hooks
        .as_object_mut()
        .ok_or("hooks is not an object")?;

    // Notification hook with matcher
    hooks_obj.insert(
        "Notification".to_string(),
        serde_json::json!([{
            "matcher": "permission_prompt|idle_prompt|elicitation_dialog",
            "hooks": [{ "type": "command", "command": "claude-notify", "async": true }]
        }]),
    );

    // Stop hook
    hooks_obj.insert(
        "Stop".to_string(),
        serde_json::json!([{
            "hooks": [{ "type": "command", "command": "claude-notify", "async": true }]
        }]),
    );

    // TaskCompleted hook
    hooks_obj.insert(
        "TaskCompleted".to_string(),
        serde_json::json!([{
            "hooks": [{ "type": "command", "command": "claude-notify", "async": true }]
        }]),
    );

    // Write back with pretty formatting
    let content = serde_json::to_string_pretty(&settings)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)?;

    println!("Hooks configured in {}", path.display());
    println!("\nMake sure claude-notify is in your PATH (e.g. ~/.local/bin/)");
    println!("and that ~/.config/claude-notify/config.toml has your Telegram credentials.");

    Ok(())
}
