mod cli;
mod export;
mod file_info;
mod ui;

use anyhow::{Context, Result};
use arboard::Clipboard;
use clap::Parser;
use cli::Cli;
use export::{export_data, ExportFormat};
use file_info::FileInfo;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let export_format: Option<ExportFormat> = if cli.json {
        Some(ExportFormat::Json)
    } else if cli.xml {
        Some(ExportFormat::Xml)
    } else if cli.csv {
        Some(ExportFormat::Csv)
    } else if cli.plain {
        Some(ExportFormat::PlainText)
    } else if cli.markdown {
        Some(ExportFormat::Markdown)
    } else {
        None
    };

    if export_format.is_none() && (cli.output.is_some() || cli.auto_output) {
        eprintln!("error: saving to a file requires an export format flag.");
        eprintln!();
        eprintln!("Choose one of:");
        eprintln!("  -j / --json     Export as JSON");
        eprintln!("  -x / --xml      Export as XML");
        eprintln!("  -c / --csv      Export as CSV (semicolon-delimited)");
        eprintln!("  -p / --plain    Export as plain text");
        eprintln!("  -M / --markdown Export as Markdown table");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  ll -j -o list.json .");
        eprintln!("  ll -c -O .");
        eprintln!("  ll -M -o README.md .");
        std::process::exit(1);
    }

    let resolve_symlinks = cli.f || export_format.is_some();

    let mut files = Vec::new();
    let entries = fs::read_dir(&cli.path).with_context(|| format!("Failed to read directory: '{}'", cli.path))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        if !cli.hidden && file_name.starts_with('.') {
            continue;
        }

        if let Some(info) = FileInfo::from_path(path, resolve_symlinks) {
            if cli.files_only && (info.is_dir || info.is_symlink) {
                continue;
            }
            if cli.dirs_only && !info.is_dir {
                continue;
            }
            files.push(info);
        }
    }

    if cli.s {
        let active_sort_flags = [cli.n, cli.size, cli.m, cli.b].iter().filter(|&&x| x).count();

        if active_sort_flags > 1 {
            files.sort_by(|a, b| a.name.cmp(&b.name));
        } else if cli.size {
            files.sort_by(|a, b| a.size.cmp(&b.size));
        } else if cli.m {
            files.sort_by(|a, b| a.modified.cmp(&b.modified));
        } else if cli.b {
            files.sort_by(|a, b| match (a.created, b.created) {
                (Some(ta), Some(tb)) => ta.cmp(&tb),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            });
        } else {
            files.sort_by(|a, b| a.name.cmp(&b.name));
        }

        if cli.d {
            files.reverse();
        }
    } else {
        files.sort_by(|a, b| a.name.cmp(&b.name));
    }

    if let Some(format) = export_format {
        let data = export_data(&files, format, cli.f)?;

        let output_path: Option<PathBuf> = if let Some(ref path) = cli.output {
            Some(PathBuf::from(path))
        } else if cli.auto_output {
            let dir_path = Path::new(&cli.path);

            let absolute_path = if dir_path == Path::new(".") || dir_path.as_os_str().is_empty() {
                std::env::current_dir()?
            } else {
                dir_path
                    .canonicalize()
                    .with_context(|| format!("Failed to resolve path: '{}'", dir_path.display()))?
            };

            let dir_name = absolute_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            let extension = format.extension();
            let base_name = if dir_name.is_empty() || dir_name == ".." {
                "export".to_string()
            } else {
                dir_name
            };

            let mut candidate = absolute_path.join(format!("{}.{}", base_name, extension));
            let mut counter = 1u32;
            while candidate.exists() {
                candidate = absolute_path.join(format!("{}({}).{}", base_name, counter, extension));
                counter += 1;
            }
            Some(candidate)
        } else {
            None
        };

        if let Some(path) = output_path {
            let mut file =
                File::create(&path).with_context(|| format!("Failed to create output file: '{}'", path.display()))?;
            file.write_all(data.as_bytes())?;
            println!("{}", path.display());
        } else {
            print!("{}", data);
            let _ = std::io::stdout().flush();

            match Clipboard::new() {
                Ok(mut clipboard) => {
                    if clipboard.set_text(data).is_err() {
                        eprintln!("(Failed to copy output to clipboard)");
                    } else {
                        eprintln!("\n==[Copied to clipboard!]==");
                    }
                }
                Err(_) => {
                    eprintln!("(Clipboard unavailable)");
                }
            }
        }
    } else {
        ui::render_table(&files, cli.f);
    }

    Ok(())
}
