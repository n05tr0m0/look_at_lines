use crate::file_info::{self, FileInfo};
use crate::theme::Palette;
use owo_colors::OwoColorize;
use terminal_size::{terminal_size, Width};
use unicode_width::UnicodeWidthStr;

fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Truncates `s` to at most `max_cols` terminal columns, appending `".."`.
/// A character is only included if it fits completely within the budget.
fn truncate_to_cols(s: &str, max_cols: usize) -> String {
    if display_width(s) <= max_cols {
        return s.to_owned();
    }

    if max_cols < 3 {
        return "..".chars().take(max_cols).collect();
    }

    let budget = max_cols - 2;
    let mut used = 0usize;
    let mut result = String::new();

    for ch in s.chars() {
        let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
        if used + ch_width > budget {
            break;
        }
        result.push(ch);
        used += ch_width;
    }

    result.push_str("..");
    result
}

/// Renders the file list as a Unicode box-drawing table, sized to the terminal.
/// Names that exceed the available column width are truncated with `".."`.
pub fn render_table(files: &[FileInfo], is_full_mode: bool, pal: &Palette) {
    if files.is_empty() {
        println!("{}", "Directory is empty.".color(pal.border));
        return;
    }

    let term_width = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    };

    let mut perm_w = if is_full_mode { 9 } else { 3 };
    let mut size_w = 4;
    let type_w = 4;
    let mut mod_w = 8;
    let mut user_w = 4;
    let mut group_w = 5;
    let mut created_w = 7;

    let time_fmt = "%Y-%m-%d %H:%M:%S";

    let mut users_cache = std::collections::HashMap::new();
    let mut groups_cache = std::collections::HashMap::new();

    for f in files {
        let size_str = if is_full_mode {
            f.size.to_string()
        } else {
            file_info::format_size_human(f.size)
        };
        size_w = size_w.max(size_str.len());

        if let Some(m) = f.modified {
            mod_w = mod_w.max(m.format(time_fmt).to_string().len());
        }

        if is_full_mode {
            let u_name = users_cache.entry(f.uid).or_insert_with(|| {
                uzers::get_user_by_uid(f.uid)
                    .map(|u| format!("{} ({})", u.name().to_string_lossy(), f.uid))
                    .unwrap_or_else(|| f.uid.to_string())
            });
            user_w = user_w.max(u_name.len());

            let g_name = groups_cache.entry(f.gid).or_insert_with(|| {
                uzers::get_group_by_gid(f.gid)
                    .map(|g| format!("{} ({})", g.name().to_string_lossy(), f.gid))
                    .unwrap_or_else(|| f.gid.to_string())
            });
            group_w = group_w.max(g_name.len());

            if let Some(c) = f.created {
                created_w = created_w.max(c.format(time_fmt).to_string().len());
            }
        }
    }

    perm_w = perm_w.max(4);

    let mut fixed_widths = vec![perm_w, size_w, type_w, mod_w];
    if is_full_mode {
        fixed_widths.extend_from_slice(&[created_w, user_w, group_w]);
    }

    let cols_count = fixed_widths.len() + 1;
    let total_fixed_space: usize = fixed_widths.iter().sum::<usize>() + (1 + 3 * cols_count);
    let name_w = term_width.saturating_sub(total_fixed_space).max(10);

    let print_border = |start: &str, mid: &str, end: &str| {
        print!("{}", start.color(pal.border));
        print!("{}", "━".repeat(name_w + 2).color(pal.border));

        for w in &fixed_widths {
            print!("{}", mid.color(pal.border));
            print!("{}", "━".repeat(*w + 2).color(pal.border));
        }
        println!("{}", end.color(pal.border));
    };

    let v_bar = format!("{}", "┃".color(pal.border));

    print_border("┏", "┳", "┓");

    print!("{} {:<width$} ", v_bar, "Name", width = name_w);
    print!("{} {:>width$} ", v_bar, "Perm", width = perm_w);
    print!("{} {:>width$} ", v_bar, "Size", width = size_w);
    print!("{} {:>width$} ", v_bar, "Type", width = type_w);
    print!("{} {:>width$} ", v_bar, "Modified", width = mod_w);
    if is_full_mode {
        print!("{} {:>width$} ", v_bar, "Created", width = created_w);
        print!("{} {:>width$} ", v_bar, "User", width = user_w);
        print!("{} {:>width$} ", v_bar, "Group", width = group_w);
    }
    println!("{}", v_bar);

    print_border("┣", "╋", "┫");

    for f in files {
        let raw_display: String = if is_full_mode && f.is_symlink {
            if let Some(ref tgt) = f.target {
                format!("{} --> {}", f.name, tgt)
            } else {
                f.name.clone()
            }
        } else {
            f.name.clone()
        };

        let raw_width = display_width(&raw_display);

        let (styled_display, pad_len) = if raw_width > name_w {
            let truncated = truncate_to_cols(&raw_display, name_w);
            let styled = if f.is_dir {
                truncated.color(pal.dir).to_string()
            } else if f.is_exec {
                truncated.color(pal.exec).bold().to_string()
            } else {
                truncated.color(pal.file).to_string()
            };
            (styled, 0usize)
        } else {
            (f.display_name(is_full_mode, pal), name_w - raw_width)
        };

        let perm_str = if is_full_mode {
            file_info::format_permissions_rwx(f.mode)
        } else {
            format!("{:03o}", f.mode & 0o777)
        };
        let size_str = if is_full_mode {
            f.size.to_string()
        } else {
            file_info::format_size_human(f.size)
        };
        let type_str = if f.is_dir {
            "d".to_string()
        } else if f.is_symlink {
            "l".to_string()
        } else if f.is_exec {
            "*f".to_string()
        } else {
            "f".to_string()
        };

        let m_str = f
            .modified
            .map(|m| m.format(time_fmt).to_string())
            .unwrap_or_else(|| "-".to_string());

        print!("{} {}{} ", v_bar, styled_display, " ".repeat(pad_len));
        print!("{} {:>width$} ", v_bar, perm_str, width = perm_w);
        print!("{} {:>width$} ", v_bar, size_str, width = size_w);
        print!("{} {:>width$} ", v_bar, type_str, width = type_w);
        print!("{} {:>width$} ", v_bar, m_str, width = mod_w);

        if is_full_mode {
            let u_name = users_cache.get(&f.uid).expect("User ID missing from cache");
            let g_name = groups_cache.get(&f.gid).expect("Group ID missing from cache");
            let c_str = f
                .created
                .map(|c| c.format(time_fmt).to_string())
                .unwrap_or_else(|| "-".to_string());

            print!("{} {:>width$} ", v_bar, c_str, width = created_w);
            print!("{} {:>width$} ", v_bar, u_name, width = user_w);
            print!("{} {:>width$} ", v_bar, g_name, width = group_w);
        }
        println!("{}", v_bar);
    }

    print_border("┗", "┻", "┛");
}

