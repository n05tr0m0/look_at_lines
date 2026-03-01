use clap::Parser;

#[derive(Parser)]
#[command(
    name = "ll",
    version,
    about = "Look at Lines is a tool for listing files in a directory – quickly, beautifully, and with style!",
    after_help = r#"SORTING LOGIC:
You can combine sorting flags for quick access. The base flag is '-s' (enable sorting).
If no field is specified, it defaults to sorting by Name (ascending).

Fields:
  -n   Name (default)
  -S   Size
  -m   Modification time
  -b   Creation time (btime)
  -d   Descending order (reverses the result)

Examples of combined flags:
  ll -snd   -> Sort by Name, Descending
  ll -sS    -> Sort by Size, Ascending
  ll -smd   -> Sort by Modification time, Descending
  ll -sbd   -> Sort by Creation time, Descending

If you mix conflicting fields (e.g., -snS), the tool safely falls back to Name sorting.

FILTERING:
  -F   Show only regular files (excludes directories and symlinks)
  -D   Show only directories
  -H   Include hidden entries (names starting with '.')
       By default hidden entries are always excluded, even with -f.

  -F and -D are mutually exclusive.

OUTPUT TO FILE:
Flags -o and -O require an export format flag (-j, -x, -c, -p, -M) to be set.
Without a format flag, saving to a file is not possible.

Examples:
  ll -j -o list.json .      -> export JSON to list.json
  ll -c -O .                -> export CSV, auto-named file inside the directory
  ll -M -o README.md .      -> export Markdown table to README.md"#
)]
pub struct Cli {
    #[arg(short, help = "Enable full output mode (permissions, owner, group, timestamps)")]
    pub f: bool,

    #[arg(short, help = "Enable sorting options")]
    pub s: bool,

    #[arg(short, help = "Sort by name")]
    pub n: bool,

    #[arg(short = 'S', help = "Sort by size")]
    pub size: bool,

    #[arg(short, help = "Sort by modification time")]
    pub m: bool,

    #[arg(short, help = "Sort by creation time (btime)")]
    pub b: bool,

    #[arg(short, help = "Sort descending")]
    pub d: bool,

    /// Show only regular files (excludes directories and symlinks)
    #[arg(short = 'F', long = "files-only", group = "entry_filter")]
    pub files_only: bool,

    /// Show only directories
    #[arg(short = 'D', long = "dirs-only", group = "entry_filter")]
    pub dirs_only: bool,

    /// Include hidden entries (names starting with '.')
    #[arg(short = 'H', long = "hidden")]
    pub hidden: bool,

    /// Export to JSON
    #[arg(short = 'j', long, group = "export_mode")]
    pub json: bool,

    /// Export to XML
    #[arg(short = 'x', long, group = "export_mode")]
    pub xml: bool,

    /// Export to CSV (semicolon-delimited)
    #[arg(short = 'c', long, group = "export_mode")]
    pub csv: bool,

    /// Export to Plain Text (one entry per line)
    #[arg(short = 'p', long, group = "export_mode")]
    pub plain: bool,

    /// Export to Markdown table (GFM-compatible)
    #[arg(short = 'M', long, group = "export_mode")]
    pub markdown: bool,

    /// Write output to a named file.
    /// Requires an export format flag (-j, -x, -c, -p, -M).
    #[arg(short = 'o', long, conflicts_with = "auto_output")]
    pub output: Option<String>,

    /// Automatically generate an output filename inside the target directory.
    /// Requires an export format flag (-j, -x, -c, -p, -M).
    #[arg(short = 'O', long, conflicts_with = "output")]
    pub auto_output: bool,

    #[arg(default_value = ".", help = "Target directory to list")]
    pub path: String,
}
