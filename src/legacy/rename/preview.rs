use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::error::NomiError;

use super::{Entry, MatchMode, execute::RenameTransaction};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenameOp {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Debug)]
pub struct RenamePreview {
    pub names: Vec<Option<String>>,
    pub operations: Vec<RenameOp>,
    pub error: Option<String>,
}

impl RenamePreview {
    pub fn build(
        directory: &Path,
        entries: &[Entry],
        pattern: &str,
        replacement: &str,
        mode: MatchMode,
    ) -> Self {
        let mut preview = Self::empty(entries.len());
        if pattern.is_empty() {
            return preview;
        }

        let rule = match RenameRule::compile(pattern, replacement, mode) {
            Ok(rule) => rule,
            Err(error) => {
                preview.error = Some(format!("Invalid regex: {error}"));
                return preview;
            }
        };

        for (index, entry) in entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.selected)
        {
            let Some(original) = entry.name.to_str() else {
                preview.error = Some(format!(
                    "'{}' is not valid UTF-8 and cannot be renamed",
                    entry.name.to_string_lossy()
                ));
                return preview;
            };
            let Some(destination) = rule.apply(original) else {
                continue;
            };

            preview.names[index] = Some(destination.clone());
            preview.operations.push(RenameOp {
                from: directory.join(&entry.name),
                to: directory.join(destination),
            });
        }

        if let Err(error) = preview.validate(directory) {
            preview.error = Some(error.to_string());
        }
        preview
    }

    pub fn execute(&self, directory: &Path) -> Result<(), NomiError> {
        self.validate(directory)?;
        RenameTransaction::new(directory, &self.operations).execute()
    }

    pub fn len(&self) -> usize {
        self.operations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    fn empty(entry_count: usize) -> Self {
        Self {
            names: vec![None; entry_count],
            operations: Vec::new(),
            error: None,
        }
    }

    fn validate(&self, directory: &Path) -> Result<(), NomiError> {
        let sources: HashSet<&Path> = self
            .operations
            .iter()
            .map(|operation| operation.from.as_path())
            .collect();
        let mut destinations: HashMap<String, &Path> = HashMap::new();

        for operation in &self.operations {
            if operation.from.parent() != Some(directory) {
                return Err(NomiError::Validation(format!(
                    "'{}' is outside the rename directory",
                    operation.from.display()
                )));
            }
            let name = destination_name(directory, operation)?;
            let key = destination_key(&name);

            if let Some(existing) = destinations.insert(key, operation.to.as_path()) {
                return Err(NomiError::Validation(format!(
                    "both '{}' and another entry would become '{}'",
                    existing.display(),
                    name
                )));
            }
            if operation.to.exists() && !sources.contains(operation.to.as_path()) {
                return Err(NomiError::Validation(format!(
                    "'{}' already exists",
                    operation.to.display()
                )));
            }
        }
        Ok(())
    }
}

enum RenameRule<'a> {
    Literal {
        pattern: &'a str,
        replacement: &'a str,
    },
    Regex {
        pattern: Regex,
        replacement: &'a str,
    },
}

impl<'a> RenameRule<'a> {
    fn compile(
        pattern: &'a str,
        replacement: &'a str,
        mode: MatchMode,
    ) -> Result<Self, regex::Error> {
        match mode {
            MatchMode::Literal => Ok(Self::Literal {
                pattern,
                replacement,
            }),
            MatchMode::Regex => Ok(Self::Regex {
                pattern: Regex::new(pattern)?,
                replacement,
            }),
        }
    }

    fn apply(&self, original: &str) -> Option<String> {
        let transformed = match self {
            Self::Literal {
                pattern,
                replacement,
            } if original.contains(pattern) => original.replace(pattern, replacement),
            Self::Literal { .. } => return None,
            Self::Regex {
                pattern,
                replacement,
            } => pattern.replace_all(original, *replacement).into_owned(),
        };

        (transformed != original).then_some(transformed)
    }
}

fn destination_name<'a>(
    directory: &Path,
    operation: &'a RenameOp,
) -> Result<std::borrow::Cow<'a, str>, NomiError> {
    let Some(name) = operation.to.file_name() else {
        return Err(NomiError::Validation("a destination name is empty".into()));
    };
    let name = name.to_string_lossy();

    if name.is_empty() || name == "." || name == ".." {
        return Err(NomiError::Validation(format!(
            "'{name}' is not a valid filename"
        )));
    }
    if operation.to.parent() != Some(directory) {
        return Err(NomiError::Validation(format!(
            "'{name}' contains a path separator"
        )));
    }

    Ok(name)
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn destination_key(name: &str) -> String {
    name.to_lowercase()
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn destination_key(name: &str) -> String {
    name.to_owned()
}
