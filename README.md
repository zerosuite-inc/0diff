<p align="center">
  <strong>0diff</strong>
</p>

<h3 align="center">Know who changed what. Even when it's not human.</h3>

<p align="center">
  Real-time code modification tracking for the multi-agent era.<br>
  Smart diffs. Git attribution. AI agent detection. One binary. Zero config.
</p>

<p align="center">
  <a href="https://0diff.dev">Website</a> &middot;
  <a href="#install">Install</a> &middot;
  <a href="#quick-start">Quick Start</a> &middot;
  <a href="https://github.com/zerosuite-inc/0diff/releases">Releases</a>
</p>

<p align="center">
  <a href="https://github.com/zerosuite-inc/0diff/releases"><img alt="Release" src="https://img.shields.io/github/v/release/zerosuite-inc/0diff?include_prereleases&style=flat-square&color=00ff87&label=version"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue?style=flat-square"></a>
  <a href="https://github.com/zerosuite-inc/0diff"><img alt="Built with Rust" src="https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square"></a>
</p>

---

## The Problem

You're running Claude Code, Cursor, Copilot, Windsurf, or Devin alongside your team. They write code in your files, under your name, without asking. A function gets refactored silently. An indentation change hides a logic bug. A config file is modified in a 200-file PR and nobody notices.

**67% of production incidents trace back to a code change that bypassed proper review.** Teams spend an average of 4.2 hours identifying the root cause of change-related regressions. Traditional tools like `git diff` and `git blame` weren't built for a world where AI agents are co-authoring your codebase.

**0diff fixes this.** It watches your files in real-time, computes intelligent diffs, identifies who made each change (human or AI), and logs everything to a searchable history. One binary. Zero config. Total visibility.

## Demo

```
$ 0diff watch
▸ Watching 847 files across 23 directories...
▸ Git: main branch, 3 contributors tracked

[14:32:05] src/parser.rs  +12 -3  (alice on feature/parser)
  @@ -45,3 +45,12 @@
  - fn parse_expression(&mut self) -> Result<Expr> {
  + fn parse_expression(&mut self, precedence: u8) -> Result<Expr> {
  +     let left = self.parse_primary()?;

[14:33:41] src/checker.rs  +47 -2  (⚡ Claude Agent on fix/enum-collision)  [AI AGENT]
  @@ -189,2 +189,49 @@
  + // Resolve enum variant / entity name collision
  + fn resolve_name_shadowing(&mut self, scope: &Scope) {
```

## Why 0diff?

| Capability | git diff | watchexec | fswatch | **0diff** |
|---|---|---|---|---|
| Real-time file watching | - | Yes | Yes | **Yes** |
| Smart diff (show what changed) | Yes | - | - | **Yes** |
| Author attribution | Manual | - | - | **Auto** |
| AI agent detection | - | - | - | **Yes** |
| Searchable change history | - | - | - | **Yes** |
| Whitespace filtering | Flags | - | - | **Yes** |
| JSON output (`--json`) | - | - | - | **Yes** |
| Single binary, zero deps | Yes | Yes | Build | **Yes** |

## Features

**Real-Time File Watching** — Native OS-level monitoring using inotify (Linux) and FSEvents (macOS). Debounced to avoid noise. Instant detection, zero CPU overhead.

**Smart Diff Engine** — Myers algorithm computes precise, line-level diffs. Optionally ignore whitespace-only changes or changes below a configurable threshold. See what actually matters.

**Git Attribution** — Automatically detects the current branch, runs git blame on modified lines, and identifies the author. Know exactly who changed each line, even before the commit.

**AI Agent Detection** — The killer feature. Scans `Co-Authored-By` headers, commit messages, environment variables, and TTY sessions to identify changes made by Claude, Cursor, Copilot, Windsurf, and Devin. Non-human changes are flagged with a `[AI AGENT]` badge.

**Searchable History** — Every change is logged to a local JSON-lines file. Query by author, file, date, branch, or agent. Full audit trail without git log archaeology.

**Zero Configuration** — Run `0diff init` and you're watching. Smart defaults for ignore patterns, extensions, and debouncing. Customize later with a simple TOML file. Works with any language, any project.

**JSON Output** — Every command supports `--json` for composability. Pipe 0diff output into your own tools, dashboards, or CI pipelines.

**Single Binary** — Written in Rust. Compiles to a ~2MB static binary. No runtime, no dependencies, no node_modules. Install with curl, run anywhere.

## Install

```bash
curl -fsSL https://0diff.dev/install.sh | sh
```

The installer auto-detects your OS and architecture. If no pre-built binary is available, it falls back to building from source via `cargo install`.

## Build from Source

