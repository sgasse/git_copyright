//! Extract added/last modified times from git history and add/update copyright note.

pub mod cli;
pub mod config;
pub mod error;
pub mod file_ops;
pub mod git_ops;
pub mod regex_ops;
pub mod runner;

pub use config::Config;
use serde::Deserialize;

/// Comment sign for a specific file type
#[derive(Debug, Deserialize, Hash, PartialEq)]
#[serde(untagged)]
pub enum CommentSign {
    LeftOnly(String),
    Enclosing(String, String),
}
