use std::fs::{self, File};
use std::os::unix::fs::symlink;
use std::process::Command;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_ll")
}

// ---------------------------------------------------------------------------
// Basic smoke tests
// ---------------------------------------------------------------------------

#[test]
fn test_cli_runs_successfully() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();

    File::create(root.join("file1.txt")).expect("Failed to create file1");
    File::create(root.join("file2.log")).expect("Failed to create file2");
    fs::create_dir(root.join("subdir")).expect("Failed to create subdir");

    let link_path = root.join("link_to_file1");
    let target_path = root.join("file1.txt");
    symlink(&target_path, &link_path).expect("Failed to create symlink");

    let output = Command::new(bin())
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success(), "Binary execution failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("file1.txt"), "Output missing file1.txt");
    assert!(stdout.contains("subdir"), "Output missing subdir");
    assert!(stdout.contains("link_to_file1"), "Output missing symlink");
}

#[test]
fn test_cli_full_mode_flags() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("data.bin")).expect("Failed to create data.bin");

    let output = Command::new(bin())
        .arg("-f")
        .arg("-s")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("data.bin"));
    assert!(stdout.contains("User"));
    assert!(stdout.contains("Group"));
}

// ---------------------------------------------------------------------------
// Export: JSON
// ---------------------------------------------------------------------------

#[test]
fn test_export_json_output() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("test.json")).expect("Failed to create test.json");

    let output = Command::new(bin())
        .arg("-j")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.trim().starts_with('['));
    assert!(stdout.trim().ends_with(']'));
    assert!(stdout.contains("\"name\": \"test.json\""));
    assert!(stdout.contains("\"size\": 0"));

    // mode must be an octal string like "644", NOT a bare integer
    assert!(
        !stdout.contains("\"mode\": 1"),
        "mode should be an octal string, not a raw integer"
    );
    assert!(
        stdout.contains("\"mode\": \""),
        "mode must be serialized as a quoted octal string"
    );
}

#[test]
fn test_export_json_mode_is_octal_string() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("perm_test.txt")).expect("Failed to create file");

    let output = Command::new(bin())
        .arg("-j")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // The mode value must look like "644" or "755" — a quoted 3-digit octal string
    let re_match = stdout
        .lines()
        .any(|line| line.contains("\"mode\": \"") && line.contains("\""));
    assert!(re_match, "Expected mode to be a quoted octal string in JSON");
}

// ---------------------------------------------------------------------------
// Export: CSV  (semicolon delimiter)
// ---------------------------------------------------------------------------

#[test]
fn test_export_csv_output_semicolon_delimiter() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("data.csv")).expect("Failed to create data.csv");

    let output = Command::new(bin())
        .arg("-c")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Header must use semicolons
    assert!(
        stdout.contains("name;"),
        "CSV header must use ';' as delimiter, got:\n{}",
        stdout
    );
    // The file name must appear in the data rows
    assert!(stdout.contains("data.csv"), "CSV output missing data.csv");

    // Must NOT use comma as delimiter
    assert!(!stdout.starts_with("name,"), "CSV must NOT use ',' as delimiter");
}

#[test]
fn test_export_csv_mode_is_octal_string() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("file.txt")).expect("Failed to create file.txt");

    let output = Command::new(bin())
        .arg("-c")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // In CSV the mode value should be a short octal string (e.g. "644"),
    // not a large decimal integer like 33188.
    let data_line = stdout
        .lines()
        .skip(1) // skip header
        .next()
        .unwrap_or("");

    // The mode field (index 5 in the struct, 0-based) should look like 3 digits
    // We just verify it's not a 5-digit decimal integer.
    assert!(
        !data_line.contains(";33188;"),
        "mode must not be a raw decimal integer in CSV"
    );
}

// ---------------------------------------------------------------------------
// Export: Plain text
// ---------------------------------------------------------------------------

#[test]
fn test_plain_text_export() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("simple.txt")).expect("Failed to create simple.txt");

    let output = Command::new(bin())
        .arg("-p")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("simple.txt"));
    // Must NOT contain table borders
    assert!(!stdout.contains("┃"));
    assert!(!stdout.contains("┏"));
}

#[test]
fn test_plain_text_symlink_shows_arrow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();

    let target = root.join("target.txt");
    File::create(&target).expect("Failed to create target.txt");
    symlink(&target, root.join("link_to_target")).expect("Failed to create symlink");

    let output = Command::new(bin())
        .arg("-p")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Plain text export for symlinks should show "name -> target"
    assert!(
        stdout.contains("link_to_target ->"),
        "Plain text export should show symlink arrow"
    );
}

