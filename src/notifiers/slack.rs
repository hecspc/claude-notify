use crate::config::SlackConfig;
use crate::notifier::Notifier;

pub struct SlackNotifier {
    webhook_url: String,
}

impl SlackNotifier {
    pub fn new(config: &SlackConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let webhook_url = config
            .webhook_url
            .clone()
            .ok_or("slack webhook_url not configured")?;
        Ok(Self { webhook_url })
    }

    fn html_to_mrkdwn(html: &str) -> String {
        html.replace("<b>", "*")
            .replace("</b>", "*")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
    }
}

impl Notifier for SlackNotifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let text = Self::html_to_mrkdwn(message);

        let body = serde_json::json!({
            "text": text,
        });

        let response = ureq::post(&self.webhook_url).send_json(&body)?;

        if response.status() != 200 {
            let status = response.status();
            let body = response.into_body().read_to_string()?;
            return Err(format!("Slack webhook error {}: {}", status, body).into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "slack"
    }
}
