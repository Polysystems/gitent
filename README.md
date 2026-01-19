# gitent - Version Control for AI Agents

Git, but for AI agents. Track, commit, and rollback changes made by AI agents to your codebase.

## Overview

gitent is a version control system designed specifically for AI agent workflows. It automatically tracks file changes, provides an API for agents to announce their actions, and allows you to review and rollback changes with full transparency.

## Features

- ✅ **Automatic Change Tracking** - File watcher monitors all changes in real-time
- ✅ **Agent Integration** - Simple SDK for agents to announce their actions
- ✅ **Commit History** - Full commit tree with messages and metadata
- ✅ **Rollback Support** - Revert to any previous commit
- ✅ **Diff Viewer** - See exactly what changed
- ✅ **HTTP API** - REST API for programmatic access
- ✅ **CLI Tool** - Command-line interface for humans
- ✅ **SQLite Backend** - Lightweight, file-based storage
- ✅ **Content Hashing** - Detect duplicate content

## Installation

```bash
cargo install gitent-cli
```

Or build from source:

```bash
git clone https://github.com/yourusername/gitent
cd gitent
cargo build --release
```

The binary will be at `target/release/gitent`.

## Quick Start

### 1. Start Tracking

```bash
# Start gitent server in current directory
gitent start .

# Or specify a different directory
gitent start /path/to/project

# Custom port
gitent start . --port 8080
```

This starts:
- File watcher (monitors changes automatically)
- HTTP API server (for agent integration)
- SQLite database in `.gitent/gitent.db`

### 2. Make Changes

Either let the file watcher detect changes automatically, or have your agent announce them via the API/SDK.

### 3. Check Status

```bash
gitent status
```

Output:
```
Session Status
  Root: /home/user/project
  Session ID: 550e8400-e29b-41d4-a716-446655440000
  Started: 2024-01-15 10:30:00

Uncommitted changes: (3)

  + src/main.rs
  ~ README.md
  - old_file.txt

Run gitent commit "message" to commit these changes
```

### 4. Commit Changes

```bash
gitent commit "Implemented new feature"
```

### 5. View History

```bash
gitent log
```

Output:
```
Commit History

commit 7c9e6679-7425-40de-944b-e07fc1f90ae7
Agent: my-agent
Date: 2024-01-15 10:35:22

    Implemented new feature

    3 file(s) changed
      • src/main.rs
      • README.md
      • old_file.txt
```

### 6. View Diff

```bash
# See uncommitted changes
gitent diff

# See specific commit
gitent diff 7c9e6679-7425-40de-944b-e07fc1f90ae7
```

### 7. Rollback

```bash
# Preview rollback
gitent rollback 7c9e6679-7425-40de-944b-e07fc1f90ae7

# Actually perform rollback
gitent rollback 7c9e6679-7425-40de-944b-e07fc1f90ae7 --execute
```

## Agent Integration

### Using the Rust SDK

Add to your `Cargo.toml`:

```toml
[dependencies]
gitent-sdk = "0.1"
```

Example usage:

```rust
use gitent_sdk::GitentClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to gitent server
    let client = GitentClient::new("http://localhost:3030", "my-agent");

    // Announce file creation
    client.file_created("src/new_file.rs", "fn main() {}")?;

    // Announce file modification
    client.file_modified(
        "src/main.rs",
        "old content",
        "new content"
    )?;

    // Announce file deletion
    client.file_deleted("old_file.txt", Some("old content"))?;

    // Commit changes
    let commit_id = client.commit("Implemented feature X")?;
    println!("Created commit: {}", commit_id);

    Ok(())
}
```

### Using the HTTP API

#### Create a Change

```bash
curl -X POST http://localhost:3030/changes \
  -H "Content-Type: application/json" \
  -d '{
    "change_type": "modify",
    "path": "src/main.rs",
    "content_before": "old code",
    "content_after": "new code",
    "agent_id": "my-agent"
  }'
```

#### Get Uncommitted Changes

```bash
curl http://localhost:3030/changes
```

#### Create a Commit

```bash
curl -X POST http://localhost:3030/commits \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Implemented feature",
    "agent_id": "my-agent",
    "change_ids": ["uuid-1", "uuid-2"]
  }'
```

#### Get Commit History

```bash
curl http://localhost:3030/commits
```

## CLI Reference

### `gitent start`

Start tracking changes in a directory.

```bash
gitent start [PATH] [OPTIONS]

Options:
  -p, --port <PORT>    API server port [default: 3030]
  -d, --db <PATH>      Database path [default: .gitent/gitent.db]
```

### `gitent status`

Show current session status and uncommitted changes.

```bash
gitent status [OPTIONS]

Options:
  -d, --db <PATH>      Database path
```

### `gitent commit`

Commit uncommitted changes.

