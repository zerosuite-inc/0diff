# 0diff

**Know who changed what. Even when it's not human.**

Real-time code modification tracking for the multi-agent era.

## Install

```bash
curl -fsSL https://0diff.dev/install.sh | sh
```

## Build from Source

Requires [Rust](https://rustup.rs) (1.70+).

```bash
# Clone
git clone https://github.com/zerosuite-inc/0diff.git
cd 0diff

# Build (debug)
cargo build

# Build (release — optimized, ~2MB binary)
cargo build --release

# The binary is at:
#   debug:   ./target/debug/0diff
#   release: ./target/release/0diff

# Install to PATH
cargo install --path .

# Run tests
cargo test
```

Or install directly from GitHub without cloning:

```bash
cargo install --git https://github.com/zerosuite-inc/0diff.git
```

## Quick Start

```bash
cd your-project
0diff init      # Creates .0diff.toml + .0diff/ directory
0diff watch     # Start watching — edit a file to see it in action
```

## Commands

```
0diff init          Create .0diff.toml config
0diff watch         Watch files, show diffs in real-time
0diff diff <file>   Show current diff vs last commit
0diff log           Browse change history
0diff status        See what's being watched
```

All commands support `--json` for composability.

```bash
# Examples
0diff log --json                    # JSON output
0diff log --author "alice"          # Filter by author
0diff log --agent "Claude"          # Filter by AI agent
0diff log --limit 20                # Last 20 changes
0diff diff src/main.rs              # Diff a specific file
0diff diff src/main.rs --json       # Diff as JSON
```

## AI Agent Detection

0diff automatically detects changes made by AI agents:

- **Commit analysis** — scans `Co-Authored-By` headers and commit messages for Claude, Cursor, Copilot, Windsurf, Devin
- **Environment detection** — checks for agent-specific environment variables (`CLAUDE_CODE`, `CURSOR_SESSION`, etc.)
- **TTY heuristic** — flags non-interactive sessions as potential agent edits

Terminal output marks AI changes with a `[AI AGENT]` badge:

```
[14:33:12] src/vm.rs  +3 -1  (Claude on main)  [AI AGENT]
  @@ -100,1 +100,3 @@
  + fn optimize_bytecode(&mut self) {
```

## Config

Run `0diff init` to create `.0diff.toml`:

```toml
[watch]
paths = ["src/", "app/", "entities/"]
ignore = ["target/", "node_modules/", ".git/"]
extensions = ["rs", "flin", "ts", "js", "py", "go", "java"]
debounce_ms = 500

[filter]
ignore_whitespace = true
min_lines_changed = 1

[git]
enabled = true
track_author = true
track_branch = true

[history]
max_size_mb = 10
max_days = 30

[agents]
detect_patterns = ["Claude", "Cursor", "Copilot", "Windsurf", "Devin"]
tag_non_human = true
```

## Project Structure

```
src/
├── main.rs        # CLI (clap derive)
├── lib.rs         # Module re-exports
├── config.rs      # .0diff.toml parsing + defaults
├── watcher.rs     # File watcher (notify + debouncer)
├── differ.rs      # Diff engine (similar crate)
├── filter.rs      # Whitespace filtering
├── git.rs         # Git blame/branch/author
├── history.rs     # JSON-lines change log
├── agents.rs      # AI agent detection
└── output.rs      # Terminal colored output + JSON
```

## Part of the FLIN Ecosystem

Built by ZeroSuite, Inc. — Cotonou, Benin

[flin.sh](https://flin.sh) | [0diff.dev](https://0diff.dev) | [zerosuite.dev](https://zerosuite.dev)

## License

MIT
