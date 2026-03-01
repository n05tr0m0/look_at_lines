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
- **Clipboard integration** — exporting without `-o`/`-O` prints to stdout *and* copies to clipboard automatically.
- **Multiple export formats** — JSON, XML, CSV (`;`-delimited), Plain Text, Markdown.
- **Smart sorting** — combine flags intuitively: `-sS`, `-smd`, `-sbd`.
- **Entry filtering** — show only files (`-F`), only directories (`-D`), or include hidden entries (`-H`).
- **Unicode-aware** — file names in Cyrillic, CJK, Arabic and other scripts truncate correctly.

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
ll -sS src/     # sort by size ascending
ll -F -j src/   # only files, export as JSON
```

---

## All flags

### Display

| Flag | Long form | Description |
|------|-----------|-------------|
| `-f` | — | **Full mode.** Shows permissions in `rwxrwxrwx` format, owner (user + uid), group (group + gid), creation time, and modification time. Without `-f` only name, type, compact octal mode, size, and modified date are shown. |

### Sorting

Sorting is opt-in: add `-s` to enable it, then pick a field. If no field flag is given, sorting falls back to name. Mixing two field flags (e.g. `-snS`) also falls back to name.

| Flag | Description |
|------|-------------|
| `-s` | Enable sorting |
| `-n` | Sort by name (default field) |
| `-S` | Sort by size |
| `-m` | Sort by modification time |
| `-b` | Sort by creation time |
| `-d` | Descending order (reverses the result) |

Examples:

```bash
ll -sn          # sort by name, ascending (same as ll -s)
ll -snd         # sort by name, descending
ll -sS          # sort by size, ascending
ll -sSd         # sort by size, descending
ll -sm          # sort by modification time, ascending
ll -smd         # sort by modification time, descending
ll -sbd         # sort by creation time, descending
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

Without `-o`/`-O` the result is printed to **stdout** and **copied to the clipboard**.

### Output to file

Saving to a file **requires** an export format flag. Without one, the tool prints a descriptive error and exits with code `1`.

| Flag | Long form | Description |
|------|-----------|-------------|
| `-o <file>` | `--output <file>` | Write to the specified file path. |
| `-O` | `--auto-output` | Auto-generate a filename inside the target directory (e.g. `src.json`). On collision a counter is appended: `src(1).json`, `src(2).json`, … |

```bash
ll -j -o list.json .          # write JSON  to ./list.json
ll -c -O src/                 # write CSV   to src/src.csv  (auto-named)
ll -M -o FILELIST.md ~/docs   # write Markdown table to FILELIST.md
ll -p -O .                    # write plain text to ./.<dirname>.txt (auto-named)
```

---

## Output examples

### Basic output (default)

```bash
ll src/
```

```
┏━━━━━━━━━━━━━━┳━━━━━┳━━━━━┳━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━┓
┃ Name         ┃ Exe ┃ Mode┃ Size  ┃ Modified            ┃
┣━━━━━━━━━━━━━━╋━━━━━╋━━━━━╋━━━━━━━╋━━━━━━━━━━━━━━━━━━━━━┫
┃ cli.rs       ┃     ┃ 644 ┃ 2.7K  ┃ 2025-06-01 12:34:56 ┃
┃ export.rs    ┃     ┃ 644 ┃ 7.4K  ┃ 2025-06-01 12:34:56 ┃
┃ file_info.rs ┃     ┃ 644 ┃ 5.5K  ┃ 2025-06-01 11:58:11 ┃
┃ main.rs      ┃     ┃ 644 ┃ 5.4K  ┃ 2025-06-01 12:34:56 ┃
┃ ui.rs        ┃     ┃ 644 ┃ 20.7K ┃ 2025-06-01 11:58:11 ┃
┗━━━━━━━━━━━━━━┻━━━━━┻━━━━━┻━━━━━━━┻━━━━━━━━━━━━━━━━━━━━━┛
```

Directories are highlighted in blue, executable files in bold white.
The `Mode` column shows a compact 3-digit octal string.

