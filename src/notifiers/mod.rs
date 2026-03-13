pub mod telegram;

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
            other => {
                eprintln!("Warning: unknown backend '{}'", other);
            }
        }
    }

    notifiers
}
