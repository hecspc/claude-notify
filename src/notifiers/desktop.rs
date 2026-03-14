use crate::notifier::Notifier;
use std::process::Command;

pub struct DesktopNotifier;

impl DesktopNotifier {
    pub fn new() -> Self {
        Self
    }
}

fn html_to_plain(html: &str) -> String {
    html.replace("<b>", "")
        .replace("</b>", "")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

impl Notifier for DesktopNotifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let plain = html_to_plain(message);
        let mut lines = plain.splitn(2, '\n');
        let title = lines.next().unwrap_or("claude-notify");
        let body = lines.next().unwrap_or("");

        if cfg!(target_os = "macos") {
            let script = format!(
                "display notification \"{}\" with title \"{}\"",
                body.replace('\\', "\\\\").replace('"', "\\\""),
                title.replace('\\', "\\\\").replace('"', "\\\""),
            );
            let status = Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .status()?;
            if !status.success() {
                return Err("osascript failed".into());
            }
        } else if cfg!(target_os = "linux") {
            let status = Command::new("notify-send")
                .arg(title)
                .arg(body)
                .status()?;
            if !status.success() {
                return Err("notify-send failed".into());
            }
        } else if cfg!(target_os = "windows") {
            let ps_title = title.replace('\'', "''");
            let ps_body = body.replace('\'', "''");
            let script = format!(
                "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null; \
                 $xml = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent(1); \
                 $text = $xml.GetElementsByTagName('text'); \
                 $text[0].AppendChild($xml.CreateTextNode('{ps_title}')) | Out-Null; \
                 $text[1].AppendChild($xml.CreateTextNode('{ps_body}')) | Out-Null; \
                 $toast = [Windows.UI.Notifications.ToastNotification]::new($xml); \
                 [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('claude-notify').Show($toast)"
            );
            let status = Command::new("powershell")
                .arg("-NoProfile")
                .arg("-Command")
                .arg(&script)
                .status()?;
            if !status.success() {
                return Err("powershell toast notification failed".into());
            }
        } else {
            return Err("desktop notifications not supported on this platform".into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "desktop"
    }
}
