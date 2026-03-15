use crate::config::OpenclawConfig;
use crate::notifier::Notifier;

pub struct OpenclawNotifier {
    gateway_url: String,
    token: String,
    target: String,
    channel: Option<String>,
}

impl OpenclawNotifier {
    pub fn new(config: &OpenclawConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let gateway_url = config
            .gateway_url
            .clone()
            .ok_or("openclaw gateway_url not configured")?;
        let token = config
            .token
            .clone()
            .ok_or("openclaw token not configured")?;
        let target = config
            .target
            .clone()
            .ok_or("openclaw target not configured")?;
        Ok(Self {
            gateway_url: gateway_url.trim_end_matches('/').to_string(),
            token,
            target,
            channel: config.channel.clone(),
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

impl Notifier for OpenclawNotifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let text = html_to_plain(message);
        let url = format!("{}/tools/invoke", self.gateway_url);

        let mut args = serde_json::json!({
            "to": self.target,
            "message": text,
            "deliver": true,
        });

        if let Some(ch) = &self.channel {
            args["channel"] = serde_json::Value::String(ch.clone());
        }

        let body = serde_json::json!({
            "tool": "agent_send",
            "args": args,
        });

        let response = ureq::post(&url)
            .header("Authorization", &format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .send(body.to_string().as_bytes())?;

        if response.status() != 200 {
            let status = response.status();
            let body = response.into_body().read_to_string()?;
            return Err(format!("openclaw error {}: {}", status, body).into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "openclaw"
    }
}
