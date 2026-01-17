//! Add/update copyright notes according to the git history

use std::time::Instant;

use env_logger::TimestampPrecision;
use git_copyright::Config;
use git_copyright::cli::Args;
use git_copyright::error::Error;
use git_copyright::git_ops::check_for_changes;
use git_copyright::runner::check_repo_copyright;
use log::info;

fn main() -> Result<(), Error> {
    let args: Args = argh::from_env();

    env_logger::builder()
        .format_timestamp(Some(TimestampPrecision::Millis))
        .init();

    let config = match args.config_path {
        None => {
            info!("Using default configuration");
            Config::default()
        }
        Some(cfg_path) => {
            info!("Using config {}", cfg_path);
            Config::from_file(&cfg_path)?
        }
    };

    let start = Instant::now();
    let result = check_repo_copyright(config, &args.repo_path, &args.copyright_template);
    let duration = start.elapsed().as_millis() as f32 / 1000.;
    if let Err(e) = result {
        eprintln!("Failed to check repo copyright ({duration:0.3}s): {e}",);
    } else {
        println!("Copyrights checked and updated in {duration:0.3}s",);
    }

    check_for_changes(&args.repo_path, args.fail_on_changes)?;

    Ok(())
}
