//! # gitent-core
//!
//! Core library for gitent - change tracking and storage for AI agent changes.
//!
//! This crate provides the fundamental data structures and database operations
//! for tracking file system changes, commits, and rollbacks.

pub mod diff;
pub mod error;
pub mod models;
pub mod storage;

pub use error::{Error, Result};
pub use models::{Change, ChangeType, Commit, CommitInfo, Session};
pub use storage::Storage;
