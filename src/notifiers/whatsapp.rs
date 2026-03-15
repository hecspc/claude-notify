use crate::config::WhatsappConfig;
use crate::notifier::Notifier;

pub struct WhatsappNotifier {
    phone_number_id: String,
    access_token: String,
    recipient: String,
}

impl WhatsappNotifier {
    pub fn new(config: &WhatsappConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let phone_number_id = config
            .phone_number_id
            .clone()
            .ok_or("whatsapp phone_number_id not configured")?;
        let access_token = config
            .access_token
            .clone()
            .ok_or("whatsapp access_token not configured")?;
        let recipient = config
            .recipient
            .clone()
            .ok_or("whatsapp recipient not configured")?;
        Ok(Self {
            phone_number_id,
            access_token,
            recipient,
        })
    }
}

fn html_to_plain(html: &str) -> String {
    html.replace("<b>", "*")
        .replace("</b>", "*")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

impl Notifier for WhatsappNotifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let text = html_to_plain(message);

        let url = format!(
            "https://graph.facebook.com/v21.0/{}/messages",
            self.phone_number_id
        );

        let body = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": self.recipient,
            "type": "text",
            "text": { "body": text }
        });

        let response = ureq::post(&url)
            .header("Authorization", &format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .send(body.to_string().as_bytes())?;

        if response.status() != 200 {
            let status = response.status();
            let body = response.into_body().read_to_string()?;
            return Err(format!("whatsapp error {}: {}", status, body).into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "whatsapp"
    }
}
