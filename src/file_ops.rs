//! File operations

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::sync::Arc;

use glob::Pattern;
use regex::Regex;

use crate::error::Error;

/// Filter files based on glob patterns for files and directories to ignore
pub(crate) fn filter_files(
    glob_patterns: &[Pattern],
    files: impl IntoIterator<Item = String>,
) -> impl IntoIterator<Item = String> {
    files
        .into_iter()
        .filter(|filepath| !glob_patterns.iter().any(|p| p.matches(filepath)))
        .filter(|filepath| Path::new(filepath).is_file())
}

/// Read the copyright years from an existing file
pub(crate) fn read_copyright_years(
    filepath: &Path,
    copyright_re: &Arc<Regex>,
) -> Option<(usize, String)> {
    let file = fs::File::open(filepath)
        .inspect_err(|e| eprintln!("Failed to read {}: {e}", filepath.display()))
        .ok()?;
    let file_header = BufReader::new(file).lines().take(3);

    for (line_idx, line) in file_header.enumerate() {
        if let Ok(line) = line
            && let Some(cap) = copyright_re.captures_iter(&line).take(1).next()
        {
            return Some((line_idx, cap[1].to_owned()));
        }
    }

    None
}

/// Write the copyright to the specified file
pub(crate) fn write_copyright(
    filepath: &Path,
    copyright_line: &str,
    line_idx: Option<usize>,
) -> Result<(), Error> {
    let mut content = String::new();
    fs::File::open(filepath)
        .and_then(|mut file| file.read_to_string(&mut content))
        .map_err(|e| Error::Io("reading file", e))?;

    // Create content with copyright added/updated
    let content = updated_content(&content, copyright_line, line_idx);

    fs::File::create(filepath)
        .and_then(|mut file| file.write_all(content.as_bytes()))
        .map_err(|e| Error::Io("writing file with copyright", e))?;

    Ok(())
}

fn updated_content(content: &str, copyright_line: &str, line_idx: Option<usize>) -> String {
    match line_idx {
        Some(line_idx) => {
            // Insert copyright where we found the outdated one
            content
                .split('\n')
                .enumerate()
                .flat_map(|(idx, line)| {
                    if idx == line_idx {
                        if idx == 0 {
                            ["", copyright_line]
                        } else {
                            ["\n", copyright_line]
                        }
                    } else if idx == 0 {
                        ["", line]
                    } else {
                        ["\n", line]
                    }
                })
                .collect::<String>()
        }
        None => {
            if !content.is_empty() && content.starts_with("#!") {
                // Insert copyright on the second line for shell scripts
                // that might have a shebang line
                let mut content_iter = content.split('\n');
                [
                    content_iter.next().unwrap_or_default(),
                    "\n",
                    copyright_line,
                ]
                .into_iter()
                .chain(content_iter.flat_map(|line| ["\n", line]))
                .collect::<String>()
            } else {
                // Insert copyright followed by a blank line on top
                [copyright_line, "\n\n", content]
                    .iter()
                    .copied()
                    .collect::<String>()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_new_copyright() {
        let original_content = r#"use std::path::Path;

fn main() {}
"#;
        let expected_content = r#"// Copyright 2026

use std::path::Path;

fn main() {}
"#;

        let copyright_line = "// Copyright 2026";
        let with_copyright = updated_content(original_content, copyright_line, None);

        assert_eq!(expected_content, with_copyright);
    }

    #[test]
    fn add_new_copyright_shebang() {
        let original_content = r#"#!/bin/bash

echo "Hello"
"#;
        let expected_content = r#"#!/bin/bash
# Copyright 2026

echo "Hello"
"#;

        let copyright_line = "# Copyright 2026";
        let with_copyright = updated_content(original_content, copyright_line, None);

        assert_eq!(expected_content, with_copyright);
    }

    #[test]
    fn update_existing_copyright() {
        let original_content = r#"// Copyright 2025

use std::path::Path;

fn main() {}
"#;
        let expected_content = r#"// Copyright 2026

use std::path::Path;

fn main() {}
"#;

        let copyright_line = "// Copyright 2026";
        let with_copyright = updated_content(original_content, copyright_line, Some(0));

        assert_eq!(expected_content, with_copyright);
    }

    #[test]
    fn update_copyright_shebang() {
        let original_content = r#"#!/bin/bash
# Copyright 2025

echo "Hello"
"#;
        let expected_content = r#"#!/bin/bash
# Copyright 2026

echo "Hello"
"#;

        let copyright_line = "# Copyright 2026";
        let with_copyright = updated_content(original_content, copyright_line, Some(1));

        assert_eq!(expected_content, with_copyright);
    }
}