// ---------------------------------------------------------------------------
// Export: to named file  (-o)
// ---------------------------------------------------------------------------

#[test]
fn test_export_to_named_file_json() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("file.txt")).expect("Failed to create file.txt");

    let output_file = root.join("output.json");

    let output = Command::new(bin())
        .arg("-j")
        .arg("-o")
        .arg(output_file.to_str().unwrap())
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        output_file.exists(),
        "Output file was not created at {}",
        output_file.display()
    );

    let content = fs::read_to_string(&output_file).expect("Failed to read output file");
    assert!(content.contains("\"name\": \"file.txt\""));

    // The last line of stdout is the path to the created file.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let last_line = stdout.lines().last().unwrap_or("").trim();
    assert!(
        last_line.contains("output.json"),
        "last stdout line should be the output path, got: {last_line}"
    );
}

#[test]
fn test_export_to_named_file_csv() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("data.txt")).expect("Failed to create data.txt");

    let output_file = root.join("output.csv");

    let output = Command::new(bin())
        .arg("-c")
        .arg("-o")
        .arg(output_file.to_str().unwrap())
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        output_file.exists(),
        "Output file was not created: {}",
        output_file.display()
    );

    let content = fs::read_to_string(&output_file).expect("Failed to read output file");
    assert!(content.contains("name;"), "CSV file must use ';' delimiter");
    assert!(content.contains("data.txt"));
}

#[test]
fn test_export_to_named_file_xml() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("item.txt")).expect("Failed to create item.txt");

    let output_file = root.join("result.xml");

    let output = Command::new(bin())
        .arg("-x")
        .arg("-o")
        .arg(output_file.to_str().unwrap())
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        output_file.exists(),
        "XML output file was not created: {}",
        output_file.display()
    );

    let content = fs::read_to_string(&output_file).expect("Failed to read XML output file");
    assert!(content.contains("<Files>") || content.contains("<File>"));
    assert!(content.contains("item.txt"));
}

// ---------------------------------------------------------------------------
// Export: auto-output (-O) — file must be created inside the target directory
// ---------------------------------------------------------------------------

#[test]
fn test_auto_output_creates_file_in_target_dir_json() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("sample.txt")).expect("Failed to create sample.txt");

    let output = Command::new(bin())
        .arg("-j")
        .arg("-O")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The table is printed first; the last non-empty stdout line is the created file path.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let created_path_str = stdout.lines().last().unwrap_or("").trim();
    assert!(
        !created_path_str.is_empty(),
        "stdout should contain the path of the created file"
    );

    let created_path = std::path::Path::new(created_path_str);
    assert!(
        created_path.exists(),
        "Auto-output file does not exist at '{}'",
        created_path_str
    );
    assert!(
        created_path_str.ends_with(".json"),
        "Auto-output file should have .json extension, got '{}'",
        created_path_str
    );

    let content = fs::read_to_string(created_path).expect("Failed to read auto-output file");
    assert!(content.contains("\"name\": \"sample.txt\""));
}

#[test]
fn test_auto_output_creates_file_in_target_dir_csv() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("data.txt")).expect("Failed to create data.txt");

    let output = Command::new(bin())
        .arg("-c")
        .arg("-O")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The table is printed first; the last non-empty stdout line is the created file path.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let created_path_str = stdout.lines().last().unwrap_or("").trim();

    let created_path = std::path::Path::new(created_path_str);
    assert!(
        created_path.exists(),
        "Auto-output CSV file does not exist at '{}'",
        created_path_str
    );
    assert!(
        created_path_str.ends_with(".csv"),
        "Auto-output file should have .csv extension"
    );

    let content = fs::read_to_string(created_path).expect("Failed to read auto-output CSV file");
    assert!(content.contains("name;"), "CSV auto-output must use ';' delimiter");
}