Requires [Rust](https://rustup.rs) (1.70+).

```bash
# Clone and build
git clone https://github.com/zerosuite-inc/0diff.git
cd 0diff
cargo build --release

# Binary is at ./target/release/0diff (~2MB)

# Or install directly to PATH
cargo install --path .

# Run tests (44 tests, 0 failures)
cargo test
```

Or install directly from GitHub without cloning:

```bash
cargo install --git https://github.com/zerosuite-inc/0diff.git
```

## Quick Start

```bash
cd your-project
0diff init       # Creates .0diff.toml + .0diff/ directory
0diff watch      # Start watching — edit a file to see it in action
```

That's it. Edit a file and see the diff appear instantly in your terminal.

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
# Filter history by author
0diff log --author "alice"

# Show only AI agent changes
0diff log --agent "Claude"

# Last 20 changes as JSON
0diff log --limit 20 --json

# Diff a specific file
0diff diff src/main.rs

# Pipe to other tools
0diff log --json | jq '.[] | select(.agent != null)'
```

## AI Agent Detection

0diff is the first code tracking tool designed for the multi-agent era. It automatically detects changes made by AI coding assistants:

| Detection Method | What It Checks |
|---|---|
| **Commit analysis** | `Co-Authored-By` headers for Claude, Cursor, Copilot, Windsurf, Devin |
| **Commit messages** | Keywords and patterns in recent commit messages |
| **Environment variables** | `CLAUDE_CODE`, `CURSOR_SESSION`, `GITHUB_COPILOT`, and others |
| **TTY heuristic** | Flags non-interactive sessions as potential agent edits |

### Why this matters

You're running 5 Claude Code agents in parallel. Agent 3 refactors a utility function that Agents 1, 2, 4, and 5 all depend on. Everything breaks. Every agent commits under your name. Without 0diff, you'd have to read every diff manually to find which agent changed which file.

With 0diff:

```
[14:33:12] src/vm.rs  +3 -1  (⚡ Claude Agent on main)  [AI AGENT]
  @@ -100,1 +100,3 @@
  + fn optimize_bytecode(&mut self) {
  +     self.peephole_pass();
```

## Config

Run `0diff init` to create `.0diff.toml`:

```toml
[watch]
paths = ["src/", "app/", "entities/"]
ignore = ["target/", "node_modules/", ".git/", "*.log", ".flindb/"]
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

## Use Cases

### Solo Developer

You use Copilot, Cursor, Claude Code. They write code in your files while you're focused elsewhere. With 0diff, every modification is logged the instant it happens. No more "wait, did I write that or did the AI?"

### Dev Teams

Your team is distributed across time zones. Alice refactors a function at 2 AM. Bob starts building on the old API at 8 AM. With 0diff, every developer sees every change as it happens. Conflicts are prevented, not just resolved.

### Tech Leads & CTOs

You need to know: which critical files were modified today? How many changes hit production-sensitive code? Is the new hire touching files they shouldn't be? 0diff gives you the big picture without the noise.

### Critical Systems

Database migrations. Auth logic. Payment processing. Some files cannot afford silent changes. 0diff lets you configure elevated alerts on sensitive paths — zero tolerance for unreviewed modifications.

## Architecture

```
src/
├── main.rs        # CLI entry point (clap derive)
├── lib.rs         # Module re-exports
├── config.rs      # .0diff.toml parsing, defaults, should_watch()
├── watcher.rs     # File watcher (notify + debouncer + event loop)
├── differ.rs      # Diff engine (similar crate, Myers algorithm)
├── filter.rs      # Whitespace-only change filtering
├── git.rs         # Git blame, branch, author, commit parsing
├── history.rs     # JSON-lines change log with query/filter/rotate
├── agents.rs      # AI agent detection (Co-Authored-By, env, TTY)
└── output.rs      # Colored terminal output + JSON formatting
```

**Dependencies:** notify, similar, clap, toml, serde, colored, chrono, glob, ctrlc, thiserror — no async runtime, no heavy frameworks.

## Contributing

```bash
git clone https://github.com/zerosuite-inc/0diff.git
cd 0diff
cargo test       # 44 tests, should all pass
cargo run -- watch   # Test locally
```

Issues and PRs welcome.

## Roadmap

- **v0.2** — Background daemon mode, Slack/Discord/webhook notifications, daily digest
- **v0.3** — Comment filtering, critical file alerts, multi-agent collision detection
- **v1.0** — VS Code extension, GitHub Actions integration, cross-platform release builds

## Part of the ZeroSuite Ecosystem

Built by [ZeroSuite, Inc.](https://zerosuite.dev) — Cotonou, Benin

[0diff.dev](https://0diff.dev) · [flin.sh](https://flin.sh) · [zerosuite.dev](https://zerosuite.dev)

## License

[MIT](LICENSE) — free forever, open source.
