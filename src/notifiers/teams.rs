use crate::config::TeamsConfig;
use crate::notifier::Notifier;

pub struct TeamsNotifier {
    webhook_url: String,
}

impl TeamsNotifier {
    pub fn new(config: &TeamsConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let webhook_url = config
            .webhook_url
            .clone()
            .ok_or("teams webhook_url not configured")?;
        Ok(Self { webhook_url })
    }
}

fn html_to_plain(html: &str) -> String {
    html.replace("<b>", "**")
        .replace("</b>", "**")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

impl Notifier for TeamsNotifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let text = html_to_plain(message);

        // Adaptive Card format for Teams Workflows webhooks
        let payload = serde_json::json!({
            "type": "message",
            "attachments": [{
                "contentType": "application/vnd.microsoft.card.adaptive",
                "content": {
                    "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
                    "type": "AdaptiveCard",
                    "version": "1.4",
                    "body": [{
                        "type": "TextBlock",
                        "text": text,
                        "wrap": true
                    }]
                }
            }]
        });

        let response = ureq::post(&self.webhook_url).send_json(&payload)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.into_body().read_to_string()?;
            return Err(format!("Teams webhook error {}: {}", status, body).into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "teams"
    }
}