#[test]
fn test_auto_output_no_collision() {
    // If the auto-generated filename already exists, a counter suffix is added.
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("entry.txt")).expect("Failed to create entry.txt");

    // First run — creates <dirname>.json
    let output1 = Command::new(bin())
        .arg("-j")
        .arg("-O")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");
    assert!(output1.status.success());

    // The table is printed first; the last non-empty stdout line is the created file path.
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    let path1 = stdout1.lines().last().unwrap_or("").trim().to_owned();

    // Second run — must create a different file (with counter)
    let output2 = Command::new(bin())
        .arg("-j")
        .arg("-O")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");
    assert!(output2.status.success());

    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let path2 = stdout2.lines().last().unwrap_or("").trim().to_owned();

    assert_ne!(
        path1, path2,
        "Second -O run should produce a different filename to avoid collision"
    );
    assert!(
        std::path::Path::new(&path2).exists(),
        "Second auto-output file must exist"
    );
}

// ---------------------------------------------------------------------------
// No -N flag: ensure it is rejected by the CLI
// ---------------------------------------------------------------------------

#[test]
fn test_names_only_flag_removed() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("file.txt")).expect("Failed to create file.txt");

    // -N should no longer be a recognised flag
    let output = Command::new(bin())
        .arg("-N")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(
        !output.status.success(),
        "The -N flag should have been removed and must cause a CLI error"
    );
}

// ---------------------------------------------------------------------------
// XML export
// ---------------------------------------------------------------------------

#[test]
fn test_export_xml_output() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("doc.xml")).expect("Failed to create doc.xml");

    let output = Command::new(bin())
        .arg("-x")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("<Files>") || stdout.contains("<File>"));
    assert!(stdout.contains("doc.xml"));
}

// ---------------------------------------------------------------------------
// Guard: -o / -O without a format flag must print an error and exit non-zero
// ---------------------------------------------------------------------------

#[test]
fn test_output_flag_without_format_gives_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("file.txt")).expect("Failed to create file.txt");

    let output_file = root.join("out.json");

    // -o without any export format flag
    let output = Command::new(bin())
        .arg("-o")
        .arg(output_file.to_str().unwrap())
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(
        !output.status.success(),
        "Expected non-zero exit when -o is used without a format flag"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("export format"),
        "stderr should mention 'export format', got:\n{}",
        stderr
    );

    // The output file must NOT have been created
    assert!(
        !output_file.exists(),
        "No file should be created when format flag is missing"
    );
}

#[test]
fn test_auto_output_flag_without_format_gives_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("file.txt")).expect("Failed to create file.txt");

    // -O without any export format flag
    let output = Command::new(bin())
        .arg("-O")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(
        !output.status.success(),
        "Expected non-zero exit when -O is used without a format flag"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("export format"),
        "stderr should mention 'export format', got:\n{}",
        stderr
    );
}

#[test]
fn test_output_flag_error_lists_format_options() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();

    let output = Command::new(bin())
        .arg("-O")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should list the available format flags
    assert!(
        stderr.contains("-j") || stderr.contains("json"),
        "stderr should mention JSON flag"
    );
    assert!(
        stderr.contains("-c") || stderr.contains("csv"),
        "stderr should mention CSV flag"
    );
    assert!(
        stderr.contains("-p") || stderr.contains("plain"),
        "stderr should mention plain flag"
    );
    assert!(
        stderr.contains("-M") || stderr.contains("markdown"),
        "stderr should mention Markdown flag"
    );
}

// ---------------------------------------------------------------------------
// Markdown export (-M)
// ---------------------------------------------------------------------------

#[test]
fn test_export_markdown_stdout() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("readme.md")).expect("Failed to create readme.md");
    File::create(root.join("main.rs")).expect("Failed to create main.rs");

    let output = Command::new(bin())
        .arg("-M")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Must contain GFM table header
    assert!(stdout.contains("| Name"), "Missing table Name header");
    assert!(stdout.contains("| Type"), "Missing table Type header");
    assert!(stdout.contains("| Mode"), "Missing table Mode header");
    assert!(stdout.contains("| Size"), "Missing table Size header");
    assert!(stdout.contains("| Modified"), "Missing table Modified header");

    // Must contain separator row
    assert!(
        stdout.contains("|---") || stdout.contains("| ---"),
        "Missing separator row"
    );

    // Must contain file names
    assert!(stdout.contains("readme.md"), "Missing readme.md in output");
    assert!(stdout.contains("main.rs"), "Missing main.rs in output");
}

#[test]
fn test_export_markdown_mode_is_octal_string() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("sample.txt")).expect("Failed to create sample.txt");

    let output = Command::new(bin())
        .arg("-M")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Mode column should contain short octal like "644", not a 5-digit decimal
    assert!(
        !stdout.contains("33188"),
        "mode must not appear as a raw decimal integer in Markdown output"
    );
}

