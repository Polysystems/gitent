use gitent_sdk::GitentClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 AI Agent Example - Integrating with gitent\n");

    // Connect to gitent server
    let client = GitentClient::new("http://localhost:3030", "example-agent");

    // Check if server is running
    if !client.health_check()? {
        eprintln!("Error: gitent server is not running!");
        eprintln!("Start it with: gitent start .");
        return Ok(());
    }

    println!("✓ Connected to gitent server");
    println!();

    // Simulate agent creating a new file
    println!("📝 Creating new file: src/hello.rs");
    client.file_created(
        "src/hello.rs",
        r#"pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello, World!");
    }
}
"#,
    )?;
    println!("✓ Change tracked");
    println!();

    // Simulate modifying an existing file
    println!("📝 Modifying file: README.md");
    client.file_modified(
        "README.md",
        "# My Project\n\nOld description",
        "# My Project\n\nUpdated description with new features",
    )?;
    println!("✓ Change tracked");
    println!();

    // Check uncommitted changes
    println!("📊 Checking uncommitted changes...");
    let changes = client.get_uncommitted_changes()?;
    println!("✓ Found {} uncommitted change(s)", changes.len());
    println!();

    // Commit the changes
    println!("💾 Committing changes...");
    let commit_id = client.commit("Added greeting functionality and updated README")?;
    println!("✓ Commit created: {}", commit_id);
    println!();

    // View commit history
    println!("📜 Fetching commit history...");
    let commits = client.get_commits()?;
    println!("✓ Found {} commit(s) in history", commits.len());
    println!();

    println!("✅ Example completed successfully!");
    println!();
    println!("Try these commands:");
    println!("  gitent status    - View current status");
    println!("  gitent log       - View commit history");
    println!("  gitent diff      - See what changed");

    Ok(())
}
