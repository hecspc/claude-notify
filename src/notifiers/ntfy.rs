use crate::config::NtfyConfig;
use crate::notifier::Notifier;

pub struct NtfyNotifier {
    topic_url: String,
}

impl NtfyNotifier {
    pub fn new(config: &NtfyConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let topic_url = config
            .topic_url
            .clone()
            .ok_or("ntfy topic_url not configured")?;
        Ok(Self { topic_url })
    }
}

fn html_to_plain(html: &str) -> String {
    html.replace("<b>", "")
        .replace("</b>", "")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

impl Notifier for NtfyNotifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let plain = html_to_plain(message);
        let mut lines = plain.splitn(2, '\n');
        let title = lines.next().unwrap_or("claude-notify");
        let body = lines.next().unwrap_or("");

        let response = ureq::post(&self.topic_url)
            .header("Title", title)
            .header("Content-Type", "text/plain")
            .send(body.as_bytes())?;

        if response.status() != 200 {
            let status = response.status();
            let body = response.into_body().read_to_string()?;
            return Err(format!("ntfy error {}: {}", status, body).into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "ntfy"
    }
}
