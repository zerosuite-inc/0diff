# 0diff

**Know who changed what. Even when it's not human.**

Real-time code modification tracking for the multi-agent era.

## Install

```bash
curl -fsSL https://0diff.dev/install.sh | sh
```

Or build from source:

```bash
cargo install --path .
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

## AI Agent Detection

0diff automatically detects changes made by AI agents:

- **Commit analysis** — scans `Co-Authored-By` headers and commit messages for Claude, Cursor, Copilot, Windsurf, Devin
- **Environment detection** — checks for agent-specific environment variables
- **TTY heuristic** — flags non-interactive sessions as potential agent edits

## Config

Run `0diff init` to create `.0diff.toml`:

```toml
[watch]
paths = ["src/", "app/", "entities/"]
ignore = ["target/", "node_modules/", ".git/"]
extensions = ["rs", "flin", "ts", "js", "py", "go", "java"]
debounce_ms = 500

[agents]
detect_patterns = ["Claude", "Cursor", "Copilot", "Windsurf", "Devin"]
tag_non_human = true
```

## Part of the FLIN Ecosystem

Built by ZeroSuite, Inc. — Cotonou, Benin

[flin.sh](https://flin.sh) | [0diff.dev](https://0diff.dev)

## License

MIT
