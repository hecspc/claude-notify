use crate::config::TelegramConfig;
use crate::notifier::Notifier;

pub struct TelegramNotifier {
    bot_token: String,
    chat_id: String,
}

impl TelegramNotifier {
    pub fn new(config: &TelegramConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let bot_token = config
            .bot_token
            .clone()
            .ok_or("telegram bot_token not configured")?;
        let chat_id = config
            .chat_id
            .clone()
            .ok_or("telegram chat_id not configured")?;
        Ok(Self { bot_token, chat_id })
    }
}

impl Notifier for TelegramNotifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );

        let body = serde_json::json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": "HTML",
            "disable_web_page_preview": true,
        });

        let response = ureq::post(&url).send_json(&body)?;

        if response.status() != 200 {
            let status = response.status();
            let body = response.into_body().read_to_string()?;
            return Err(format!("Telegram API error {}: {}", status, body).into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "telegram"
    }
}
