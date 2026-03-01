# Look at Lines (ll)

**Look at Lines** (`ll`) is a modern, fast, and stylish alternative to the classic `ls` command, written in Rust.
It turns file listings into clean, readable tables and gives you powerful export and filtering tools on top.

[![CI](https://github.com/n05tr0m0/look_at_lines/actions/workflows/rust.yml/badge.svg)](https://github.com/n05tr0m0/look_at_lines/actions/workflows/rust.yml)
[![Release](https://github.com/n05tr0m0/look_at_lines/actions/workflows/release.yml/badge.svg)](https://github.com/n05tr0m0/look_at_lines/releases/latest)
![Rust Version](https://img.shields.io/badge/rust-1.74%2B-orange)
![License](https://img.shields.io/badge/license-Apache--2.0-blue)

---

## Why `ll` over `ls`?

- **Beautiful output** — heavy box-drawing characters, smart column sizing, full-width Unicode table.
- **Rich metadata** — permissions, ownership, size, and timestamps with `-f`.
- **Clipboard integration** — use `--copy` to copy the output to your clipboard on demand.
- **Multiple export formats** — JSON, XML, CSV (`;`-delimited), Plain Text, Markdown.
- **Smart sorting** — combine flags intuitively: `-s`, `-nd`, `-sd`, `-md`, `-bd`.
- **Entry filtering** — show only files (`-F`), only directories (`-D`), or include hidden entries (`-H`).
- **Unicode-aware** — file names in Cyrillic, CJK, Arabic and other scripts truncate correctly.
- **Dark/light theme** — border and text colours adapt to your terminal's colour scheme automatically.

---

## Installation

### Pre-built binaries (recommended)

Pre-built binaries for every release are available on the [Releases page](https://github.com/n05tr0m0/look_at_lines/releases/latest).

| Platform | Architecture | File |
|----------|-------------|------|
| macOS | Apple Silicon (M1/M2/M3) | `ll-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| macOS | Intel | `ll-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| Linux | x86_64 | `ll-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 | `ll-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz` |

The one-liner below auto-detects your OS and CPU, downloads the latest release, and installs the binary to `/usr/local/bin`:

```bash
set -euo pipefail
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case "$ARCH" in
  x86_64)  ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "unsupported architecture: $ARCH" && exit 1 ;;
esac
case "$OS" in
  darwin) TARGET="${ARCH}-apple-darwin" ;;
  linux)  TARGET="${ARCH}-unknown-linux-gnu" ;;
  *) echo "unsupported OS: $OS" && exit 1 ;;
esac
TAG=$(curl -fsSL https://api.github.com/repos/n05tr0m0/look_at_lines/releases/latest | grep '"tag_name"' | cut -d'"' -f4)
curl -fsSL "https://github.com/n05tr0m0/look_at_lines/releases/download/${TAG}/ll-${TAG}-${TARGET}.tar.gz" \
  | tar -xz
chmod +x ll && sudo mv ll /usr/local/bin/
echo "installed: $(ll --version)"
```

### Build from source

Requires Rust 1.74+.

```bash
git clone https://github.com/n05tr0m0/look_at_lines.git
cd look_at_lines
cargo install --path .
```

Or build manually and copy the binary:

```bash
cargo build --release
sudo cp target/release/ll /usr/local/bin/
```

Verify:

```bash
ll --version
```

---

## Quick start

```bash
ll              # list current directory
ll src/         # list src/
ll -f src/      # full mode: permissions, owner, group, timestamps
ll -H .         # include hidden entries (dotfiles)
ll -s src/      # sort by size ascending
ll -F -j src/   # only files, export as JSON
ll -j src/ --copy   # export JSON to stdout and copy to clipboard
```

---

## All flags

### Display

| Flag | Long form | Description |
|------|-----------|-------------|
| `-f` | — | **Full mode.** Shows permissions in `rwxrwxrwx` format, owner (user + uid), group (group + gid), creation time, and modification time. Without `-f` only name, type, compact octal mode, size, and modified date are shown. |

### Sorting

Sorting is always active. Pick a field flag to select the sort key. With no field flag the list is sorted by name. Combining two field flags (e.g. `-ns`) also falls back to name.

| Flag | Description |
|------|-------------|
| `-n` | Sort by name (default) |
| `-s` | Sort by size |
| `-m` | Sort by modification time |
| `-b` | Sort by creation time |
| `-d` | Descending order (reverses the result) |

Examples:

```bash
ll              # sort by name, ascending (default)
ll -nd          # sort by name, descending
ll -s           # sort by size, ascending
ll -sd          # sort by size, descending
ll -m           # sort by modification time, ascending
ll -md          # sort by modification time, descending
ll -bd          # sort by creation time, descending
```

### Filtering

`-F` and `-D` are mutually exclusive.

| Flag | Long form | Description |
|------|-----------|-------------|
| `-F` | `--files-only` | Show only regular files. Directories and symlinks are excluded. |
| `-D` | `--dirs-only` | Show only directories. |
| `-H` | `--hidden` | Include hidden entries (names starting with `.`). Hidden entries are **always excluded by default**, even in full mode (`-f`). |

### Export formats

Only one format flag can be active per invocation.

| Flag | Long form | Format | Notes |
|------|-----------|--------|-------|
| `-j` | `--json` | JSON | Pretty-printed via `serde_json`. `mode` is a 3-digit octal string (`"644"`). |
| `-x` | `--xml` | XML | Indented via `quick-xml`. |
| `-c` | `--csv` | CSV | Semicolon (`;`) delimited. Header row included. `mode` as octal string. |
| `-p` | `--plain` | Plain Text | One entry per line. Symlinks rendered as `name -> target`. |
| `-M` | `--markdown` | Markdown | GFM-compatible table. Compact: Name, Type, Mode, Size, Modified. Full (`-f`): adds User, Group, Created. |

### Clipboard

| Flag | Long form | Description |
|------|-----------|-------------|
| — | `--copy` | Copy the output to the system clipboard. Works with any export format, or standalone (copies a plain newline-separated name list). |

Without `--copy` nothing is copied automatically. Use it explicitly:

```bash
ll --copy               # copy plain name list to clipboard
ll -j --copy            # export JSON to stdout and also copy it
ll -j -o list.json --copy   # save JSON to file, copy it, show table
```

### Output to file

Saving to a file **requires** an export format flag. Without one, the tool prints a descriptive error and exits with code `1`.

| Flag | Long form | Description |
|------|-----------|-------------|
| `-o <file>` | `--output <file>` | Write to the specified file path. |
| `-O` | `--auto-output` | Auto-generate a filename inside the target directory (e.g. `src.json`). On collision a counter is appended: `src(1).json`, `src(2).json`, … |

**Behaviour depends on whether a file destination is given:**

| Invocation | Table in terminal | Data |
|------------|-------------------|------|
| `ll -j` (no `-o`/`-O`) | — | JSON printed to **stdout** only — pipe-friendly |
| `ll -j -o file.json` | ✓ printed first | written to `file.json`; path printed to stdout |
| `ll -j -O` | ✓ printed first | written to auto-named file; path printed to stdout |

Without `-o`/`-O` the terminal table is **suppressed** so that the raw export data can be piped cleanly:

```bash
ll -j src/ | jq '.[].name'          # pipe JSON directly — no table noise
ll -p src/ | fzf                     # pipe plain-text names into fzf
ll -c src/ > snapshot.csv           # redirect CSV to a file via the shell
```

With `-o`/`-O` the table is printed first, then the file is written and its path appears as the last line of stdout:

```bash
ll -j -o list.json .          # table in terminal + write JSON to ./list.json
ll -c -O src/                 # table in terminal + write CSV to src/src.csv (auto-named)
ll -M -o FILELIST.md ~/docs   # table in terminal + write Markdown table to FILELIST.md
ll -p -O .                    # table in terminal + write plain text, auto-named
```

---

## Output examples

### Basic output (default)

```bash
ll src/
```

```
┏━━━━━━━━━━━━━━┳━━━━━━┳━━━━━━━┳━━━━━━┳━━━━━━━━━━━━━━━━━━━━━┓
┃ Name         ┃ Perm ┃  Size ┃ Type ┃            Modified ┃
┣━━━━━━━━━━━━━━╋━━━━━━╋━━━━━━━╋━━━━━━╋━━━━━━━━━━━━━━━━━━━━━┫
┃ cli.rs       ┃  644 ┃  2.7K ┃    f ┃ 2025-06-01 12:34:56 ┃
┃ export.rs    ┃  644 ┃  7.4K ┃    f ┃ 2025-06-01 12:34:56 ┃
┃ file_info.rs ┃  644 ┃  5.5K ┃    f ┃ 2025-06-01 11:58:11 ┃
┃ main.rs      ┃  644 ┃  5.4K ┃    f ┃ 2025-06-01 12:34:56 ┃
┃ theme.rs     ┃  644 ┃  1.4K ┃    f ┃ 2025-06-01 11:58:11 ┃
┃ ui.rs        ┃  644 ┃ 20.7K ┃    f ┃ 2025-06-01 11:58:11 ┃
┗━━━━━━━━━━━━━━┻━━━━━━┻━━━━━━━┻━━━━━━┻━━━━━━━━━━━━━━━━━━━━━┛
```

Colour coding (dark theme): directories — blue, executable files — **bold gold** (`*f`), regular files — white, symlinks — white.
On a light terminal the colours shift: directories — dark green, executables — **bold dark red**, regular files — near-black.
The `Mode` column shows a compact 3-digit octal string; the `Type` column uses single-character codes (`d` dir, `l` symlink, `f` file, `*f` executable).

### Full mode (`-f`)

```bash
ll -f src/
```

```
┏━━━━━━━━━━━━━━┳━━━━━━━━━━━┳━━━━━━━┳━━━━━━┳━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━┓
┃ Name         ┃      Perm ┃  Size ┃ Type ┃            Modified ┃             Created ┃            User ┃      Group ┃
┣━━━━━━━━━━━━━━╋━━━━━━━━━━━╋━━━━━━━╋━━━━━━╋━━━━━━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━┫
┃ cli.rs       ┃ rw-r--r-- ┃  2742 ┃    f ┃ 2025-06-01 12:34:56 ┃ 2025-05-10 09:00:00 ┃ alice (1000)    ┃ staff (20) ┃
┃ export.rs    ┃ rw-r--r-- ┃  7578 ┃    f ┃ 2025-06-01 12:34:56 ┃ 2025-05-10 09:00:00 ┃ alice (1000)    ┃ staff (20) ┃
┃ file_info.rs ┃ rw-r--r-- ┃  5521 ┃    f ┃ 2025-06-01 11:58:11 ┃ 2025-05-10 09:00:00 ┃ alice (1000)    ┃ staff (20) ┃
┃ main.rs      ┃ rw-r--r-- ┃  5890 ┃    f ┃ 2025-06-01 12:34:56 ┃ 2025-05-10 09:00:00 ┃ alice (1000)    ┃ staff (20) ┃
┃ theme.rs     ┃ rw-r--r-- ┃  1422 ┃    f ┃ 2025-06-01 11:58:11 ┃ 2025-05-10 09:00:00 ┃ alice (1000)    ┃ staff (20) ┃
┃ ui.rs        ┃ rw-r--r-- ┃ 19906 ┃    f ┃ 2025-06-01 11:58:11 ┃ 2025-05-10 09:00:00 ┃ alice (1000)    ┃ staff (20) ┃
┗━━━━━━━━━━━━━━┻━━━━━━━━━━━┻━━━━━━━┻━━━━━━┻━━━━━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━┛
```

Full mode switches `Perm` to human-readable `rwxrwxrwx` format and raw byte `Size`, then appends `Created`, `User`, and `Group` after `Modified`.

---

## Export examples

### JSON (`-j`)

```bash
ll -j src/
```

```json
[
  {
    "name": "cli.rs",
    "target": null,
    "is_dir": false,
    "is_symlink": false,
    "is_exec": false,
    "mode": "644",
    "size": 2742,
    "uid": 1000,
    "gid": 20,
    "created": "2025-05-10T09:00:00+03:00",
    "modified": "2025-06-01T12:34:56+03:00"
  },
  {
    "name": "export.rs",
    "target": null,
    "is_dir": false,
    "is_symlink": false,
    "is_exec": false,
    "mode": "644",
    "size": 7578,
    "uid": 1000,
    "gid": 20,
    "created": "2025-05-10T09:00:00+03:00",
    "modified": "2025-06-01T12:34:56+03:00"
  }
]
```

`mode` is always a 3-digit octal string. `target` is `null` for non-symlinks. Dates are RFC 3339.

Save to a file:

```bash
ll -j -o manifest.json src/
ll -j -O src/             # auto-named → src/src.json
```

### Plain Text (`-p`)

```bash
ll -p src/
```

```
cli.rs
export.rs
file_info.rs
main.rs
ui.rs
```

Symlinks are rendered with an arrow:

```bash
ll -p /etc/
```

```
hostname
localtime -> /usr/share/zoneinfo/Europe/London
resolv.conf
```

Save to a file:

```bash
ll -p -o filelist.txt src/
ll -p -O src/             # auto-named → src/src.txt
```

### Markdown (`-M`)

```bash
ll -M src/
```

```
| Name         | Type | Mode | Size  | Modified            |
| ------------ | ---- | ---- | ----- | ------------------- |
| cli.rs       | f    | 644  | 2.7K  | 2025-06-01 12:34:56 |
| export.rs    | f    | 644  | 7.4K  | 2025-06-01 12:34:56 |
| file_info.rs | f    | 644  | 5.5K  | 2025-06-01 11:58:11 |
| main.rs      | f    | 644  | 5.4K  | 2025-06-01 12:34:56 |
| ui.rs        | f    | 644  | 20.7K | 2025-06-01 11:58:11 |
```

Executable files appear as `*file` in the Type column. With `-f` three extra columns are added — `Created`, `User`, and `Group`:

```bash
ll -fM src/
```

```
| Name         | Type | Mode | Size  | Modified            | Created             | User         | Group      |
| ------------ | ---- | ---- | ----- | ------------------- | ------------------- | ------------ | ---------- |
| cli.rs       | f    | 644  | 2.7K  | 2025-06-01 12:34:56 | 2025-05-10 09:00:00 | alice (1000) | staff (20) |
| export.rs    | f    | 644  | 7.4K  | 2025-06-01 12:34:56 | 2025-05-10 09:00:00 | alice (1000) | staff (20) |
| file_info.rs | f    | 644  | 5.5K  | 2025-06-01 11:58:11 | 2025-05-10 09:00:00 | alice (1000) | staff (20) |
| main.rs      | f    | 644  | 5.4K  | 2025-06-01 12:34:56 | 2025-05-10 09:00:00 | alice (1000) | staff (20) |
| ui.rs        | f    | 644  | 20.7K | 2025-06-01 11:58:11 | 2025-05-10 09:00:00 | alice (1000) | staff (20) |
```

Save to a file:

```bash
ll -M -o FILELIST.md src/
ll -fM -O src/            # auto-named → src/src.md  (full mode)
```

---

## Shell alias conflict (oh-my-zsh)

oh-my-zsh defines `alias ll='ls -lh'` in `~/.oh-my-zsh/lib/directories.zsh`.
Add the following line to `~/.zshrc` **after** the `source $ZSH/oh-my-zsh.sh` line:

```zsh
unalias ll || true
```

This silently removes the built-in alias without error if it does not exist.

---

## Development

```bash
cargo build              # debug build
cargo build --release    # optimised release binary
cargo run -- [OPTIONS]   # run without installing
cargo test               # all tests (unit + integration)
cargo clippy             # lint
cargo fmt                # format
```

### Releasing a new version

Use the release script — it bumps the version, runs all checks, commits, and tags in one step:

```bash
./scripts/release.sh 1.2.0
```

The script:
1. Validates the version format (`X.Y.Z`)
2. Checks the working tree is clean
3. Updates `version` in `Cargo.toml`
4. Runs `cargo check`, `cargo test`, `cargo clippy`
5. Commits `Cargo.toml` and `Cargo.lock` as `chore: release v1.2.0`
6. Creates an annotated git tag `v1.2.0`
7. Prints the push command — nothing is pushed automatically

When you're ready to publish:

```bash
git push origin main && git push origin v1.2.0
```

Pushing the tag triggers the **Release** CI workflow, which:
- Validates that the tag matches the version in `Cargo.toml`
- Runs the full test suite and clippy
- Builds binaries for all four targets in parallel
- Creates a GitHub Release with the archives attached

---

## License

Apache License, Version 2.0 — see [LICENSE](LICENSE).