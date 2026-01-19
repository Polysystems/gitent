use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod display;

use commands::{commit, diff, log, rollback, start, status};

#[derive(Parser)]
#[command(name = "gitent")]
#[command(version, about = "Version control for AI agent changes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start tracking changes in a directory
    Start {
        /// Directory to track (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Port for the API server
        #[arg(short, long, default_value = "3030")]
        port: u16,

        /// Database path
        #[arg(short, long)]
        db: Option<PathBuf>,
    },

    /// Commit changes with a message
    Commit {
        /// Commit message
        message: String,

        /// Agent ID
        #[arg(short, long, default_value = "cli-user")]
        agent: String,

        /// Database path
        #[arg(short, long)]
        db: Option<PathBuf>,
    },

    /// Show commit history
    Log {
        /// Number of commits to show
        #[arg(short, long)]
        limit: Option<usize>,

        /// Database path
        #[arg(short, long)]
        db: Option<PathBuf>,
    },

    /// Show current status
    Status {
        /// Database path
        #[arg(short, long)]
        db: Option<PathBuf>,
    },

    /// Show diff for a commit or uncommitted changes
    Diff {
        /// Commit ID (if not provided, shows uncommitted changes)
        commit_id: Option<String>,

        /// Database path
        #[arg(short, long)]
        db: Option<PathBuf>,
    },

    /// Rollback to a specific commit
    Rollback {
        /// Commit ID to rollback to
        commit_id: String,

        /// Actually perform the rollback (without this, just shows preview)
        #[arg(long)]
        execute: bool,

        /// Database path
        #[arg(short, long)]
        db: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { path, port, db } => {
            start::run(path, port, db).await?;
        }
        Commands::Commit { message, agent, db } => {
            commit::run(message, agent, db)?;
        }
        Commands::Log { limit, db } => {
            log::run(limit, db)?;
        }
        Commands::Status { db } => {
            status::run(db)?;
        }
        Commands::Diff { commit_id, db } => {
            diff::run(commit_id, db)?;
        }
        Commands::Rollback {
            commit_id,
            execute,
            db,
        } => {
            rollback::run(commit_id, execute, db)?;
        }
    }

    Ok(())
}
