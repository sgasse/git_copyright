//! Runner definition

use std::path::Path;
use std::sync::Arc;
use std::thread;

use crossbeam_channel::{Receiver, Sender};
use log::debug;

use crate::Config;
use crate::error::Error;
use crate::file_ops::{filter_files, read_copyright_years, write_copyright};
use crate::git_ops::{get_added_mod_times_for_file, get_files_on_ref};
use crate::regex_ops::{RegexCache, generate_copyright_line};

/// Check the copyrights of tracked files in a repository
pub fn check_repo_copyright(
    config: Config,
    repo_path: &str,
    copyright_template: &str,
) -> Result<(), Error> {
    let config = Arc::new(config);
    let files_to_check = get_files_on_ref(repo_path, "HEAD")?;
    let files_to_check = filter_files(config.glob_pattern(), files_to_check).into_iter();

    let (filenames_tx, filenames_rx) = crossbeam_channel::bounded(64);
    let (errors_tx, errors_rx) = crossbeam_channel::bounded(64);

    let regex_cache = Arc::new(RegexCache::new(copyright_template));

    // Spawn one runner per CPU to check files in parallel
    let runners: Vec<_> = (0..num_cpus::get())
        .map(|id| {
            debug!("Spawning runner {id}");
            let filename_rx = filenames_rx.clone();
            let errors_tx = errors_tx.clone();
            let regex_cache = Arc::clone(&regex_cache);
            let repo_path = repo_path.to_owned();
            let copyright_template = copyright_template.to_owned();
            let config = Arc::clone(&config);

            thread::spawn(move || {
                file_checker(
                    filename_rx,
                    repo_path,
                    errors_tx,
                    regex_cache,
                    copyright_template,
                    config,
                )
            })
        })
        .collect();

    let mut errors = vec![];

    // Pass all filenames to check to the runners
    for filename in files_to_check {
        filenames_tx
            .send(filename)
            .expect("failed to send filename to runner");

        // Retrieve errors after each filename sent
        while let Ok(err) = errors_rx.try_recv() {
            errors.push(err);
        }
    }

    // Close the filenames channel to trigger runner shutdown
    drop(filenames_tx);

    // Join all runners
    for runner in runners {
        runner.join().expect("failed to join runner");
    }

    // Report all encountered errors
    if !errors.is_empty() {
        println!("Encountered errors while checking copyrights:");
        for error in errors.iter() {
            println!("{error}");
        }
        return Err(errors.into_iter().next().unwrap());
    }

    Ok(())
}

fn file_checker(
    filename_rx: Receiver<String>,
    repo_path: String,
    errors_tx: Sender<Error>,
    regex_cache: Arc<RegexCache>,
    copyright_template: String,
    config: Arc<Config>,
) {
    loop {
        let Ok(filename) = filename_rx.recv() else {
            break;
        };

        let filepath = Path::new(&repo_path).join(&filename);

        let tracked_years = get_added_mod_times_for_file(&filepath, &repo_path);
        let Ok(comment_sign) = config.get_comment_sign(&filename) else {
            errors_tx.send(Error::UnknownCommentSign(filename)).ok();
            continue;
        };

        let copyright_re = regex_cache.get_regex(comment_sign);
        let copyright = generate_copyright_line(&copyright_template, comment_sign, &tracked_years);

        match read_copyright_years(&filepath, &copyright_re) {
            Some((_, copyright_years)) if copyright_years == tracked_years => {
                debug!(
                    "File {} has correct copyright with years {tracked_years}",
                    filepath.display()
                );
            }
            Some((line, copyright_years)) => {
                println!(
                    "File {} has copyright with year(s) {copyright_years} on line {line} but should have {tracked_years}",
                    filepath.display()
                );
                if let Err(e) = write_copyright(&filepath, &copyright, Some(line)) {
                    errors_tx.send(e).ok();
                }
            }
            None => {
                println!(
                    "File {} has no copyright but should have {tracked_years}",
                    filepath.display()
                );
                if let Err(e) = write_copyright(&filepath, &copyright, None) {
                    errors_tx.send(e).ok();
                }
            }
        }
    }
}
