//! Configuration
//!
//! If no custom configuration is specified, we fall back to the default
//! configuration which is included as bytes in the compiled binary.

use std::collections::HashMap;
use std::path::Path;

use glob::Pattern;
use log::error;
use serde::Deserialize;

use crate::CommentSign;
use crate::error::Error;

/// Configuration
#[derive(Debug, Deserialize)]
pub struct Config {
    comment_sign_map: HashMap<String, CommentSign>,
    ignore_files: Vec<String>,
    ignore_dirs: Vec<String>,
    #[serde(skip)]
    glob_pattern: Vec<Pattern>,
}

impl Config {
    /// Try to parse a configuration from a file
    pub fn from_file(cfg_file: &str) -> Result<Self, Error> {
        let cfg_str = std::fs::read_to_string(cfg_file).map_err(|e| Error::Io("read config", e))?;
        cfg_str.parse()
    }

    /// Try to get the comment sign for the specified filename
    pub fn get_comment_sign(&self, filename: &str) -> Result<&CommentSign, Error> {
        let filepath = Path::new(filename);
        filepath
            .extension()
            .or_else(|| filepath.file_name())
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.comment_sign_map.get(ext))
            .ok_or_else(|| Error::UnknownCommentSign(filename.into()))
    }

    /// Get the glob patterns specified in the configuration
    pub fn glob_pattern(&self) -> &[Pattern] {
        &self.glob_pattern
    }

    /// Build glob patterns specified in the configuration
    fn build_glob_pattern(&mut self) {
        self.glob_pattern = self
            .ignore_files
            .iter()
            .chain(self.ignore_dirs.iter())
            .filter_map(|expr| {
                Pattern::new(expr)
                    .inspect_err(|e| error!("Failed to compile glob pattern: {e}"))
                    .ok()
            })
            .collect();
    }
}

impl Default for Config {
    fn default() -> Self {
        let cfg_bytes = include_bytes!("./default_config.toml");
        let cfg_str = String::from_utf8_lossy(cfg_bytes);
        cfg_str.parse().expect("Failed to load default config")
    }
}

impl std::str::FromStr for Config {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cfg = toml::from_str::<Self>(s).map_err(Error::ParseConfig)?;
        cfg.build_glob_pattern();
        Ok(cfg)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_config_from_file() {
        let cfg = Config::from_file("./src/default_config.toml").unwrap();
        assert_eq!(
            cfg.get_comment_sign("file.rs").unwrap(),
            &CommentSign::LeftOnly("//".into())
        );

        let cfg = Config::default();
        assert_eq!(
            cfg.get_comment_sign("file.py").unwrap(),
            &CommentSign::LeftOnly("#".into())
        );
    }
}
