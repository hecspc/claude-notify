use crate::config::DesktopConfig;
use crate::notifier::Notifier;
use std::process::Command;

pub struct DesktopNotifier {
    activate_bundle_id: Option<String>,
    execute: Option<String>,
}

impl DesktopNotifier {
    pub fn new(config: Option<&DesktopConfig>) -> Self {
        Self {
            activate_bundle_id: config.and_then(|c| c.activate_bundle_id.clone()),
            execute: config.and_then(|c| c.execute.clone()),
        }
    }
}

fn html_to_plain(html: &str) -> String {
    html.replace("<b>", "")
        .replace("</b>", "")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

#[cfg(target_os = "macos")]
fn terminal_notifier_path() -> Option<String> {
    let out = Command::new("/usr/bin/which")
        .arg("terminal-notifier")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let path = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if path.is_empty() { None } else { Some(path) }
}

impl Notifier for DesktopNotifier {
    fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let plain = html_to_plain(message);
        let mut lines = plain.splitn(2, '\n');
        let title = lines.next().unwrap_or("claude-notify");
        let body = lines.next().unwrap_or("");

        if cfg!(target_os = "macos") {
            #[cfg(target_os = "macos")]
            if let Some(tn) = terminal_notifier_path() {
                let mut cmd = Command::new(&tn);
                cmd.arg("-title").arg(title)
                    .arg("-message").arg(body)
                    .arg("-group").arg("claude-notify");

                // execute wins over activate
                if let Some(exec) = &self.execute {
                    cmd.arg("-execute").arg(exec);
                } else {
                    let bundle = self
                        .activate_bundle_id
                        .clone()
                        .unwrap_or_else(|| "com.apple.Terminal".to_string());
                    cmd.arg("-activate").arg(&bundle).arg("-sender").arg(&bundle);
                }

                let status = cmd.status()?;
                if !status.success() {
                    return Err("terminal-notifier failed".into());
                }
                return Ok(());
            }

            // Fallback: osascript. Click opens Script Editor (no way to override).
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
            if self.activate_bundle_id.is_some() || self.execute.is_some() {
                eprintln!(
                    "claude-notify: install `terminal-notifier` (brew install terminal-notifier) \
                     to make desktop notification clicks honor activate_bundle_id/execute."
                );
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