### Full mode (`-f`)

```bash
ll -f src/
```

```
┏━━━━━━━━━━━━━━┳━━━━━┳━━━━━━━━━━━┳━━━━━━━┳━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━┓
┃ Name         ┃ Exe ┃ Perm      ┃ Size  ┃ User            ┃ Group          ┃ Created             ┃ Modified            ┃
┣━━━━━━━━━━━━━━╋━━━━━╋━━━━━━━━━━━╋━━━━━━━╋━━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━━━━━━━━╋━━━━━━━━━━━━━━━━━━━━━┫
┃ cli.rs       ┃     ┃ rw-r--r-- ┃ 2.7K  ┃ alice (1000)    ┃ staff (20)     ┃ 2025-05-10 09:00:00 ┃ 2025-06-01 12:34:56 ┃
┃ export.rs    ┃     ┃ rw-r--r-- ┃ 7.4K  ┃ alice (1000)    ┃ staff (20)     ┃ 2025-05-10 09:00:00 ┃ 2025-06-01 12:34:56 ┃
┃ file_info.rs ┃     ┃ rw-r--r-- ┃ 5.5K  ┃ alice (1000)    ┃ staff (20)     ┃ 2025-05-10 09:00:00 ┃ 2025-06-01 11:58:11 ┃
┃ main.rs      ┃     ┃ rw-r--r-- ┃ 5.4K  ┃ alice (1000)    ┃ staff (20)     ┃ 2025-05-10 09:00:00 ┃ 2025-06-01 12:34:56 ┃
┃ ui.rs        ┃     ┃ rw-r--r-- ┃ 20.7K ┃ alice (1000)    ┃ staff (20)     ┃ 2025-05-10 09:00:00 ┃ 2025-06-01 11:58:11 ┃
┗━━━━━━━━━━━━━━┻━━━━━┻━━━━━━━━━━━┻━━━━━━━┻━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━━━━━━━┛
```

Full mode replaces the compact `Mode` column with human-readable `Perm` (`rwxrwxrwx`) and adds `User`, `Group`, and `Created`.

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
| cli.rs       | file | 644  | 2.7K  | 2025-06-01 12:34:56 |
| export.rs    | file | 644  | 7.4K  | 2025-06-01 12:34:56 |
| file_info.rs | file | 644  | 5.5K  | 2025-06-01 11:58:11 |
| main.rs      | file | 644  | 5.4K  | 2025-06-01 12:34:56 |
| ui.rs        | file | 644  | 20.7K | 2025-06-01 11:58:11 |
```

With `-f` three extra columns are added — `User`, `Group`, and `Created`:

```bash
ll -fM src/
```

```
| Name         | Type | Mode | Size  | Modified            | User         | Group      | Created             |
| ------------ | ---- | ---- | ----- | ------------------- | ------------ | ---------- | ------------------- |
| cli.rs       | file | 644  | 2.7K  | 2025-06-01 12:34:56 | alice (1000) | staff (20) | 2025-05-10 09:00:00 |
| export.rs    | file | 644  | 7.4K  | 2025-06-01 12:34:56 | alice (1000) | staff (20) | 2025-05-10 09:00:00 |
| file_info.rs | file | 644  | 5.5K  | 2025-06-01 11:58:11 | alice (1000) | staff (20) | 2025-05-10 09:00:00 |
| main.rs      | file | 644  | 5.4K  | 2025-06-01 12:34:56 | alice (1000) | staff (20) | 2025-05-10 09:00:00 |
| ui.rs        | file | 644  | 20.7K | 2025-06-01 11:58:11 | alice (1000) | staff (20) | 2025-05-10 09:00:00 |
```

Save to a file:

```bash
ll -M -o FILELIST.md src/
ll -fM -O src/            # auto-named → src/src.md  (full mode)
```

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

The release profile uses `lto = "fat"`, `opt-level = "z"`, `codegen-units = 1`, and `strip = true` for a small, fast binary.

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
