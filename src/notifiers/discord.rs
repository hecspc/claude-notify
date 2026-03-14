use crate::config::DiscordConfig;
use crate::notifier::Notifier;

pub struct DiscordNotifier {
    webhook_url: String,
}

impl DiscordNotifier {
    pub fn new(config: &DiscordConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let webhook_url = config
            .webhook_url
            .clone()
            .ok_or("discord webhook_url not configured")?;
        Ok(Self { webhook_url })
    }

    fn html_to_discord(html: &str) -> String {
        html.replace("<b>", "**")
            .replace("</b>", "**")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
    }
}

impl Notifier for DiscordNotifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let text = Self::html_to_discord(message);

        let body = serde_json::json!({
            "content": text,
        });

        let response = ureq::post(&self.webhook_url).send_json(&body)?;

        if response.status() != 204 {
            let status = response.status();
            let body = response.into_body().read_to_string()?;
            return Err(format!("Discord webhook error {}: {}", status, body).into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "discord"
    }
}
