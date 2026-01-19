use anyhow::Result;
use colored::Colorize;
use gitent_core::Storage;
use std::path::PathBuf;

pub fn run(db: Option<PathBuf>) -> Result<()> {
    let db_path = super::get_db_path(db);

    if !db_path.exists() {
        println!("{}", "No active gitent session found".red());
        println!("Run {} to start tracking", "gitent start".cyan());
        return Ok(());
    }

    let storage = Storage::new(&db_path)?;
    let session = storage.get_active_session()?;
    let changes = storage.get_uncommitted_changes(&session.id)?;

    println!("{}", "Session Status".bold().cyan());
    println!("  {}: {}", "Root".bold(), session.root_path.display());
    println!("  {}: {}", "Session ID".bold(), session.id);
    println!(
        "  {}: {}",
        "Started".bold(),
        session.started.format("%Y-%m-%d %H:%M:%S")
    );
    println!();

    if changes.is_empty() {
        println!("{}", "No uncommitted changes".green());
    } else {
        println!(
            "{} {}",
            "Uncommitted changes:".bold(),
            format!("({})", changes.len()).yellow()
        );
        println!();

        for change in changes.iter().take(10) {
            let icon = match change.change_type {
                gitent_core::ChangeType::Create => "+".green(),
                gitent_core::ChangeType::Modify => "~".yellow(),
                gitent_core::ChangeType::Delete => "-".red(),
                gitent_core::ChangeType::Rename => "→".blue(),
            };

            println!("  {} {}", icon, change.path.display());
        }

        if changes.len() > 10 {
            println!();
            println!(
                "  {} and {} more...",
                "...".dimmed(),
                (changes.len() - 10).to_string().yellow()
            );
        }

        println!();
        println!(
            "Run {} to commit these changes",
            "gitent commit \"message\"".cyan()
        );
    }

    Ok(())
}
