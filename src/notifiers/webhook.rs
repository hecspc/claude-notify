use crate::config::WebhookConfig;
use crate::notifier::Notifier;
use std::collections::HashMap;

pub struct WebhookNotifier {
    url: String,
    headers: HashMap<String, String>,
    display_name: String,
}

impl WebhookNotifier {
    pub fn new(config: &WebhookConfig, name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let url = config.url.clone().ok_or("webhook url not configured")?;
        let headers = config.headers.clone().unwrap_or_default();
        Ok(Self {
            url,
            headers,
            display_name: name.to_string(),
        })
    }
}

fn html_to_plain(html: &str) -> String {
    html.replace("<b>", "")
        .replace("</b>", "")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

impl Notifier for WebhookNotifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let plain = html_to_plain(message);
        let mut lines = plain.splitn(2, '\n');
        let title = lines.next().unwrap_or("claude-notify");
        let body = lines.next().unwrap_or("");

        let payload = serde_json::json!({
            "title": title,
            "body": body,
            "text": plain,
        });

        let mut req = ureq::post(&self.url);
        for (key, value) in &self.headers {
            req = req.header(key, value);
        }
        let response = req.send_json(&payload)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.into_body().read_to_string()?;
            return Err(format!("Webhook '{}' error {}: {}", self.display_name, status, body).into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.display_name
    }
}
