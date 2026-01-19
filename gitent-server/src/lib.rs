//! # gitent-server
//!
//! Server component for gitent that watches files and provides an API for agents.

pub mod api;
pub mod server;
pub mod watcher;

pub use server::GitentServer;
pub use watcher::FileWatcher;
