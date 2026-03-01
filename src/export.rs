use crate::file_info::{self, FileInfo};
use anyhow::{Context, Result};
use serde::Serialize;

#[derive(Clone, Copy)]
pub enum ExportFormat {
    Json,
    Xml,
    Csv,
    PlainText,
    Markdown,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Xml => "xml",
            Self::Csv => "csv",
            Self::PlainText => "txt",
            Self::Markdown => "md",
        }
    }
}

/// Escapes `|` so it does not break GFM table cell boundaries.
#[inline]
fn md_escape(s: &str) -> String {
    s.replace('|', "\\|")
}

pub fn export_data(files: &[FileInfo], format: ExportFormat, is_full_mode: bool) -> Result<String> {
    match format {
        ExportFormat::Json => serde_json::to_string_pretty(files).context("Failed to serialize files to JSON"),

        ExportFormat::Xml => {
            let mut buffer = String::new();
            let mut ser = quick_xml::se::Serializer::new(&mut buffer);
            ser.indent(' ', 2);

            #[derive(Serialize)]
            #[serde(rename = "Files")]
            struct FilesWrapper<'a> {
                #[serde(rename = "File")]
                files: &'a [FileInfo],
            }
            let wrapper = FilesWrapper { files };
            wrapper.serialize(ser).context("Failed to serialize files to XML")?;

            Ok(buffer)
        }

        ExportFormat::Csv => {
            let mut wtr = csv::WriterBuilder::new().delimiter(b';').from_writer(vec![]);

            for f in files {
                wtr.serialize(f).context("Failed to serialize CSV record")?;
            }

            let data = String::from_utf8(wtr.into_inner()?).context("Failed to convert CSV buffer to UTF-8")?;
            Ok(data)
        }

        ExportFormat::PlainText => {
            let mut buffer = String::new();
            for f in files {
                if f.is_symlink {
                    if let Some(target) = &f.target {
                        buffer.push_str(&format!("{} -> {}", f.name, target));
                    } else {
                        buffer.push_str(&f.name);
                    }
                } else {
                    buffer.push_str(&f.name);
                }
                buffer.push('\n');
            }
            Ok(buffer)
        }

        ExportFormat::Markdown => {
            let time_fmt = "%Y-%m-%d %H:%M:%S";

            struct Row {
                name: String,
                kind: &'static str,
                mode: String,
                size: String,
                modified: String,
                user: Option<String>,
                group: Option<String>,
                created: Option<String>,
            }

            let rows: Vec<Row> = files
                .iter()
                .map(|f| {
                    let name = if f.is_symlink {
                        if let Some(ref tgt) = f.target {
                            format!("{} -> {}", f.name, tgt)
                        } else {
                            f.name.clone()
                        }
                    } else {
                        f.name.clone()
                    };

                    let kind = if f.is_dir {
                        "dir"
                    } else if f.is_symlink {
                        "symlink"
                    } else if f.is_exec {
                        "exe"
                    } else {
                        "file"
                    };

                    let (user, group, created) = if is_full_mode {
                        let u = uzers::get_user_by_uid(f.uid)
                            .map(|u| format!("{} ({})", u.name().to_string_lossy(), f.uid))
                            .unwrap_or_else(|| f.uid.to_string());
                        let g = uzers::get_group_by_gid(f.gid)
                            .map(|g| format!("{} ({})", g.name().to_string_lossy(), f.gid))
                            .unwrap_or_else(|| f.gid.to_string());
                        let c = f
                            .created
                            .map(|t| t.format(time_fmt).to_string())
                            .unwrap_or_else(|| "-".to_string());
                        (Some(u), Some(g), Some(c))
                    } else {
                        (None, None, None)
                    };

                    Row {
                        name,
                        kind,
                        mode: format!("{:03o}", f.mode & 0o777),
                        size: file_info::format_size_human(f.size),
                        modified: f
                            .modified
                            .map(|m| m.format(time_fmt).to_string())
                            .unwrap_or_else(|| "-".to_string()),
                        user,
                        group,
                        created,
                    }
                })
                .collect();

            const H_NAME: &str = "Name";
            const H_TYPE: &str = "Type";
            const H_MODE: &str = "Mode";
            const H_SIZE: &str = "Size";
            const H_MOD: &str = "Modified";
            const H_USER: &str = "User";
            const H_GRP: &str = "Group";
            const H_CRT: &str = "Created";

            let w_name = rows.iter().map(|r| r.name.len()).max().unwrap_or(0).max(H_NAME.len());
            let w_type = rows.iter().map(|r| r.kind.len()).max().unwrap_or(0).max(H_TYPE.len());
            let w_mode = rows.iter().map(|r| r.mode.len()).max().unwrap_or(0).max(H_MODE.len());
            let w_size = rows.iter().map(|r| r.size.len()).max().unwrap_or(0).max(H_SIZE.len());
            let w_mod = rows
                .iter()
                .map(|r| r.modified.len())
                .max()
                .unwrap_or(0)
                .max(H_MOD.len());

            let (w_user, w_grp, w_crt) = if is_full_mode {
                let wu = rows
                    .iter()
                    .filter_map(|r| r.user.as_deref().map(|s| s.len()))
                    .max()
                    .unwrap_or(0)
                    .max(H_USER.len());
                let wg = rows
                    .iter()
                    .filter_map(|r| r.group.as_deref().map(|s| s.len()))
                    .max()
                    .unwrap_or(0)
                    .max(H_GRP.len());
                let wc = rows
                    .iter()
                    .filter_map(|r| r.created.as_deref().map(|s| s.len()))
                    .max()
                    .unwrap_or(0)
                    .max(H_CRT.len());
                (wu, wg, wc)
            } else {
                (0, 0, 0)
            };

            let mut buf = String::new();

            if is_full_mode {
                buf.push_str(&format!(
                    "| {:<w_name$} | {:<w_type$} | {:<w_mode$} | {:<w_size$} | {:<w_mod$} | {:<w_crt$} | {:<w_user$} | {:<w_grp$} |\n",
                    H_NAME, H_TYPE, H_MODE, H_SIZE, H_MOD, H_CRT, H_USER, H_GRP,
                    w_name = w_name, w_type = w_type, w_mode = w_mode,
                    w_size = w_size, w_mod = w_mod,
                    w_crt = w_crt, w_user = w_user, w_grp = w_grp,
                ));
                buf.push_str(&format!(
                    "| {:-<w_name$} | {:-<w_type$} | {:-<w_mode$} | {:-<w_size$} | {:-<w_mod$} | {:-<w_crt$} | {:-<w_user$} | {:-<w_grp$} |\n",
                    "", "", "", "", "", "", "", "",
                    w_name = w_name, w_type = w_type, w_mode = w_mode,
                    w_size = w_size, w_mod = w_mod,
                    w_crt = w_crt, w_user = w_user, w_grp = w_grp,
                ));
            } else {
                buf.push_str(&format!(
                    "| {:<w_name$} | {:<w_type$} | {:<w_mode$} | {:<w_size$} | {:<w_mod$} |\n",
                    H_NAME,
                    H_TYPE,
                    H_MODE,
                    H_SIZE,
                    H_MOD,
                    w_name = w_name,
                    w_type = w_type,
                    w_mode = w_mode,
                    w_size = w_size,
                    w_mod = w_mod,
                ));
                buf.push_str(&format!(
                    "| {:-<w_name$} | {:-<w_type$} | {:-<w_mode$} | {:-<w_size$} | {:-<w_mod$} |\n",
                    "",
                    "",
                    "",
                    "",
                    "",
                    w_name = w_name,
                    w_type = w_type,
                    w_mode = w_mode,
                    w_size = w_size,
                    w_mod = w_mod,
                ));
            }

            for r in &rows {
                if is_full_mode {
                    buf.push_str(&format!(
                        "| {:<w_name$} | {:<w_type$} | {:<w_mode$} | {:<w_size$} | {:<w_mod$} | {:<w_crt$} | {:<w_user$} | {:<w_grp$} |\n",
                        md_escape(&r.name),
                        md_escape(r.kind),
                        md_escape(&r.mode),
                        md_escape(&r.size),
                        md_escape(&r.modified),
                        md_escape(r.created.as_deref().unwrap_or("-")),
                        md_escape(r.user.as_deref().unwrap_or("-")),
                        md_escape(r.group.as_deref().unwrap_or("-")),
                        w_name = w_name, w_type = w_type, w_mode = w_mode,
                        w_size = w_size, w_mod = w_mod,
                        w_crt = w_crt, w_user = w_user, w_grp = w_grp,
                    ));
                } else {
                    buf.push_str(&format!(
                        "| {:<w_name$} | {:<w_type$} | {:<w_mode$} | {:<w_size$} | {:<w_mod$} |\n",
                        md_escape(&r.name),
                        md_escape(r.kind),
                        md_escape(&r.mode),
                        md_escape(&r.size),
                        md_escape(&r.modified),
                        w_name = w_name,
                        w_type = w_type,
                        w_mode = w_mode,
                        w_size = w_size,
                        w_mod = w_mod,
                    ));
                }
            }

            Ok(buf)
        }
    }
}