#[test]
fn test_export_markdown_to_named_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("entry.txt")).expect("Failed to create entry.txt");

    let output_file = root.join("report.md");

    let output = Command::new(bin())
        .arg("-M")
        .arg("-o")
        .arg(output_file.to_str().unwrap())
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        output_file.exists(),
        "Markdown output file was not created: {}",
        output_file.display()
    );

    let content = fs::read_to_string(&output_file).expect("Failed to read Markdown file");
    assert!(content.contains("| Name"), "Missing header in Markdown file");
    assert!(content.contains("entry.txt"), "Missing entry.txt in Markdown file");

    // stdout should print the created file path
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("report.md"), "stdout should print the output file path");
}

#[test]
fn test_export_markdown_auto_output() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("data.txt")).expect("Failed to create data.txt");

    let output = Command::new(bin())
        .arg("-M")
        .arg("-O")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The table is printed first; the last non-empty stdout line is the created file path.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let created_path_str = stdout.lines().last().unwrap_or("").trim();

    assert!(
        !created_path_str.is_empty(),
        "stdout should contain the path of the auto-generated file"
    );
    assert!(
        created_path_str.ends_with(".md"),
        "Auto-output Markdown file should have .md extension, got '{}'",
        created_path_str
    );

    let created_path = std::path::Path::new(created_path_str);
    assert!(
        created_path.exists(),
        "Auto-output Markdown file does not exist at '{}'",
        created_path_str
    );

    let content = fs::read_to_string(created_path).expect("Failed to read Markdown auto-output");
    assert!(
        content.contains("| Name"),
        "Missing header in auto-generated Markdown file"
    );
    assert!(content.contains("data.txt"), "Missing data.txt in Markdown auto-output");
}

#[test]
fn test_export_markdown_symlink_shows_arrow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();

    let target = root.join("original.txt");
    File::create(&target).expect("Failed to create original.txt");
    symlink(&target, root.join("link_to_original")).expect("Failed to create symlink");

    let output = Command::new(bin())
        .arg("-M")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("link_to_original ->"),
        "Markdown export should show symlink arrow, got:\n{}",
        stdout
    );
}

#[test]
fn test_export_markdown_directory_type() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    fs::create_dir(root.join("mydir")).expect("Failed to create mydir");

    let output = Command::new(bin())
        .arg("-M")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("mydir"), "Missing mydir in Markdown output");
    assert!(stdout.contains("dir"), "Directory type should be 'dir' in Markdown");
}

// ---------------------------------------------------------------------------
// Markdown full mode (-fM) — extra columns User, Group, Created
// ---------------------------------------------------------------------------

#[test]
fn test_export_markdown_full_mode_columns() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("item.txt")).expect("Failed to create item.txt");

    let output = Command::new(bin())
        .arg("-f")
        .arg("-M")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("| User"), "Full Markdown should have User column");
    assert!(stdout.contains("| Group"), "Full Markdown should have Group column");
    assert!(stdout.contains("| Created"), "Full Markdown should have Created column");
    assert!(
        stdout.contains("item.txt"),
        "item.txt must appear in full Markdown output"
    );
}

#[test]
fn test_export_markdown_compact_mode_no_extra_columns() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("item.txt")).expect("Failed to create item.txt");

    let output = Command::new(bin())
        .arg("-M")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!stdout.contains("| User"), "Compact Markdown must NOT have User column");
    assert!(
        !stdout.contains("| Group"),
        "Compact Markdown must NOT have Group column"
    );
    assert!(
        !stdout.contains("| Created"),
        "Compact Markdown must NOT have Created column"
    );
}

// ---------------------------------------------------------------------------
// -F: files only
// ---------------------------------------------------------------------------

#[test]
fn test_files_only_flag() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("file.txt")).expect("Failed to create file.txt");
    fs::create_dir(root.join("subdir")).expect("Failed to create subdir");

    let output = Command::new(bin())
        .arg("-F")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("file.txt"), "file.txt must appear with -F");
    assert!(!stdout.contains("subdir"), "subdir must NOT appear with -F");
}

#[test]
fn test_files_only_excludes_symlinks() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();

    let target = root.join("target.txt");
    File::create(&target).expect("Failed to create target.txt");
    symlink(&target, root.join("link")).expect("Failed to create symlink");

    let output = Command::new(bin())
        .arg("-F")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("target.txt"), "target.txt must appear with -F");
    assert!(!stdout.contains("link"), "symlink must NOT appear with -F");
}

