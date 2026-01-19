use crate::api::{create_router, AppState};
use crate::watcher::FileWatcher;
use gitent_core::{Session, Storage};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::info;

pub struct GitentServer {
    session: Session,
    storage: Arc<Mutex<Storage>>,
    _watcher: FileWatcher,
}

impl GitentServer {
    pub fn new(root_path: PathBuf, db_path: PathBuf) -> anyhow::Result<Self> {
        let session = Session::new(root_path);
        let storage = Arc::new(Mutex::new(Storage::new(db_path)?));

        {
            let storage_guard = storage.lock().unwrap();
            storage_guard.create_session(&session)?;
        }

        let watcher = FileWatcher::new(&session, Arc::clone(&storage))?;

        Ok(Self {
            session,
            storage,
            _watcher: watcher,
        })
    }

    pub async fn serve(self, addr: SocketAddr) -> anyhow::Result<()> {
        let state = AppState {
            storage: self.storage,
        };

        let app = create_router(state);

        info!("Server listening on {}", addr);
        info!("Session ID: {}", self.session.id);
        info!("Watching: {:?}", self.session.root_path);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    pub fn session_id(&self) -> uuid::Uuid {
        self.session.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_server_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_dir = TempDir::new().unwrap();

        let db_path = db_dir.path().join("test.db");
        let server = GitentServer::new(temp_dir.path().to_path_buf(), db_path);

        assert!(server.is_ok());
    }
}
