use crate::config::EmailConfig;
use crate::notifier::Notifier;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub struct EmailNotifier {
    from: String,
    to: String,
    smtp_host: String,
    smtp_port: u16,
    credentials: Credentials,
}

impl EmailNotifier {
    pub fn new(config: &EmailConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let from = config.from.clone().ok_or("email from not configured")?;
        let to = config.to.clone().ok_or("email to not configured")?;
        let smtp_host = config
            .smtp_host
            .clone()
            .ok_or("email smtp_host not configured")?;
        let smtp_port = config.smtp_port.unwrap_or(587);
        let username = config
            .smtp_username
            .clone()
            .ok_or("email smtp_username not configured")?;
        let password = config
            .smtp_password
            .clone()
            .ok_or("email smtp_password not configured")?;

        let credentials = Credentials::new(username, password);

        Ok(Self {
            from,
            to,
            smtp_host,
            smtp_port,
            credentials,
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

impl Notifier for EmailNotifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let plain = html_to_plain(message);
        let mut lines = plain.splitn(2, '\n');
        let subject = lines.next().unwrap_or("claude-notify");
        let body = lines.next().unwrap_or("");

        let email = Message::builder()
            .from(self.from.parse()?)
            .to(self.to.parse()?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())?;

        let mailer = SmtpTransport::starttls_relay(&self.smtp_host)?
            .port(self.smtp_port)
            .credentials(self.credentials.clone())
            .build();

        mailer.send(&email)?;

        Ok(())
    }

    fn name(&self) -> &str {
        "email"
    }
}
