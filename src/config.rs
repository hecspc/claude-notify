use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub backends: Vec<String>,
    pub events: Option<Vec<String>>,
    #[serde(default)]
    pub telegram: Option<TelegramConfig>,
    #[serde(default)]
    pub slack: Option<SlackConfig>,
    #[serde(default)]
    pub discord: Option<DiscordConfig>,
    #[serde(default)]
    pub ntfy: Option<NtfyConfig>,
    #[serde(default)]
    pub pushbullet: Option<PushbulletConfig>,
    #[serde(default)]
    pub webhook: Option<WebhookConfig>,
    #[serde(default)]
    pub teams: Option<TeamsConfig>,
    #[serde(default)]
    pub email: Option<EmailConfig>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TelegramConfig {
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SlackConfig {
    pub webhook_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DiscordConfig {
    pub webhook_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct NtfyConfig {
    pub topic_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct PushbulletConfig {
    pub api_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct WebhookConfig {
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TeamsConfig {
    pub webhook_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct EmailConfig {
    pub from: Option<String>,
    pub to: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
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

    pub fn config_path() -> PathBuf {
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

        if let Ok(val) = std::env::var("SLACK_WEBHOOK_URL") {
            let slack = self.slack.get_or_insert_with(SlackConfig::default);
            slack.webhook_url = Some(val);
        }

        if let Ok(val) = std::env::var("DISCORD_WEBHOOK_URL") {
            let discord = self.discord.get_or_insert_with(DiscordConfig::default);
            discord.webhook_url = Some(val);
        }

        if let Ok(val) = std::env::var("NTFY_TOPIC_URL") {
            let ntfy = self.ntfy.get_or_insert_with(NtfyConfig::default);
            ntfy.topic_url = Some(val);
        }

        if let Ok(val) = std::env::var("PUSHBULLET_API_TOKEN") {
            let pb = self.pushbullet.get_or_insert_with(PushbulletConfig::default);
            pb.api_token = Some(val);
        }

        if let Ok(val) = std::env::var("WEBHOOK_URL") {
            let wh = self.webhook.get_or_insert_with(WebhookConfig::default);
            wh.url = Some(val);
        }

        if let Ok(val) = std::env::var("TEAMS_WEBHOOK_URL") {
            let teams = self.teams.get_or_insert_with(TeamsConfig::default);
            teams.webhook_url = Some(val);
        }

        let has_email_env = std::env::var("EMAIL_SMTP_HOST").is_ok();
        if has_email_env {
            let email = self.email.get_or_insert_with(EmailConfig::default);
            if let Ok(val) = std::env::var("EMAIL_FROM") {
                email.from = Some(val);
            }
            if let Ok(val) = std::env::var("EMAIL_TO") {
                email.to = Some(val);
            }
            if let Ok(val) = std::env::var("EMAIL_SMTP_HOST") {
                email.smtp_host = Some(val);
            }
            if let Ok(val) = std::env::var("EMAIL_SMTP_PORT") {
                email.smtp_port = val.parse().ok();
            }
            if let Ok(val) = std::env::var("EMAIL_SMTP_USERNAME") {
                email.smtp_username = Some(val);
            }
            if let Ok(val) = std::env::var("EMAIL_SMTP_PASSWORD") {
                email.smtp_password = Some(val);
            }
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
