use regex::Regex;

use nomi::error::Result;

use crate::app::MatchMode;

#[derive(Debug)]
pub struct RenameConfig {
    pattern: String,
    replacement: String,
    match_mode: MatchMode,
}

impl RenameConfig {
    pub fn new(pattern: String, replacement: String, match_mode: MatchMode) -> Self {
        RenameConfig {
            pattern,
            replacement,
            match_mode,
        }
    }

    pub fn apply(&self, target: String) -> Result<String> {
        // TODO: Compile Regex for performance?
        match self.match_mode {
            MatchMode::Literal => Ok(target.replace(&self.pattern, &self.replacement)),
            MatchMode::Regex => {
                let re = Regex::new(&self.pattern)?;
                let output = re.replace_all(&target, self.replacement.clone());
                Ok(output.to_string())
            }
        }
    }
}
