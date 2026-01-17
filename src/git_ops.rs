//! Git operations

use std::path::Path;
use std::process::{self, Command};

use chrono::Utc;
use log::debug;

use crate::error::Error;

/// Check the repository for changes of tracked files
pub fn check_for_changes(repo_path: &str, fail_on_changes: bool) -> Result<(), Error> {
    let diff_files = get_diffs(repo_path)?;
    if !diff_files.is_empty() {
        println!("The following files have changed:");
        for file in diff_files {
            println!("- {file}");
        }

        if fail_on_changes {
            return Err(Error::FilesChanged);
        }
    }

    Ok(())
}

/// Get all tracked files on a `git` reference
pub(crate) fn get_files_on_ref(repo_path: &str, ref_name: &str) -> Result<Vec<String>, Error> {
    let output = Command::new("git")
        .arg("ls-tree")
        .arg("-r")
        .arg(ref_name)
        .arg("--name-only")
        .current_dir(repo_path)
        .output()
        .map_err(|e| Error::Io("getting files on ref", e))?;

    parse_cmd_output(output)
}

/// Get the added and modified times for a file in a git repository
pub(crate) fn get_added_mod_times_for_file(filepath: &Path, repo_path: &str) -> String {
    let output = Command::new("git")
        .arg("log")
        .arg("--follow")
        .arg("-m")
        .arg("--pretty=%ci")
        .arg(filepath)
        .current_dir(repo_path)
        .output()
        .expect("failed to run `git log`");
    let commit_years: Vec<String> = str::from_utf8(&output.stdout)
        .expect("failed to parse command output as utf8")
        .split('\n')
        .filter(|s| !s.is_empty())
        // Take only first four chars (the year)
        .map(|s| s.chars().take(4).collect())
        .collect();

    match commit_years.len() {
        0 => {
            debug!("File {} is untracked, add current year", filepath.display());
            Utc::now().date_naive().format("%Y").to_string()
        }
        1 => {
            debug!("File {} was only committed once", filepath.display());
            commit_years[0].clone()
        }
        num_commits => {
            debug!(
                "File {} was modified {num_commits} times",
                filepath.display()
            );
            let mut years_iter = commit_years.into_iter();
            let last_modified = years_iter.next().unwrap();
            let added = years_iter.last().unwrap();

            match added == last_modified {
                true => added,
                false => format!("{}-{}", added, last_modified),
            }
        }
    }
}

fn get_diffs(repo_path: &str) -> Result<Vec<String>, Error> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--name-only")
        .current_dir(repo_path)
        .output()
        .map_err(|e| Error::Io("checking for diffs", e))?;

    parse_cmd_output(output)
}

fn parse_cmd_output(output: process::Output) -> Result<Vec<String>, Error> {
    if !output.status.success() {
        return Err(Error::GitCommand(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    str::from_utf8(&output.stdout)
        .map(|output| {
            output
                .split('\n')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_owned())
                .collect()
        })
        .map_err(Into::into)
}
