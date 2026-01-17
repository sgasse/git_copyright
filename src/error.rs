//! Error definition

/// Error of checking copyright
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The copyright of some files have changed")]
    FilesChanged,

    #[error("I/O error while {0}: {1}")]
    Io(&'static str, std::io::Error),

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Failed to run git subcommand: {0}")]
    GitCommand(String),

    #[error("Failed to parse config: {0}")]
    ParseConfig(toml::de::Error),

    #[error("No comment sign found for file {0}, please update the configuration")]
    UnknownCommentSign(String),
}
