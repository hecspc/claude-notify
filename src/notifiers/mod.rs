pub mod desktop;
pub mod discord;
pub mod email;
pub mod ntfy;
pub mod pushbullet;
pub mod slack;
pub mod teams;
pub mod telegram;
pub mod webhook;

use crate::config::Config;
use crate::notifier::Notifier;

pub fn build_notifiers(config: &Config) -> Vec<Box<dyn Notifier>> {
    let mut notifiers: Vec<Box<dyn Notifier>> = Vec::new();

    for backend in &config.backends {
        match backend.as_str() {
            "telegram" => {
                if let Some(tg_config) = &config.telegram {
                    match telegram::TelegramNotifier::new(tg_config) {
                        Ok(n) => notifiers.push(Box::new(n)),
                        Err(e) => eprintln!("Warning: failed to init telegram: {}", e),
                    }
                } else {
                    eprintln!("Warning: telegram backend enabled but not configured");
                }
            }
            "slack" => {
                if let Some(slack_config) = &config.slack {
                    match slack::SlackNotifier::new(slack_config) {
                        Ok(n) => notifiers.push(Box::new(n)),
                        Err(e) => eprintln!("Warning: failed to init slack: {}", e),
                    }
                } else {
                    eprintln!("Warning: slack backend enabled but not configured");
                }
            }
            "desktop" => {
                notifiers.push(Box::new(desktop::DesktopNotifier::new()));
            }
            "discord" => {
                if let Some(discord_config) = &config.discord {
                    match discord::DiscordNotifier::new(discord_config) {
                        Ok(n) => notifiers.push(Box::new(n)),
                        Err(e) => eprintln!("Warning: failed to init discord: {}", e),
                    }
                } else {
                    eprintln!("Warning: discord backend enabled but not configured");
                }
            }
            "email" => {
                if let Some(email_config) = &config.email {
                    match email::EmailNotifier::new(email_config) {
                        Ok(n) => notifiers.push(Box::new(n)),
                        Err(e) => eprintln!("Warning: failed to init email: {}", e),
                    }
                } else {
                    eprintln!("Warning: email backend enabled but not configured");
                }
            }
            "ntfy" => {
                if let Some(ntfy_config) = &config.ntfy {
                    match ntfy::NtfyNotifier::new(ntfy_config) {
                        Ok(n) => notifiers.push(Box::new(n)),
                        Err(e) => eprintln!("Warning: failed to init ntfy: {}", e),
                    }
                } else {
                    eprintln!("Warning: ntfy backend enabled but not configured");
                }
            }
            "pushbullet" => {
                if let Some(pb_config) = &config.pushbullet {
                    match pushbullet::PushbulletNotifier::new(pb_config) {
                        Ok(n) => notifiers.push(Box::new(n)),
                        Err(e) => eprintln!("Warning: failed to init pushbullet: {}", e),
                    }
                } else {
                    eprintln!("Warning: pushbullet backend enabled but not configured");
                }
            }
            "teams" => {
                if let Some(teams_config) = &config.teams {
                    match teams::TeamsNotifier::new(teams_config) {
                        Ok(n) => notifiers.push(Box::new(n)),
                        Err(e) => eprintln!("Warning: failed to init teams: {}", e),
                    }
                } else {
                    eprintln!("Warning: teams backend enabled but not configured");
                }
            }
            "webhook" => {
                if let Some(wh_config) = &config.webhook {
                    match webhook::WebhookNotifier::new(wh_config) {
                        Ok(n) => notifiers.push(Box::new(n)),
                        Err(e) => eprintln!("Warning: failed to init webhook: {}", e),
                    }
                } else {
                    eprintln!("Warning: webhook backend enabled but not configured");
                }
            }
            other => {
                eprintln!("Warning: unknown backend '{}'", other);
            }
        }
    }

    notifiers
}
