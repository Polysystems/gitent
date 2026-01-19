//! # gitent-sdk
//!
//! SDK for AI agents to integrate with gitent version control.
//!
//! ## Example
//!
//! ```no_run
//! use gitent_sdk::GitentClient;
//!
//! let client = GitentClient::new("http://localhost:3030", "my-agent");
//!
//! // Announce a file write
//! client.file_written("src/main.rs", "fn main() {}", None).unwrap();
//!
//! // Commit changes
//! client.commit("Implemented main function").unwrap();
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone)]
pub struct GitentClient {
    base_url: String,
    agent_id: String,
    client: reqwest::blocking::Client,
}

#[derive(Serialize)]
struct CreateChangeRequest {
    change_type: String,
    path: String,
    content_before: Option<String>,
    content_after: Option<String>,
    agent_id: Option<String>,
}

#[derive(Serialize)]
struct CreateCommitRequest {
    message: String,
    agent_id: String,
    change_ids: Vec<String>,
}

#[derive(Deserialize)]
struct Change {
    id: String,
}

impl GitentClient {
    /// Create a new gitent client
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL of the gitent server (e.g., "http://localhost:3030")
    /// * `agent_id` - Unique identifier for this agent
    pub fn new(base_url: impl Into<String>, agent_id: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            agent_id: agent_id.into(),
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Announce that a file was created
    pub fn file_created(&self, path: &str, content: &str) -> Result<()> {
        self.create_change("create", path, None, Some(content))
    }

    /// Announce that a file was modified
    pub fn file_modified(
        &self,
        path: &str,
        content_before: &str,
        content_after: &str,
    ) -> Result<()> {
        self.create_change("modify", path, Some(content_before), Some(content_after))
    }

    /// Announce that a file was written (create or modify)
    pub fn file_written(
        &self,
        path: &str,
        content: &str,
        previous_content: Option<&str>,
    ) -> Result<()> {
        if let Some(prev) = previous_content {
            self.file_modified(path, prev, content)
        } else {
            self.file_created(path, content)
        }
    }

    /// Announce that a file was deleted
    pub fn file_deleted(&self, path: &str, content_before: Option<&str>) -> Result<()> {
        self.create_change("delete", path, content_before, None)
    }

    fn create_change(
        &self,
        change_type: &str,
        path: &str,
        content_before: Option<&str>,
        content_after: Option<&str>,
    ) -> Result<()> {
        let request = CreateChangeRequest {
            change_type: change_type.to_string(),
            path: path.to_string(),
            content_before: content_before.map(|s| s.to_string()),
            content_after: content_after.map(|s| s.to_string()),
            agent_id: Some(self.agent_id.clone()),
        };

        self.client
            .post(format!("{}/changes", self.base_url))
            .json(&request)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    /// Get all uncommitted changes
    pub fn get_uncommitted_changes(&self) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        let response = self
            .client
            .get(format!("{}/changes", self.base_url))
            .send()?
            .error_for_status()?;

        Ok(response.json()?)
    }

    /// Commit all uncommitted changes
    pub fn commit(&self, message: &str) -> Result<String> {
        // Get uncommitted changes
        let changes: Vec<Change> = self
            .client
            .get(format!("{}/changes", self.base_url))
            .send()?
            .error_for_status()?
            .json()?;

        let change_ids: Vec<String> = changes.iter().map(|c| c.id.clone()).collect();

        let request = CreateCommitRequest {
            message: message.to_string(),
            agent_id: self.agent_id.clone(),
            change_ids,
        };

        let response: serde_json::Value = self
            .client
            .post(format!("{}/commits", self.base_url))
            .json(&request)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(response["id"].as_str().unwrap_or("unknown").to_string())
    }

    /// Get commit history
    pub fn get_commits(&self) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        let response = self
            .client
            .get(format!("{}/commits", self.base_url))
            .send()?
            .error_for_status()?;

        Ok(response.json()?)
    }

    /// Check server health
    pub fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GitentClient::new("http://localhost:3030", "test-agent");
        assert_eq!(client.base_url, "http://localhost:3030");
        assert_eq!(client.agent_id, "test-agent");
    }
}
