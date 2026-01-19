use anyhow::Result;
use colored::Colorize;
use gitent_server::GitentServer;
use std::path::PathBuf;

pub async fn run(path: PathBuf, port: u16, db: Option<PathBuf>) -> Result<()> {
    let abs_path = std::fs::canonicalize(&path)?;

    let db_path = db.unwrap_or_else(|| abs_path.join(".gitent").join("gitent.db"));

    // Create .gitent directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!("{}", "🚀 Starting gitent server...".bold().cyan());
    println!("   {}: {:?}", "Watching".bold(), abs_path);
    println!("   {}: {:?}", "Database".bold(), db_path);

    let server = GitentServer::new(abs_path.clone(), db_path)?;

    println!("   {}: {}", "Session ID".bold(), server.session_id());
    println!(
        "   {}: {}",
        "API Server".bold(),
        format!("http://localhost:{}", port).green()
    );
    println!();
    println!("{}", "Press Ctrl+C to stop".dimmed());
    println!();

    let addr = format!("0.0.0.0:{}", port).parse()?;
    server.serve(addr).await?;

    Ok(())
}
