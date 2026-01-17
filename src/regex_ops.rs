//! Compile and cache copyright regexes
//!
//! This module contains functions to parse existing copyright notes.
//! Regexes are compiled once per comment sign and stored in a cache.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hasher as _};
use std::sync::{Arc, RwLock};

use log::debug;
use regex::Regex;

use crate::CommentSign;

/// Generate a copyright line based on the template
///
/// The template has to contain `{years}` for the year,
/// e.g. `Copyright (c) DummyCompany Ltd. {years}`
/// or `Copyright {years} DummyCompany. All rights reserved.`
pub(crate) fn generate_copyright_line(
    template: &str,
    comment_sign: &CommentSign,
    years: &str,
) -> String {
    let copyright = template.replace(r"{years}", years);

    match comment_sign {
        CommentSign::LeftOnly(left) => [left, " ", &copyright].join(" "),
        CommentSign::Enclosing(left, right) => [left, " ", &copyright, " ", right].join(" "),
    }
}

/// Copyright regex cache
pub(crate) struct RegexCache {
    regexes: RwLock<HashMap<u64, Arc<Regex>>>,
    template: String,
}

impl RegexCache {
    /// Create a new instance
    pub(crate) fn new(template: &str) -> Self {
        RegexCache {
            regexes: RwLock::new(HashMap::new()),
            template: template.to_owned(),
        }
    }

    /// Get regex for a certain comment sign
    pub(crate) fn get_regex(&self, comment_sign: &CommentSign) -> Arc<Regex> {
        let comment_sign_hash = get_hash(comment_sign);

        if let Some(regex) = self.regexes.read().unwrap().get(&comment_sign_hash) {
            return Arc::clone(regex);
        }

        debug!("Initializing regex for comment sign {comment_sign:?}");
        let regex = Arc::new(generate_copyright_regex(&self.template, comment_sign));
        self.regexes
            .write()
            .unwrap()
            .insert(comment_sign_hash, Arc::clone(&regex));

        regex
    }
}

fn escape_for_regex(text: &str) -> String {
    text.chars()
        .map(|char| match char {
            '*' => String::from(r"\*"),
            '.' => String::from(r"\."),
            '(' => String::from(r"\("),
            ')' => String::from(r"\)"),
            '{' => String::from(r"\{"),
            '}' => String::from(r"\}"),
            '^' => String::from(r"\^"),
            '$' => String::from(r"\$"),
            other => String::from(other),
        })
        .collect::<String>()
}

/// Turn a copyright template into a regex
///
/// The template has to contain `{year}` for the year,
/// e.g. `Copyright (c) DummyCompany Ltd. {year}`
/// or `Copyright {year} DummyCompany. All rights reserved.`
fn generate_copyright_regex(template: &str, comment_sign: &CommentSign) -> Regex {
    const YEARS_REGEX: &str = r"(\d{4}(-\d{4}){0,1})";
    const ESCAPED_YEARS_PLACEHOLDER: &str = r"\{years\}";

    let template = escape_for_regex(template).replace(ESCAPED_YEARS_PLACEHOLDER, YEARS_REGEX);

    let regex_expr = match comment_sign {
        CommentSign::LeftOnly(left_sign) => {
            ["^", &escape_for_regex(left_sign), " ", &template, "$"].join("")
        }

        CommentSign::Enclosing(left_sign, right_sign) => [
            "^",
            &escape_for_regex(left_sign),
            " ",
            &template,
            " ",
            &escape_for_regex(right_sign),
            "$",
        ]
        .join(""),
    };

    Regex::new(&regex_expr).unwrap()
}

fn get_hash<T: std::hash::Hash>(obj: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_escape_for_regex() {
        assert_eq!(escape_for_regex("/"), r"/");
        assert_eq!(escape_for_regex("//"), r"//");
        assert_eq!(escape_for_regex("/*"), r"/\*");
        assert_eq!(escape_for_regex("*/"), r"\*/");
        assert_eq!(escape_for_regex("#"), "#");
    }

    #[test]
    fn test_regex_match() {
        let valid_copyrights = [
            "# Copyright (c) DummyCompany Ltd. 2019",
            "# Copyright (c) DummyCompany Ltd. 2020-2021",
        ];
        let invalid_copyrights = [
            "# Copyright (c) DummyCompany Ltd. 2019-",
            "# Copyright (c) DummyCompany Ltd. 2020-2021-2023",
            "# Copyright (c) DummyCompany Ltd. 20202021",
        ];

        let template = "Copyright (c) DummyCompany Ltd. {years}";
        let comment_sign = CommentSign::LeftOnly("#".into());
        let copyright_re = generate_copyright_regex(template, &comment_sign);

        for example in valid_copyrights {
            assert!(copyright_re.is_match(example));
        }

        for example in invalid_copyrights {
            assert!(!copyright_re.is_match(example));
        }
    }
}
