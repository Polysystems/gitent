use anyhow::Result;
use colored::Colorize;
use gitent_core::Storage;
use std::path::PathBuf;

pub fn run(limit: Option<usize>, db: Option<PathBuf>) -> Result<()> {
    let db_path = super::get_db_path(db);

    if !db_path.exists() {
        anyhow::bail!("No active gitent session found. Run 'gitent start' first.");
    }

    let storage = Storage::new(&db_path)?;
    let session = storage.get_active_session()?;
    let commits = storage.get_commits_for_session(&session.id)?;

    if commits.is_empty() {
        println!("{}", "No commits yet".yellow());
        return Ok(());
    }

    println!("{}", "Commit History".bold().cyan());
    println!();

    let to_show = limit.unwrap_or(commits.len()).min(commits.len());

    for commit_info in commits.iter().take(to_show) {
        let commit = &commit_info.commit;

        println!(
            "{} {}",
            "commit".yellow().bold(),
            commit.id.to_string().yellow()
        );
        println!("{}: {}", "Agent".bold(), commit.agent_id);
        println!(
            "{}: {}",
            "Date".bold(),
            commit.timestamp.format("%Y-%m-%d %H:%M:%S")
        );
        println!();
        println!("    {}", commit.message);
        println!();
        println!(
            "    {} file(s) changed",
            commit_info.change_count.to_string().cyan()
        );

        if !commit_info.files_affected.is_empty() {
            for path in commit_info.files_affected.iter().take(5) {
                println!("      • {}", path.display().to_string().dimmed());
            }
            if commit_info.files_affected.len() > 5 {
                println!(
                    "      {} and {} more...",
                    "...".dimmed(),
                    (commit_info.files_affected.len() - 5).to_string().dimmed()
                );
            }
        }

        println!();
    }

    if commits.len() > to_show {
        println!(
            "{}",
            format!("... and {} more commits", commits.len() - to_show).dimmed()
        );
        println!("Use {} to see more", "--limit N".cyan());
    }

    Ok(())
}
