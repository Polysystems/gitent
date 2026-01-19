use gitent_core::{Change, ChangeType, Session, Storage};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info};
use uuid::Uuid;

pub struct FileWatcher {
    _session_id: Uuid,
    _storage: Arc<Mutex<Storage>>,
    _debouncer: Debouncer<notify::RecommendedWatcher, FileIdMap>,
}

impl FileWatcher {
    pub fn new(session: &Session, storage: Arc<Mutex<Storage>>) -> anyhow::Result<Self> {
        let session_id = session.id;
        let root_path = session.root_path.clone();
        let root_path_for_watch = root_path.clone();
        let ignore_patterns = session.ignore_patterns.clone();
        let storage_clone = Arc::clone(&storage);

        let (tx, mut rx) = mpsc::channel(100);

        let debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            move |result: DebounceEventResult| {
                if let Err(e) = tx.blocking_send(result) {
                    error!("Failed to send event: {}", e);
                }
            },
        )?;

        let mut watcher = Self {
            _session_id: session_id,
            _storage: storage,
            _debouncer: debouncer,
        };

        watcher
            ._debouncer
            .watcher()
            .watch(&root_path_for_watch, RecursiveMode::Recursive)?;

        info!("File watcher started for {:?}", root_path);

        tokio::spawn(async move {
            while let Some(result) = rx.recv().await {
                match result {
                    Ok(events) => {
                        for event in events {
                            if let Err(e) = Self::handle_event(
                                event.event,
                                session_id,
                                &root_path,
                                &ignore_patterns,
                                &storage_clone,
                            ) {
                                error!("Error handling event: {}", e);
                            }
                        }
                    }
                    Err(errors) => {
                        for error in errors {
                            error!("Watch error: {:?}", error);
                        }
                    }
                }
            }
        });

        Ok(watcher)
    }

    fn handle_event(
        event: Event,
        session_id: Uuid,
        root_path: &Path,
        ignore_patterns: &[String],
        storage: &Arc<Mutex<Storage>>,
    ) -> anyhow::Result<()> {
        for path in event.paths {
            if Self::should_ignore(&path, root_path, ignore_patterns) {
                continue;
            }

            let change = match event.kind {
                EventKind::Create(_) => {
                    info!("File created: {:?}", path);
                    let content = std::fs::read(&path).ok();
                    let mut change = Change::new(ChangeType::Create, path.clone(), session_id);
                    if let Some(content) = content {
                        change = change.with_content_after(content);
                    }
                    Some(change)
                }
                EventKind::Modify(_) => {
                    info!("File modified: {:?}", path);
                    let content_after = std::fs::read(&path).ok();
                    let mut change = Change::new(ChangeType::Modify, path.clone(), session_id);
                    if let Some(content) = content_after {
                        change = change.with_content_after(content);
                    }
                    Some(change)
                }
                EventKind::Remove(_) => {
                    info!("File removed: {:?}", path);
                    Some(Change::new(ChangeType::Delete, path.clone(), session_id))
                }
                _ => None,
            };

            if let Some(change) = change {
                let storage = storage.lock().unwrap();
                storage.create_change(&change)?;
            }
        }

        Ok(())
    }

    fn should_ignore(path: &Path, root_path: &Path, ignore_patterns: &[String]) -> bool {
        let relative_path = path.strip_prefix(root_path).unwrap_or(path);
        let path_str = relative_path.to_string_lossy();

        for pattern in ignore_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitent_core::Session;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let session = Session::new(temp_dir.path().to_path_buf());
        let storage = Arc::new(Mutex::new(Storage::in_memory().unwrap()));

        storage.lock().unwrap().create_session(&session).unwrap();

        let _watcher = FileWatcher::new(&session, storage).unwrap();

        // Just verify it doesn't panic
    }

    #[test]
    fn test_should_ignore() {
        let root = PathBuf::from("/test");
        let ignore_patterns = vec!["target".to_string(), ".git".to_string()];

        assert!(FileWatcher::should_ignore(
            &PathBuf::from("/test/target/debug"),
            &root,
            &ignore_patterns
        ));

        assert!(FileWatcher::should_ignore(
            &PathBuf::from("/test/.git/config"),
            &root,
            &ignore_patterns
        ));

        assert!(!FileWatcher::should_ignore(
            &PathBuf::from("/test/src/main.rs"),
            &root,
            &ignore_patterns
        ));
    }
}
