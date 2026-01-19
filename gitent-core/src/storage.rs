use crate::error::{Error, Result};
use crate::models::{Change, ChangeType, Commit, CommitInfo, Session};
use chrono::DateTime;
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::path::{Path, PathBuf};
use uuid::Uuid;

const SCHEMA_VERSION: i32 = 1;

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let mut storage = Self { conn };
        storage.initialize()?;
        Ok(storage)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let mut storage = Self { conn };
        storage.initialize()?;
        Ok(storage)
    }

    fn initialize(&mut self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                root_path TEXT NOT NULL,
                started TEXT NOT NULL,
                ended TEXT,
                active INTEGER NOT NULL,
                ignore_patterns TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS changes (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                change_type TEXT NOT NULL,
                path TEXT NOT NULL,
                old_path TEXT,
                content_before BLOB,
                content_after BLOB,
                content_hash_before TEXT,
                content_hash_after TEXT,
                agent_id TEXT,
                metadata TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            );

            CREATE TABLE IF NOT EXISTS commits (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                parent TEXT,
                timestamp TEXT NOT NULL,
                message TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                metadata TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id),
                FOREIGN KEY (parent) REFERENCES commits(id)
            );

            CREATE TABLE IF NOT EXISTS commit_changes (
                commit_id TEXT NOT NULL,
                change_id TEXT NOT NULL,
                PRIMARY KEY (commit_id, change_id),
                FOREIGN KEY (commit_id) REFERENCES commits(id),
                FOREIGN KEY (change_id) REFERENCES changes(id)
            );

            CREATE INDEX IF NOT EXISTS idx_changes_session ON changes(session_id);
            CREATE INDEX IF NOT EXISTS idx_changes_timestamp ON changes(timestamp);
            CREATE INDEX IF NOT EXISTS idx_commits_session ON commits(session_id);
            CREATE INDEX IF NOT EXISTS idx_commits_timestamp ON commits(timestamp);
            CREATE INDEX IF NOT EXISTS idx_commits_parent ON commits(parent);
            "#,
        )?;

        let version: Option<i32> = self
            .conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .optional()?;

        if version.is_none() {
            self.conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                params![SCHEMA_VERSION],
            )?;
        }

        Ok(())
    }

    // Session operations
    pub fn create_session(&self, session: &Session) -> Result<()> {
        let ignore_patterns = serde_json::to_string(&session.ignore_patterns)?;

        self.conn.execute(
            "INSERT INTO sessions (id, root_path, started, ended, active, ignore_patterns)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session.id.to_string(),
                session.root_path.to_string_lossy().as_ref(),
                session.started.to_rfc3339(),
                session.ended.map(|dt| dt.to_rfc3339()),
                session.active as i32,
                ignore_patterns,
            ],
        )?;

        Ok(())
    }

    pub fn get_session(&self, id: &Uuid) -> Result<Session> {
        self.conn
            .query_row(
                "SELECT id, root_path, started, ended, active, ignore_patterns FROM sessions WHERE id = ?1",
                params![id.to_string()],
                |row| self.session_from_row(row),
            )
            .map_err(|_| Error::SessionNotFound(id.to_string()))
    }

    pub fn get_active_session(&self) -> Result<Session> {
        self.conn
            .query_row(
                "SELECT id, root_path, started, ended, active, ignore_patterns FROM sessions WHERE active = 1 LIMIT 1",
                [],
                |row| self.session_from_row(row),
            )
            .map_err(|_| Error::NoActiveSession)
    }

    pub fn update_session(&self, session: &Session) -> Result<()> {
        let ignore_patterns = serde_json::to_string(&session.ignore_patterns)?;

        self.conn.execute(
            "UPDATE sessions SET ended = ?1, active = ?2, ignore_patterns = ?3 WHERE id = ?4",
            params![
                session.ended.map(|dt| dt.to_rfc3339()),
                session.active as i32,
                ignore_patterns,
                session.id.to_string(),
            ],
        )?;

        Ok(())
    }

    // Change operations
    pub fn create_change(&self, change: &Change) -> Result<()> {
        let metadata = serde_json::to_string(&change.metadata)?;

        self.conn.execute(
            "INSERT INTO changes (id, session_id, timestamp, change_type, path, old_path,
                                  content_before, content_after, content_hash_before, content_hash_after,
                                  agent_id, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                change.id.to_string(),
                change.session_id.to_string(),
                change.timestamp.to_rfc3339(),
                change.change_type.as_str(),
                change.path.to_string_lossy().as_ref(),
                change.old_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                change.content_before.as_ref(),
                change.content_after.as_ref(),
                change.content_hash_before.as_ref(),
                change.content_hash_after.as_ref(),
                change.agent_id.as_ref(),
                metadata,
            ],
        )?;

        Ok(())
    }

    pub fn get_change(&self, id: &Uuid) -> Result<Change> {
        self.conn
            .query_row(
                "SELECT id, session_id, timestamp, change_type, path, old_path,
                        content_before, content_after, content_hash_before, content_hash_after,
                        agent_id, metadata FROM changes WHERE id = ?1",
                params![id.to_string()],
                |row| self.change_from_row(row),
            )
            .map_err(|_| Error::ChangeNotFound(id.to_string()))
    }

    pub fn get_uncommitted_changes(&self, session_id: &Uuid) -> Result<Vec<Change>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.session_id, c.timestamp, c.change_type, c.path, c.old_path,
                    c.content_before, c.content_after, c.content_hash_before, c.content_hash_after,
                    c.agent_id, c.metadata
             FROM changes c
             WHERE c.session_id = ?1 AND c.id NOT IN (
                 SELECT change_id FROM commit_changes
             )
             ORDER BY c.timestamp DESC",
        )?;

        let changes = stmt
            .query_map(params![session_id.to_string()], |row| {
                self.change_from_row(row)
            })?
            .collect::<rusqlite::Result<Vec<Change>>>()?;

        Ok(changes)
    }

    // Commit operations
    pub fn create_commit(&self, commit: &Commit) -> Result<()> {
        let metadata = serde_json::to_string(&commit.metadata)?;

        self.conn.execute(
            "INSERT INTO commits (id, session_id, parent, timestamp, message, agent_id, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                commit.id.to_string(),
                commit.session_id.to_string(),
                commit.parent.as_ref().map(|p| p.to_string()),
                commit.timestamp.to_rfc3339(),
                commit.message,
                commit.agent_id,
                metadata,
            ],
        )?;

        for change_id in &commit.changes {
            self.conn.execute(
                "INSERT INTO commit_changes (commit_id, change_id) VALUES (?1, ?2)",
                params![commit.id.to_string(), change_id.to_string()],
            )?;
        }

        Ok(())
    }

    pub fn get_commit(&self, id: &Uuid) -> Result<Commit> {
        let commit = self
            .conn
            .query_row(
                "SELECT id, session_id, parent, timestamp, message, agent_id, metadata
                 FROM commits WHERE id = ?1",
                params![id.to_string()],
                |row| self.commit_from_row(row),
            )
            .map_err(|_| Error::CommitNotFound(id.to_string()))?;

        Ok(commit)
    }

    pub fn get_commits_for_session(&self, session_id: &Uuid) -> Result<Vec<CommitInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, parent, timestamp, message, agent_id, metadata
             FROM commits WHERE session_id = ?1 ORDER BY timestamp DESC",
        )?;

        let mut commits = Vec::new();
        let rows = stmt.query_map(params![session_id.to_string()], |row| {
            self.commit_from_row(row)
        })?;

        for commit_result in rows {
            let commit = commit_result?;
            let info = self.get_commit_info(&commit)?;
            commits.push(info);
        }

        Ok(commits)
    }

    fn get_commit_info(&self, commit: &Commit) -> Result<CommitInfo> {
        let changes: Vec<Change> = commit
            .changes
            .iter()
            .filter_map(|id| self.get_change(id).ok())
            .collect();

        let files_affected: Vec<PathBuf> = changes.iter().map(|c| c.path.clone()).collect();

        Ok(CommitInfo {
            commit: commit.clone(),
            change_count: changes.len(),
            files_affected,
        })
    }

    // Helper methods
    fn session_from_row(&self, row: &Row) -> rusqlite::Result<Session> {
        let id: String = row.get(0)?;
        let root_path: String = row.get(1)?;
        let started: String = row.get(2)?;
        let ended: Option<String> = row.get(3)?;
        let active: i32 = row.get(4)?;
        let ignore_patterns: String = row.get(5)?;

        Ok(Session {
            id: Uuid::parse_str(&id).unwrap(),
            root_path: PathBuf::from(root_path),
            started: DateTime::parse_from_rfc3339(&started).unwrap().into(),
            ended: ended.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.into())),
            active: active != 0,
            ignore_patterns: serde_json::from_str(&ignore_patterns).unwrap_or_default(),
        })
    }

    fn change_from_row(&self, row: &Row) -> rusqlite::Result<Change> {
        let id: String = row.get(0)?;
        let session_id: String = row.get(1)?;
        let timestamp: String = row.get(2)?;
        let change_type: String = row.get(3)?;
        let path: String = row.get(4)?;
        let old_path: Option<String> = row.get(5)?;
        let content_before: Option<Vec<u8>> = row.get(6)?;
        let content_after: Option<Vec<u8>> = row.get(7)?;
        let content_hash_before: Option<String> = row.get(8)?;
        let content_hash_after: Option<String> = row.get(9)?;
        let agent_id: Option<String> = row.get(10)?;
        let metadata: String = row.get(11)?;

        Ok(Change {
            id: Uuid::parse_str(&id).unwrap(),
            timestamp: DateTime::parse_from_rfc3339(&timestamp).unwrap().into(),
            change_type: ChangeType::parse(&change_type).unwrap(),
            path: PathBuf::from(path),
            old_path: old_path.map(PathBuf::from),
            content_before,
            content_after,
            content_hash_before,
            content_hash_after,
            agent_id,
            metadata: serde_json::from_str(&metadata).unwrap_or_default(),
            session_id: Uuid::parse_str(&session_id).unwrap(),
        })
    }

    fn commit_from_row(&self, row: &Row) -> rusqlite::Result<Commit> {
        let id: String = row.get(0)?;
        let session_id: String = row.get(1)?;
        let parent: Option<String> = row.get(2)?;
        let timestamp: String = row.get(3)?;
        let message: String = row.get(4)?;
        let agent_id: String = row.get(5)?;
        let metadata: String = row.get(6)?;

        let changes = self.get_changes_for_commit(&id)?;

        Ok(Commit {
            id: Uuid::parse_str(&id).unwrap(),
            parent: parent.and_then(|p| Uuid::parse_str(&p).ok()),
            timestamp: DateTime::parse_from_rfc3339(&timestamp).unwrap().into(),
            message,
            agent_id,
            changes,
            session_id: Uuid::parse_str(&session_id).unwrap(),
            metadata: serde_json::from_str(&metadata).unwrap_or_default(),
        })
    }

    fn get_changes_for_commit(&self, commit_id: &str) -> rusqlite::Result<Vec<Uuid>> {
        let mut stmt = self
            .conn
            .prepare("SELECT change_id FROM commit_changes WHERE commit_id = ?1")?;

        let changes = stmt
            .query_map(params![commit_id], |row| {
                let id: String = row.get(0)?;
                Ok(Uuid::parse_str(&id).unwrap())
            })?
            .collect::<rusqlite::Result<Vec<Uuid>>>()?;

        Ok(changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_initialization() {
        let storage = Storage::in_memory().unwrap();
        assert!(storage.conn.is_autocommit());
    }

    #[test]
    fn test_session_crud() {
        let storage = Storage::in_memory().unwrap();
        let session = Session::new(PathBuf::from("/test"));

        storage.create_session(&session).unwrap();
        let retrieved = storage.get_session(&session.id).unwrap();

        assert_eq!(session.id, retrieved.id);
        assert_eq!(session.root_path, retrieved.root_path);
        assert!(retrieved.active);
    }

    #[test]
    fn test_change_creation() {
        let storage = Storage::in_memory().unwrap();
        let session = Session::new(PathBuf::from("/test"));
        storage.create_session(&session).unwrap();

        let change = Change::new(ChangeType::Create, PathBuf::from("test.txt"), session.id)
            .with_content_after(b"Hello".to_vec());

        storage.create_change(&change).unwrap();
        let retrieved = storage.get_change(&change.id).unwrap();

        assert_eq!(change.id, retrieved.id);
        assert_eq!(change.path, retrieved.path);
        assert_eq!(change.change_type, retrieved.change_type);
    }

    #[test]
    fn test_commit_with_changes() {
        let storage = Storage::in_memory().unwrap();
        let session = Session::new(PathBuf::from("/test"));
        storage.create_session(&session).unwrap();

        let change1 = Change::new(ChangeType::Create, PathBuf::from("file1.txt"), session.id);
        let change2 = Change::new(ChangeType::Create, PathBuf::from("file2.txt"), session.id);

        storage.create_change(&change1).unwrap();
        storage.create_change(&change2).unwrap();

        let commit = Commit::new(
            "Test commit".to_string(),
            "test-agent".to_string(),
            vec![change1.id, change2.id],
            session.id,
        );

        storage.create_commit(&commit).unwrap();
        let retrieved = storage.get_commit(&commit.id).unwrap();

        assert_eq!(commit.id, retrieved.id);
        assert_eq!(commit.message, retrieved.message);
        assert_eq!(2, retrieved.changes.len());
    }
}
