use anyhow::Result;
use colored::Colorize;
use gitent_core::Storage;
use std::path::PathBuf;
use uuid::Uuid;

pub fn run(commit_id: String, execute: bool, db: Option<PathBuf>) -> Result<()> {
    let db_path = super::get_db_path(db);

    if !db_path.exists() {
        anyhow::bail!("No active gitent session found. Run 'gitent start' first.");
    }

    let storage = Storage::new(&db_path)?;
    let session = storage.get_active_session()?;
    let commit_uuid = Uuid::parse_str(&commit_id)?;
    let commit = storage.get_commit(&commit_uuid)?;

    println!("{}", "Rollback Preview".bold().cyan());
    println!("  {}: {}", "Target Commit".bold(), commit.id);
    println!("  {}: {}", "Message".bold(), commit.message);
    println!("  {}: {}", "Agent".bold(), commit.agent_id);
    println!(
        "  {}: {}",
        "Date".bold(),
        commit.timestamp.format("%Y-%m-%d %H:%M:%S")
    );
    println!();

    // Get all changes from this commit
    let changes: Vec<_> = commit
        .changes
        .iter()
        .filter_map(|id| storage.get_change(id).ok())
        .collect();

    if changes.is_empty() {
        println!("{}", "No changes in this commit to rollback".yellow());
        return Ok(());
    }

    println!("{}", "Files to be restored:".bold());
    for change in &changes {
        let status = match change.change_type {
            gitent_core::ChangeType::Create => "will be removed".red(),
            gitent_core::ChangeType::Modify => "will be restored".yellow(),
            gitent_core::ChangeType::Delete => "will be recreated".green(),
            gitent_core::ChangeType::Rename => "will be renamed back".blue(),
        };
        println!("  {} {}", change.path.display(), status);
    }
    println!();

    if !execute {
        println!("{}", "This is a preview only.".yellow());
        println!(
            "Run with {} to actually perform the rollback",
            "--execute".cyan()
        );
        return Ok(());
    }

    // Perform the rollback
    println!("{}", "Performing rollback...".bold());

    let mut errors = Vec::new();
    let mut success_count = 0;

    for change in &changes {
        match perform_rollback_for_change(change, &session.root_path) {
            Ok(_) => {
                success_count += 1;
                println!("  {} {}", "✓".green(), change.path.display());
            }
            Err(e) => {
                errors.push((change.path.clone(), e));
                println!(
                    "  {} {} - {}",
                    "✗".red(),
                    change.path.display(),
                    "failed".red()
                );
            }
        }
    }

    println!();
    if errors.is_empty() {
        println!(
            "{}",
            format!("✓ Successfully rolled back {} file(s)", success_count)
                .green()
                .bold()
        );
    } else {
        println!(
            "{}",
            format!("⚠ Rolled back {}/{} files", success_count, changes.len())
                .yellow()
                .bold()
        );
        println!();
        println!("{}", "Errors:".red().bold());
        for (path, error) in errors {
            println!("  {}: {}", path.display(), error);
        }
    }

    Ok(())
}

fn perform_rollback_for_change(
    change: &gitent_core::Change,
    root_path: &std::path::Path,
) -> Result<()> {
    let full_path = root_path.join(&change.path);

    match change.change_type {
        gitent_core::ChangeType::Create => {
            // Remove the created file
            if full_path.exists() {
                std::fs::remove_file(&full_path)?;
            }
        }
        gitent_core::ChangeType::Modify => {
            // Restore previous content
            if let Some(content_before) = &change.content_before {
                std::fs::write(&full_path, content_before)?;
            }
        }
        gitent_core::ChangeType::Delete => {
            // Recreate the deleted file
            if let Some(content_before) = &change.content_before {
                if let Some(parent) = full_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&full_path, content_before)?;
            }
        }
        gitent_core::ChangeType::Rename => {
            // Rename back to old path
            if let Some(old_path) = &change.old_path {
                let old_full_path = root_path.join(old_path);
                if full_path.exists() {
                    std::fs::rename(&full_path, &old_full_path)?;
                }
            }
        }
    }

    Ok(())
}
