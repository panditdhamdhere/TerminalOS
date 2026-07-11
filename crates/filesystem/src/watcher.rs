use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use terminalos_shared::{Error, Result};

/// Filesystem change events.
#[derive(Debug, Clone)]
pub enum WatchEvent {
    Created(String),
    Modified(String),
    Removed(String),
    Rescan,
}

/// Watches a directory for changes using the notify crate.
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<WatchEvent>,
}

impl FileWatcher {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    for path in event.paths {
                        let path_str = path.display().to_string();
                        let watch_event = match event.kind {
                            notify::EventKind::Create(_) => WatchEvent::Created(path_str),
                            notify::EventKind::Modify(_) => WatchEvent::Modified(path_str),
                            notify::EventKind::Remove(_) => WatchEvent::Removed(path_str),
                            _ => WatchEvent::Rescan,
                        };
                        let _ = tx.send(watch_event);
                    }
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .map_err(|e| Error::Filesystem(format!("watcher init failed: {e}")))?;

        watcher
            .watch(path.as_ref(), RecursiveMode::Recursive)
            .map_err(|e| Error::Filesystem(format!("watch failed: {e}")))?;

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }

    /// Non-blocking poll for the next watch event.
    pub fn try_recv(&self) -> Option<WatchEvent> {
        self.receiver.try_recv().ok()
    }

    /// Returns a clone of the sender for external event injection.
    pub fn sender(&self) -> Sender<WatchEvent> {
        let (tx, _) = mpsc::channel();
        tx
    }
}
