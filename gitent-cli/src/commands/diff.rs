use anyhow::Result;
use colored::Colorize;
use gitent_core::{diff::FileDiff, Storage};
use std::path::PathBuf;
use uuid::Uuid;

pub fn run(commit_id: Option<String>, db: Option<PathBuf>) -> Result<()> {
    let db_path = super::get_db_path(db);

    if !db_path.exists() {
        anyhow::bail!("No active gitent session found. Run 'gitent start' first.");
    }

    let storage = Storage::new(&db_path)?;
    let session = storage.get_active_session()?;

    let changes = if let Some(id_str) = commit_id {
        let commit_id = Uuid::parse_str(&id_str)?;
        let commit = storage.get_commit(&commit_id)?;

        println!("{}", format!("Diff for commit {}", commit.id).bold().cyan());
        println!("{}: {}", "Message".bold(), commit.message);
        println!();

        commit
            .changes
            .iter()
            .filter_map(|id| storage.get_change(id).ok())
            .collect()
    } else {
        let changes = storage.get_uncommitted_changes(&session.id)?;

        if changes.is_empty() {
            println!("{}", "No uncommitted changes".green());
            return Ok(());
        }

        println!("{}", "Uncommitted changes".bold().cyan());
        println!();
        changes
    };

    for change in changes {
        println!("{}", "━".repeat(80).bright_black());

        let status = match change.change_type {
            gitent_core::ChangeType::Create => "NEW".green(),
            gitent_core::ChangeType::Modify => "MOD".yellow(),
            gitent_core::ChangeType::Delete => "DEL".red(),
            gitent_core::ChangeType::Rename => "REN".blue(),
        };

        println!(
            "{} {}",
            status,
            change.path.display().to_string().white().bold()
        );
        println!();

        match FileDiff::from_change(&change) {
            Ok(diff) => {
                for line in &diff.diff_lines {
                    let (prefix, color): (&str, fn(&str) -> colored::ColoredString) =
                        match line.line_type {
                            gitent_core::diff::DiffLineType::Addition => ("+", |s| s.green()),
                            gitent_core::diff::DiffLineType::Deletion => ("-", |s| s.red()),
                            gitent_core::diff::DiffLineType::Context => (" ", |s| s.normal()),
                        };
                    print!("{}", color(&format!("{}{}", prefix, line.content)));
                }
            }
            Err(_) => {
                println!("  {}", "[Binary file or unable to generate diff]".dimmed());
            }
        }
        println!();
    }

    Ok(())
}
