use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Create,
    Modify,
    Delete,
    Rename,
}

impl ChangeType {
    pub fn as_str(&self) -> &str {
        match self {
            ChangeType::Create => "create",
            ChangeType::Modify => "modify",
            ChangeType::Delete => "delete",
            ChangeType::Rename => "rename",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "create" => Some(ChangeType::Create),
            "modify" => Some(ChangeType::Modify),
            "delete" => Some(ChangeType::Delete),
            "rename" => Some(ChangeType::Rename),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub change_type: ChangeType,
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
    pub content_before: Option<Vec<u8>>,
    pub content_after: Option<Vec<u8>>,
    pub content_hash_before: Option<String>,
    pub content_hash_after: Option<String>,
    pub agent_id: Option<String>,
    pub metadata: HashMap<String, String>,
    pub session_id: Uuid,
}

impl Change {
    pub fn new(change_type: ChangeType, path: PathBuf, session_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            change_type,
            path,
            old_path: None,
            content_before: None,
            content_after: None,
            content_hash_before: None,
            content_hash_after: None,
            agent_id: None,
            metadata: HashMap::new(),
            session_id,
        }
    }

    pub fn with_content_before(mut self, content: Vec<u8>) -> Self {
        self.content_hash_before = Some(Self::hash_content(&content));
        self.content_before = Some(content);
        self
    }

    pub fn with_content_after(mut self, content: Vec<u8>) -> Self {
        self.content_hash_after = Some(Self::hash_content(&content));
        self.content_after = Some(content);
        self
    }

    pub fn with_agent_id(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_old_path(mut self, old_path: PathBuf) -> Self {
        self.old_path = Some(old_path);
        self
    }

    fn hash_content(content: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content);
        hex::encode(hasher.finalize())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: Uuid,
    pub parent: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub agent_id: String,
    pub changes: Vec<Uuid>,
    pub session_id: Uuid,
    pub metadata: HashMap<String, String>,
}

impl Commit {
    pub fn new(message: String, agent_id: String, changes: Vec<Uuid>, session_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent: None,
            timestamp: Utc::now(),
            message,
            agent_id,
            changes,
            session_id,
            metadata: HashMap::new(),
        }
    }

    pub fn with_parent(mut self, parent: Uuid) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub root_path: PathBuf,
    pub started: DateTime<Utc>,
    pub ended: Option<DateTime<Utc>>,
    pub active: bool,
    pub ignore_patterns: Vec<String>,
}

impl Session {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            root_path,
            started: Utc::now(),
            ended: None,
            active: true,
            ignore_patterns: vec![
                ".git".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
                ".gitent".to_string(),
            ],
        }
    }

    pub fn with_ignore_patterns(mut self, patterns: Vec<String>) -> Self {
        self.ignore_patterns = patterns;
        self
    }

    pub fn end(&mut self) {
        self.active = false;
        self.ended = Some(Utc::now());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub commit: Commit,
    pub change_count: usize,
    pub files_affected: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_creation() {
        let session_id = Uuid::new_v4();
        let change = Change::new(ChangeType::Create, PathBuf::from("test.txt"), session_id);

        assert_eq!(change.change_type, ChangeType::Create);
        assert_eq!(change.path, PathBuf::from("test.txt"));
        assert_eq!(change.session_id, session_id);
    }

    #[test]
    fn test_change_with_content() {
        let session_id = Uuid::new_v4();
        let content = b"Hello, World!".to_vec();
        let change = Change::new(ChangeType::Create, PathBuf::from("test.txt"), session_id)
            .with_content_after(content.clone());

        assert!(change.content_after.is_some());
        assert!(change.content_hash_after.is_some());
        assert_eq!(change.content_after.unwrap(), content);
    }

    #[test]
    fn test_commit_creation() {
        let session_id = Uuid::new_v4();
        let change_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        let commit = Commit::new(
            "Initial commit".to_string(),
            "test-agent".to_string(),
            change_ids.clone(),
            session_id,
        );

        assert_eq!(commit.message, "Initial commit");
        assert_eq!(commit.agent_id, "test-agent");
        assert_eq!(commit.changes, change_ids);
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new(PathBuf::from("/test/path"));

        assert_eq!(session.root_path, PathBuf::from("/test/path"));
        assert!(session.active);
        assert!(session.ended.is_none());
        assert!(!session.ignore_patterns.is_empty());
    }
}
