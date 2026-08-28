use regex::Regex;

use nomi::error::Result;

use crate::app::MatchMode;

#[derive(Debug)]
enum Matcher {
    Literal(String),
    Regex(Regex),
}

#[derive(Debug)]
pub struct RenameConfig {
    matcher: Matcher,
    replacement: String,
}

impl RenameConfig {
    pub fn new(pattern: &str, replacement: &str, match_mode: MatchMode) -> Result<Self> {
        let matcher = match match_mode {
            MatchMode::Literal => Matcher::Literal(pattern.to_owned()),
            MatchMode::Regex => Matcher::Regex(Regex::new(pattern)?),
        };

        Ok(Self {
            matcher,
            replacement: replacement.to_owned(),
        })
    }

    pub fn apply(&self, source: &str) -> String {
        match &self.matcher {
            Matcher::Literal(pattern) => source.replace(pattern, &self.replacement),
            Matcher::Regex(regex) => regex
                .replace_all(source, self.replacement.as_str())
                .into_owned(),
        }
    }
}

#[derive(Debug)]
pub struct Rename {
    source: String,
    destination: String,
}

impl Rename {
    pub fn new(source: String) -> Self {
        Self {
            destination: source.clone(),
            source,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn destination(&self) -> &str {
        &self.destination
    }

    pub fn preview(&mut self, config: &RenameConfig) {
        self.destination = config.apply(&self.source);
    }
}
