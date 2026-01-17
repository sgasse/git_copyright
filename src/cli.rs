//! Command-line arguments

use argh::FromArgs;

/// Check and update copyright of git repository
#[derive(FromArgs)]
pub struct Args {
    /// path to repository
    #[argh(option, default = "String::from(\"./\")")]
    pub repo_path: String,

    /// template for the copyright (include `{years}` as placeholder)
    #[argh(
        option,
        default = "String::from(r#\"Copyright {years} DummyCorp. All rights reserved.\"#)"
    )]
    pub copyright_template: String,

    /// path to the configuration file
    #[argh(option)]
    pub config_path: Option<String>,

    /// fail on changes
    #[argh(switch)]
    pub fail_on_changes: bool,
}
