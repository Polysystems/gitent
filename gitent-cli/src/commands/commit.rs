use anyhow::{Context, Result};
use colored::Colorize;
use gitent_core::{Commit, Storage};
use std::path::PathBuf;

pub fn run(message: String, agent_id: String, db: Option<PathBuf>) -> Result<()> {
    let db_path = super::get_db_path(db);

    if !db_path.exists() {
        anyhow::bail!("No active gitent session found. Run 'gitent start' first.");
    }

    let storage = Storage::new(&db_path)?;
    let session = storage
        .get_active_session()
        .context("No active session found")?;

    let changes = storage.get_uncommitted_changes(&session.id)?;

    if changes.is_empty() {
        println!("{}", "No changes to commit".yellow());
        return Ok(());
    }

    println!("{}", "Creating commit...".bold());
    println!("  {}: {}", "Changes".bold(), changes.len());
    println!();

    let change_ids: Vec<_> = changes.iter().map(|c| c.id).collect();
    let commit = Commit::new(message.clone(), agent_id.clone(), change_ids, session.id);

    storage.create_commit(&commit)?;

    println!("{}", "✓ Commit created successfully!".green().bold());
    println!("  {}: {}", "Commit ID".bold(), commit.id);
    println!("  {}: {}", "Message".bold(), message);
    println!("  {}: {}", "Agent".bold(), agent_id);
    println!("  {}: {}", "Files changed".bold(), changes.len());

    Ok(())
}
