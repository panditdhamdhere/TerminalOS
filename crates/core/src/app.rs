use std::sync::Arc;

use terminalos_config::AppConfig;
use terminalos_shared::{LogEntry, Theme};
use tokio::sync::RwLock;

use crate::traits::EventBus;

/// Shared application context injected into services and the UI layer.
#[derive(Clone)]
pub struct AppContext {
    pub config: Arc<RwLock<AppConfig>>,
    pub theme: Arc<RwLock<Theme>>,
    pub logs: Arc<RwLock<Vec<LogEntry>>>,
    pub events: Arc<dyn EventBus>,
}

impl AppContext {
    pub fn new(config: AppConfig, events: Arc<dyn EventBus>) -> Self {
        let theme = match config.ui.theme {
            terminalos_shared::ThemeMode::Dark => Theme::dark(),
            terminalos_shared::ThemeMode::Light => Theme::light(),
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            theme: Arc::new(RwLock::new(theme)),
            logs: Arc::new(RwLock::new(Vec::new())),
            events,
        }
    }

    pub async fn push_log(&self, entry: LogEntry) {
        let mut logs = self.logs.write().await;
        logs.push(entry);
        if logs.len() > 500 {
            let drain = logs.len() - 500;
            logs.drain(0..drain);
        }
    }
}
