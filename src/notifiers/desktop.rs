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
        } else {
            return Err("desktop notifications not supported on this platform".into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "desktop"
    }
}
