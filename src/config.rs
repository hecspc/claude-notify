use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub backends: Vec<String>,
    pub events: Option<Vec<String>>,
    #[serde(default)]
    pub telegram: Option<TelegramConfig>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TelegramConfig {
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        let mut config = Self::load_from_file().unwrap_or_default();
        config.apply_env_overrides();

        if config.backends.is_empty() {
            config.backends = vec!["telegram".to_string()];
        }

        config
    }

    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("claude-notify")
            .join("config.toml")
    }

    fn load_from_file() -> Option<Self> {
        let path = Self::config_path();
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("NOTIFY_BACKEND") {
            self.backends = val.split(',').map(|s| s.trim().to_string()).collect();
        }

        if let Ok(val) = std::env::var("NOTIFY_EVENTS") {
            self.events = Some(val.split(',').map(|s| s.trim().to_string()).collect());
        }

        let tg = self.telegram.get_or_insert_with(TelegramConfig::default);
        if let Ok(val) = std::env::var("TELEGRAM_BOT_TOKEN") {
            tg.bot_token = Some(val);
        }
        if let Ok(val) = std::env::var("TELEGRAM_CHAT_ID") {
            tg.chat_id = Some(val);
        }
    }

    /// Returns true if the given event name should trigger a notification.
    pub fn should_notify(&self, event: &str) -> bool {
        match &self.events {
            None => true,
            Some(events) => events.iter().any(|e| e == event),
        }
    }
}