```bash
gitent commit <MESSAGE> [OPTIONS]

Arguments:
  <MESSAGE>            Commit message

Options:
  -a, --agent <AGENT>  Agent ID [default: cli-user]
  -d, --db <PATH>      Database path
```

### `gitent log`

Show commit history.

```bash
gitent log [OPTIONS]

Options:
  -l, --limit <N>      Number of commits to show
  -d, --db <PATH>      Database path
```

### `gitent diff`

Show diff for a commit or uncommitted changes.

```bash
gitent diff [COMMIT_ID] [OPTIONS]

Arguments:
  [COMMIT_ID]          Commit ID (if not provided, shows uncommitted changes)

Options:
  -d, --db <PATH>      Database path
```

### `gitent rollback`

Rollback to a specific commit.

```bash
gitent rollback <COMMIT_ID> [OPTIONS]

Arguments:
  <COMMIT_ID>          Commit ID to rollback to

Options:
  --execute            Actually perform the rollback (preview only by default)
  -d, --db <PATH>      Database path
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│                 AI Agent                         │
│  (uses gitent-sdk to track changes)             │
└──────────────────┬──────────────────────────────┘
                   │ HTTP API
                   ↓
┌─────────────────────────────────────────────────┐
│           gitent-server                          │
│  ┌─────────────────────────────────────────┐   │
│  │  File Watcher                           │   │
│  │  - Watches for file changes             │   │
│  │  - Debounces rapid changes              │   │
│  └─────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────┐   │
│  │  HTTP API (Axum)                        │   │
│  │  - REST endpoints for agents            │   │
│  └─────────────────────────────────────────┘   │
└──────────────────┬──────────────────────────────┘
                   │
                   ↓
┌─────────────────────────────────────────────────┐
│         SQLite Database                          │
│  - changes table                                 │
│  - commits table                                 │
│  - sessions table                                │
└─────────────────────────────────────────────────┘
                   ↑
                   │
┌──────────────────┴──────────────────────────────┐
│           gitent-cli                             │
│  - gitent start                                  │
│  - gitent commit                                 │
│  - gitent log                                    │
│  - gitent status                                 │
│  - gitent diff                                   │
│  - gitent rollback                               │
└─────────────────────────────────────────────────┘
```

## Use Cases

### 1. AI Coding Assistant Safety

Track all changes made by your AI coding assistant and rollback problematic changes:

```bash
# Start tracking
gitent start .

# Let AI make changes...
# AI uses SDK to announce changes

# Review changes
gitent status
gitent diff

# Commit good changes
gitent commit "Added error handling"

# Or rollback if needed
gitent rollback <previous-commit-id> --execute
```

### 2. Multi-Agent Collaboration

Track changes from multiple agents:

```rust
// Agent 1
let agent1 = GitentClient::new("http://localhost:3030", "refactoring-agent");
agent1.file_modified("src/lib.rs", old, new)?;
agent1.commit("Refactored modules")?;

// Agent 2
let agent2 = GitentClient::new("http://localhost:3030", "test-agent");
agent2.file_created("tests/integration.rs", test_code)?;
agent2.commit("Added integration tests")?;
```

View per-agent history:

```bash
gitent log
# Shows commits from both agents
```

### 3. Experiment Tracking

Track experimental changes:

```bash
gitent start ./experiments

# Run experiment
python run_experiment.py  # Uses gitent SDK

# Review results
gitent log
gitent diff

# Keep good experiments
gitent commit "Successful optimization"

# Discard bad ones
gitent rollback <before-experiment> --execute
```

## Development

### Project Structure

```
gitent/
├── gitent-core/          # Core library (storage, models, diff)
├── gitent-server/        # File watcher and HTTP API
├── gitent-cli/           # CLI tool
└── gitent-sdk/           # SDK for agent integration
```

### Building

```bash
cargo build --workspace
```

### Testing

```bash
cargo test --workspace
```

### Running in Dev Mode

```bash
# Start server
cargo run -p gitent-cli -- start .

# In another terminal
cargo run -p gitent-cli -- status
cargo run -p gitent-cli -- commit "test"
```

## Comparison with Git

| Feature | gitent | git |
|---------|--------|-----|
| Target Users | AI Agents | Humans |
| Change Detection | Automatic file watching | Manual `git add` |
| API | HTTP REST API | Command-line only |
| SDK | Rust, Python (planned) | None (CLI only) |
| Granularity | Per-file | Per-hunk |
| Focus | Agent transparency | Code collaboration |

gitent is **not** a replacement for git. Use git for human collaboration, use gitent for AI agent transparency.

## Roadmap

- [ ] Python SDK (PyO3 bindings)
- [ ] JavaScript/TypeScript SDK
- [ ] Web UI for visualization
- [ ] Branching support
- [ ] Change suggestions/review
- [ ] Integration with popular agent frameworks

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
