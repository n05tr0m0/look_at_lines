use chrono::{DateTime, Local};
use owo_colors::OwoColorize;
use serde::{Serialize, Serializer};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::PathBuf;

/// Serializes a Unix mode bitmask as a 3-digit octal string (e.g. `0o644` → `"644"`).
fn serialize_mode_octal<S: Serializer>(mode: &u32, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&format!("{:03o}", mode & 0o777))
}

#[derive(Serialize)]
pub struct FileInfo {
    pub name: String,
    pub target: Option<String>,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub is_exec: bool,
    #[serde(serialize_with = "serialize_mode_octal")]
    pub mode: u32,
    pub size: u64,
    pub uid: u32,
    pub gid: u32,
    pub created: Option<DateTime<Local>>,
    pub modified: Option<DateTime<Local>>,
}

impl FileInfo {
    /// Builds a `FileInfo` from a path using `symlink_metadata` to avoid
    /// following symlinks eagerly. Directory detection checks the symlink
    /// target when applicable so that `dir -> /some/dir` is correctly
    /// classified as a directory.
    pub fn from_path(path: PathBuf, resolve_symlinks: bool) -> Option<Self> {
        let metadata = fs::symlink_metadata(&path).ok()?;
        let file_name = path.file_name()?.to_string_lossy().into_owned();

        let is_symlink = metadata.is_symlink();
        let target = if is_symlink && resolve_symlinks {
            fs::read_link(&path).ok().map(|p| p.to_string_lossy().into_owned())
        } else {
            None
        };

        let target_metadata = if is_symlink { fs::metadata(&path).ok() } else { None };
        let is_dir = target_metadata
            .as_ref()
            .map(|m| m.is_dir())
            .unwrap_or(metadata.is_dir());

        let mode = metadata.permissions().mode();
        let is_exec = !is_dir && (mode & 0o111 != 0);

        Some(Self {
            name: file_name,
            target,
            is_dir,
            is_symlink,
            is_exec,
            mode,
            size: metadata.len(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            created: metadata.created().ok().map(DateTime::from),
            modified: metadata.modified().ok().map(DateTime::from),
        })
    }

    pub fn display_name(&self, is_full_mode: bool, palette: &crate::theme::Palette) -> String {
        let mut display = self.name.clone();

        if is_full_mode && self.is_symlink {
            if let Some(ref tgt) = self.target {
                display = format!("{} --> {}", self.name, tgt);
            }
        }

        if self.is_dir {
            display.color(palette.dir).to_string()
        } else if self.is_exec {
            display.color(palette.exec).bold().to_string()
        } else {
            display.color(palette.file).to_string()
        }
    }
}

pub fn format_size_human(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.1}G", b / GB)
    } else if b >= MB {
        format!("{:.1}M", b / MB)
    } else if b >= KB {
        format!("{:.1}K", b / KB)
    } else {
        format!("{}B", bytes)
    }
}

pub fn format_permissions_rwx(mode: u32) -> String {
    let user = format_triplet(mode >> 6);
    let group = format_triplet(mode >> 3);
    let other = format_triplet(mode);
    format!("{}{}{}", user, group, other)
}

fn format_triplet(mode: u32) -> String {
    let r = if mode & 4 != 0 { "r" } else { "-" };
    let w = if mode & 2 != 0 { "w" } else { "-" };
    let x = if mode & 1 != 0 { "x" } else { "-" };
    format!("{}{}{}", r, w, x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_human() {
        assert_eq!(format_size_human(0), "0B");
        assert_eq!(format_size_human(500), "500B");
        assert_eq!(format_size_human(1023), "1023B");
        assert_eq!(format_size_human(1024), "1.0K");
        assert_eq!(format_size_human(1536), "1.5K");
        assert_eq!(format_size_human(1024 * 1024), "1.0M");
        assert_eq!(format_size_human(1024 * 1024 * 1024), "1.0G");
        assert_eq!(format_size_human(1024 * 1024 + 500 * 1024), "1.5M");
    }

    #[test]
    fn test_format_permissions_rwx() {
        assert_eq!(format_permissions_rwx(0o777), "rwxrwxrwx");
        assert_eq!(format_permissions_rwx(0o755), "rwxr-xr-x");
        assert_eq!(format_permissions_rwx(0o644), "rw-r--r--");
        assert_eq!(format_permissions_rwx(0o000), "---------");
        assert_eq!(format_permissions_rwx(0o111), "--x--x--x");
        assert_eq!(format_permissions_rwx(0o600), "rw-------");
    }
}
