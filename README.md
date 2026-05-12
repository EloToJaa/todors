# todors

`todors` is a Rust command-line todo manager focused on parity with
[`todoman`](https://todoman.readthedocs.io/), backed by VTODO files and a local
SQLite cache.

## What it does

- Reads todo lists from calendar directories (ICS/VTODO files).
- Lists, creates, edits, completes, cancels, and deletes todos.
- Supports machine-friendly output with `--porcelain`.
- Stores cache data in SQLite for fast repeated access.

## Build and run

```bash
# Build
cargo build

# Run
cargo run -- <command>

# Examples
cargo run -- list
cargo run -- new "Buy milk"
```

If you use Nix:

```bash
nix build
nix run -- list
```

## Configuration

By default, `todors` loads config from:

- `$XDG_CONFIG_HOME/todors/config.toml`, or
- `~/.config/todors/config.toml`

You can also pass a config file explicitly:

```bash
todors --config /path/to/config.toml list
```

Minimal example:

```toml
path = "~/.local/share/calendars/*"
cache_path = "~/.cache/todors/cache.sqlite3"
default_command = "list"
```

Supported config keys:

- `path`
- `cache_path`
- `default_list`
- `default_due`
- `date_format`
- `time_format`
- `dt_separator`
- `default_command`
- `color`
- `humanize`
- `startable`

## CLI overview

Top-level commands:

- `list`
- `new`
- `show`
- `edit`
- `done`
- `undo`
- `cancel`
- `delete`
- `flush`
- `lists`
- `repl`
- `path`
- `move`
- `copy`

See all options:

```bash
todors --help
todors <command> --help
```

## Basic usage

```bash
# List open tasks
todors list

# Create a task in the default list
todors new "Write weekly report"

# Mark task 3 as done
todors done 3

# Undo completion
todors undo 3

# Output JSON-like porcelain format
todors --porcelain list
```

## Project status

This project is early-stage and evolving. Command behavior aims to stay close
to `todoman`, and tests include parity checks when `todoman` is installed.

## Releases

Releases are automated with GoReleaser when a tag matching `v*` is pushed.

### Create a release

```bash
# 1) Ensure local checks pass
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test

# 2) Bump version in Cargo.toml, commit, and tag
git tag -a v0.1.0 -m "Release v0.1.0"

# 3) Push commits and tag
git push origin main
git push origin v0.1.0
```

The release workflow builds and uploads archives for:

- Linux `x86_64` and `aarch64`
- macOS `x86_64` and `aarch64`
- Windows `x86_64`

Each release includes per-platform archives and a `checksums.txt` file.

### Dry-run locally

```bash
goreleaser check
goreleaser release --snapshot --clean
```
