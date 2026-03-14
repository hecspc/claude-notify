use crate::config::PushbulletConfig;
use crate::notifier::Notifier;

pub struct PushbulletNotifier {
    api_token: String,
}

impl PushbulletNotifier {
    pub fn new(config: &PushbulletConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let api_token = config
            .api_token
            .clone()
            .ok_or("pushbullet api_token not configured")?;
        Ok(Self { api_token })
    }
}

fn html_to_plain(html: &str) -> String {
    html.replace("<b>", "")
        .replace("</b>", "")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

impl Notifier for PushbulletNotifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let plain = html_to_plain(message);
        let mut lines = plain.splitn(2, '\n');
        let title = lines.next().unwrap_or("claude-notify");
        let body = lines.next().unwrap_or("");

        let payload = serde_json::json!({
            "type": "note",
            "title": title,
            "body": body,
        });

        let response = ureq::post("https://api.pushbullet.com/v2/pushes")
            .header("Access-Token", &self.api_token)
            .send_json(&payload)?;

        if response.status() != 200 {
            let status = response.status();
            let body = response.into_body().read_to_string()?;
            return Err(format!("Pushbullet API error {}: {}", status, body).into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "pushbullet"
    }
}