#[cfg(test)]
mod tests {
    use super::{display_width, truncate_to_cols};

    #[test]
    fn test_display_width_ascii() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width(""), 0);
        assert_eq!(display_width("a"), 1);
    }

    #[test]
    fn test_display_width_cyrillic() {
        assert_eq!(display_width("привет"), 6);
        assert_eq!(display_width("файл"), 4);
    }

    #[test]
    fn test_display_width_cjk() {
        assert_eq!(display_width("文件"), 4);
        assert_eq!(display_width("ファイル"), 8);
    }

    #[test]
    fn test_display_width_arabic() {
        assert_eq!(display_width("ملف"), 3);
    }

    #[test]
    fn test_display_width_mixed() {
        // ASCII (5) + space (1) + Cyrillic (6) = 12
        assert_eq!(display_width("hello привет"), 12);
    }

    #[test]
    fn test_truncate_fits_without_truncation() {
        let s = "hello";
        let result = truncate_to_cols(s, 10);
        assert_eq!(result, "hello");
        assert!(display_width(&result) <= 10);
    }

    #[test]
    fn test_truncate_exact_fit() {
        let s = "hello";
        let result = truncate_to_cols(s, 5);
        assert_eq!(result, "hello");
        assert!(display_width(&result) <= 5);
    }

    #[test]
    fn test_truncate_ascii_long_name() {
        // "hello_world_long" is 16 chars, truncate to 10 -> 8 chars + ".."
        let s = "hello_world_long";
        let result = truncate_to_cols(s, 10);
        assert!(display_width(&result) <= 10, "width={}", display_width(&result));
        assert!(result.ends_with(".."), "should end with '..'");
    }

    #[test]
    fn test_truncate_cyrillic() {
        // Cyrillic: each char = 1 col. "привет_мир" = 10 cols, truncate to 8 -> 6 + ".."
        let s = "привет_мир";
        let result = truncate_to_cols(s, 8);
        assert!(display_width(&result) <= 8, "width={}", display_width(&result));
        assert!(result.ends_with(".."));
    }

    #[test]
    fn test_truncate_cyrillic_no_overflow_edge() {
        // 20-column budget, name exactly 20 wide — no truncation
        let s = "абвгдеёжзийклмнопрст"; // 20 Cyrillic chars = 20 cols
        let result = truncate_to_cols(s, 20);
        assert_eq!(result, s);
    }

    #[test]
    fn test_truncate_cjk() {
        // CJK: each char = 2 cols. "文件系统目录" = 12 cols, truncate to 8 -> 3 chars (6) + ".." = 8
        let s = "文件系统目录";
        let result = truncate_to_cols(s, 8);
        assert!(display_width(&result) <= 8, "width={}", display_width(&result));
        assert!(result.ends_with(".."));
    }

    #[test]
    fn test_truncate_cjk_no_overflow() {
        // 12 cols budget, name = 12 cols — no truncation
        let s = "文件系统目录"; // 6 * 2 = 12
        let result = truncate_to_cols(s, 12);
        assert_eq!(result, s);
    }

    #[test]
    fn test_truncate_arabic() {
        // Arabic: each char = 1 col (LTR rendering width).
        // "ملف_اسم_طويل_جدا" is 17 chars. truncate to 10 -> 8 chars + ".."
        let s = "ملف_اسم_طويل_جدا";
        let result = truncate_to_cols(s, 10);
        assert!(display_width(&result) <= 10, "width={}", display_width(&result));
        assert!(result.ends_with(".."));
    }

    #[test]
    fn test_truncate_hindi_devanagari() {
        // Devanagari: each base char = 1 col (unicode-width treats as 1).
        // Truncate a longer name to 10 cols.
        let s = "फ़ाइलनामजोबहुतलंबाहै";
        let result = truncate_to_cols(s, 10);
        assert!(display_width(&result) <= 10, "width={}", display_width(&result));
    }

    #[test]
    fn test_truncate_japanese() {
        // Hiragana: each char = 2 cols (full-width).
        // "あいうえおかきくけこ" = 10 chars * 2 = 20 cols. truncate to 12 -> 5 chars (10) + ".." = 12
        let s = "あいうえおかきくけこ";
        let result = truncate_to_cols(s, 12);
        assert!(display_width(&result) <= 12, "width={}", display_width(&result));
        assert!(result.ends_with(".."));
    }

    #[test]
    fn test_truncate_korean() {
        // Hangul syllable blocks: each = 2 cols.
        // "파일이름이길다" = 7 * 2 = 14 cols. truncate to 10 -> 4 chars (8) + ".." = 10
        let s = "파일이름이길다";
        let result = truncate_to_cols(s, 10);
        assert!(display_width(&result) <= 10, "width={}", display_width(&result));
        assert!(result.ends_with(".."));
    }

    #[test]
    fn test_truncate_mixed_latin_cyrillic() {
        // "file_файл_name" = 5 + 1 + 4 + 1 + 4 = 15 cols, truncate to 10
        let s = "file_файл_name";
        let result = truncate_to_cols(s, 10);
        assert!(display_width(&result) <= 10, "width={}", display_width(&result));
        assert!(result.ends_with(".."));
    }

    #[test]
    fn test_truncate_very_small_budget() {
        // Budget < 3: can't even fit "..", just return shortest possible
        let s = "hello";
        let result = truncate_to_cols(s, 2);
        assert!(display_width(&result) <= 2, "width={}", display_width(&result));
    }

    #[test]
    fn test_truncate_budget_exactly_2() {
        let s = "abcde";
        let result = truncate_to_cols(s, 2);
        assert!(display_width(&result) <= 2);
    }

    #[test]
    fn test_truncate_spanish() {
        // Spanish with accented chars (1 col each in unicode-width)
        let s = "archivo_muy_largo_en_español";
        let result = truncate_to_cols(s, 15);
        assert!(display_width(&result) <= 15, "width={}", display_width(&result));
        assert!(result.ends_with(".."));
    }

    #[test]
    fn test_truncate_french() {
        // French: "fichier_très_long_nom_complet" — accentuated chars = 1 col
        let s = "fichier_très_long_nom_complet";
        let result = truncate_to_cols(s, 20);
        assert!(display_width(&result) <= 20, "width={}", display_width(&result));
        assert!(result.ends_with(".."));
    }

    #[test]
    fn test_truncate_german() {
        // German: "Dateiname_mit_Umlauten_ÄÖÜß" — umlauts = 1 col each
        let s = "Dateiname_mit_Umlauten_ÄÖÜß";
        let result = truncate_to_cols(s, 18);
        assert!(display_width(&result) <= 18, "width={}", display_width(&result));
        assert!(result.ends_with(".."));
    }

    #[test]
    fn test_truncate_portuguese() {
        let s = "arquivo_com_nome_muito_longo_português";
        let result = truncate_to_cols(s, 20);
        assert!(display_width(&result) <= 20, "width={}", display_width(&result));
        assert!(result.ends_with(".."));
    }

    #[test]
    fn test_truncate_chinese_simplified() {
        // Simplified Chinese: each char = 2 cols
        // "非常长的文件名称示例文字" = 12 * 2 = 24 cols, truncate to 14
        let s = "非常长的文件名称示例文字";
        let result = truncate_to_cols(s, 14);
        assert!(display_width(&result) <= 14, "width={}", display_width(&result));
        assert!(result.ends_with(".."));
    }

    #[test]
    fn test_truncate_indonesian() {
        // Indonesian uses Latin script, 1 col per char
        let s = "nama_berkas_yang_sangat_panjang_sekali";
        let result = truncate_to_cols(s, 20);
        assert!(display_width(&result) <= 20, "width={}", display_width(&result));
        assert!(result.ends_with(".."));
    }

    /// Calculates the name column width the same way render_table does.
    fn compute_name_w(term_width: usize, is_full_mode: bool) -> usize {
        let perm_w = if is_full_mode { 9 } else { 4 };
        let size_w = 4usize;
        let type_w = 4usize;
        let mod_w = 8usize;
        let mut fixed_widths = vec![perm_w, size_w, type_w, mod_w];
        if is_full_mode {
            fixed_widths.extend_from_slice(&[7usize, 4, 5]);
        }
        let cols_count = fixed_widths.len() + 1;
        let total_fixed_space: usize = fixed_widths.iter().sum::<usize>() + (1 + 3 * cols_count);
        term_width.saturating_sub(total_fixed_space).max(10)
    }

    /// For a given terminal width and file name, verify that the rendered name
    /// cell (after potential truncation) never exceeds `name_w` display columns.
    fn assert_name_fits(term_width: usize, file_name: &str) {
        let name_w = compute_name_w(term_width, false);
        let raw_w = display_width(file_name);
        let rendered = if raw_w > name_w {
            truncate_to_cols(file_name, name_w)
        } else {
            file_name.to_owned()
        };
        assert!(
            display_width(&rendered) <= name_w,
            "name '{}' rendered as '{}' has width {} > name_w {} (term_width={})",
            file_name,
            rendered,
            display_width(&rendered),
            name_w,
            term_width
        );
    }

    #[test]
    fn test_layout_cyrillic_80_cols() {
        assert_name_fits(80, "очень_длинное_имя_файла_на_русском_языке");
    }

    #[test]
    fn test_layout_cyrillic_40_cols() {
        assert_name_fits(40, "очень_длинное_имя_файла_на_русском_языке");
    }

    #[test]
    fn test_layout_cjk_80_cols() {
        assert_name_fits(80, "非常长的文件名称示例文字超长测试用例");
    }

    #[test]
    fn test_layout_cjk_40_cols() {
        assert_name_fits(40, "非常长的文件名称示例文字超长测试用例");
    }

    #[test]
    fn test_layout_arabic_80_cols() {
        assert_name_fits(80, "اسم_الملف_الطويل_جداً_للاختبار");
    }

    #[test]
    fn test_layout_japanese_80_cols() {
        assert_name_fits(80, "とても長いファイル名のテスト用サンプル");
    }

    #[test]
    fn test_layout_korean_80_cols() {
        assert_name_fits(80, "매우긴파일이름테스트용샘플데이터");
    }

    #[test]
    fn test_layout_hindi_80_cols() {
        assert_name_fits(80, "बहुत_लंबा_फ़ाइल_नाम_परीक्षण");
    }

    #[test]
    fn test_layout_spanish_80_cols() {
        assert_name_fits(80, "archivo_con_nombre_extremadamente_largo_en_español");
    }

    #[test]
    fn test_layout_german_80_cols() {
        assert_name_fits(80, "Dateiname_mit_vielen_Umlauten_und_sehr_lang_ÄÖÜäöüß");
    }

    #[test]
    fn test_layout_french_80_cols() {
        assert_name_fits(80, "fichier_avec_nom_très_très_très_très_long_en_français");
    }

    #[test]
    fn test_layout_portuguese_80_cols() {
        assert_name_fits(80, "arquivo_português_com_nome_muito_muito_longo");
    }

    #[test]
    fn test_layout_empty_name() {
        // Empty name should never panic
        assert_name_fits(80, "");
    }

    #[test]
    fn test_layout_ascii_exactly_at_boundary() {
        let name_w = compute_name_w(80, false);
        // Name exactly fits
        let name: String = "a".repeat(name_w);
        assert_name_fits(80, &name);
    }

    #[test]
    fn test_layout_ascii_one_over_boundary() {
        let name_w = compute_name_w(80, false);
        // Name is one char too wide — must be truncated
        let name: String = "a".repeat(name_w + 1);
        assert_name_fits(80, &name);
    }

    #[test]
    fn test_layout_cjk_odd_budget() {
        // CJK chars are 2-wide; if budget is odd the last char might not fit
        assert_name_fits(81, "文件系统目录测试用例数据文件系统");
    }

    #[test]
    fn test_layout_very_narrow_terminal() {
        // Even with a very narrow terminal, must not panic and must not exceed budget
        assert_name_fits(30, "очень_длинное_имя_файла_на_русском_языке");
        assert_name_fits(30, "非常长的文件名称");
    }
}