#[test]
fn test_files_only_json_export() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("only_file.txt")).expect("Failed to create file");
    fs::create_dir(root.join("adir")).expect("Failed to create adir");

    let output = Command::new(bin())
        .arg("-F")
        .arg("-j")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("only_file.txt"),
        "only_file.txt must appear in JSON with -F"
    );
    assert!(!stdout.contains("\"adir\""), "adir must NOT appear in JSON with -F");
}

// ---------------------------------------------------------------------------
// -D: directories only
// ---------------------------------------------------------------------------

#[test]
fn test_dirs_only_flag() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("file.txt")).expect("Failed to create file.txt");
    fs::create_dir(root.join("mydir")).expect("Failed to create mydir");

    let output = Command::new(bin())
        .arg("-D")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("mydir"), "mydir must appear with -D");
    assert!(!stdout.contains("file.txt"), "file.txt must NOT appear with -D");
}

#[test]
fn test_dirs_only_json_export() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("file.txt")).expect("Failed to create file.txt");
    fs::create_dir(root.join("onlydir")).expect("Failed to create onlydir");

    let output = Command::new(bin())
        .arg("-D")
        .arg("-j")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("\"onlydir\""), "onlydir must appear in JSON with -D");
    assert!(
        !stdout.contains("\"file.txt\""),
        "file.txt must NOT appear in JSON with -D"
    );
}

// ---------------------------------------------------------------------------
// -h: show hidden entries
// ---------------------------------------------------------------------------

#[test]
fn test_hidden_excluded_by_default() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("visible.txt")).expect("Failed to create visible.txt");
    File::create(root.join(".hidden")).expect("Failed to create .hidden");

    let output = Command::new(bin())
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("visible.txt"), "visible.txt must appear by default");
    assert!(!stdout.contains(".hidden"), "hidden entry must NOT appear without -h");
}

#[test]
fn test_hidden_excluded_even_with_full_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("normal.txt")).expect("Failed to create normal.txt");
    File::create(root.join(".secret")).expect("Failed to create .secret");

    let output = Command::new(bin())
        .arg("-f")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("normal.txt"), "normal.txt must appear with -f");
    assert!(!stdout.contains(".secret"), "-f alone must NOT reveal hidden entries");
}

#[test]
fn test_hidden_shown_with_h_flag() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("visible.txt")).expect("Failed to create visible.txt");
    File::create(root.join(".hidden")).expect("Failed to create .hidden");

    let output = Command::new(bin())
        .arg("-H")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("visible.txt"), "visible.txt must appear with -h");
    assert!(stdout.contains(".hidden"), ".hidden must appear with -h");
}

#[test]
fn test_hidden_shown_with_h_and_full_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join(".dotfile")).expect("Failed to create .dotfile");

    let output = Command::new(bin())
        .arg("-f")
        .arg("-H")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains(".dotfile"), ".dotfile must appear with -fh");
}

#[test]
fn test_hidden_shown_in_export() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join(".env")).expect("Failed to create .env");
    File::create(root.join("app.rs")).expect("Failed to create app.rs");

    let output = Command::new(bin())
        .arg("-H")
        .arg("-j")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("\".env\""), ".env must appear in JSON with -h");
    assert!(stdout.contains("\"app.rs\""), "app.rs must appear in JSON with -h");
}

#[test]
fn test_hidden_excluded_in_export_by_default() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join(".env")).expect("Failed to create .env");
    File::create(root.join("app.rs")).expect("Failed to create app.rs");

    let output = Command::new(bin())
        .arg("-j")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!stdout.contains("\".env\""), ".env must NOT appear in JSON without -h");
    assert!(stdout.contains("\"app.rs\""), "app.rs must appear in JSON without -h");
}

// ---------------------------------------------------------------------------
// accessed field must not appear in any export
// ---------------------------------------------------------------------------

#[test]
fn test_json_no_accessed_field() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("check.txt")).expect("Failed to create check.txt");

    let output = Command::new(bin())
        .arg("-j")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("\"accessed\""),
        "accessed field must not appear in JSON export"
    );
}

#[test]
fn test_csv_no_accessed_field() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    File::create(root.join("check.txt")).expect("Failed to create check.txt");

    let output = Command::new(bin())
        .arg("-c")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("accessed"),
        "accessed field must not appear in CSV export"
    );
}
