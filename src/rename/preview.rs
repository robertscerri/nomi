use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::error::NomiError;

use super::{Entry, MatchMode};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenameOp {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Debug)]
pub struct Preview {
    pub names: Vec<Option<String>>,
    pub operations: Vec<RenameOp>,
    pub error: Option<String>,
}

pub fn build_preview(
    directory: &Path,
    entries: &[Entry],
    pattern: &str,
    replacement: &str,
    mode: MatchMode,
) -> Preview {
    let mut preview = Preview::empty(entries.len());
    if pattern.is_empty() {
        return preview;
    }

    let regex = match compile_pattern(pattern, mode) {
        Ok(regex) => regex,
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
        let original = entry.name.to_string_lossy();
        let Some(destination) = transform_name(&original, pattern, replacement, regex.as_ref())
        else {
            continue;
        };

        preview.names[index] = Some(destination.clone());
        preview.operations.push(RenameOp {
            from: directory.join(&entry.name),
            to: directory.join(destination),
        });
    }

    if let Err(error) = validate(directory, &preview.operations) {
        preview.error = Some(error.to_string());
    }
    preview
}

impl Preview {
    fn empty(entry_count: usize) -> Self {
        Self {
            names: vec![None; entry_count],
            operations: Vec::new(),
            error: None,
        }
    }
}

fn compile_pattern(pattern: &str, mode: MatchMode) -> Result<Option<Regex>, regex::Error> {
    match mode {
        MatchMode::Regex => Regex::new(pattern).map(Some),
        MatchMode::Literal => Ok(None),
    }
}

fn transform_name(
    original: &str,
    pattern: &str,
    replacement: &str,
    regex: Option<&Regex>,
) -> Option<String> {
    let transformed = match regex {
        Some(regex) => regex.replace_all(original, replacement).into_owned(),
        None if original.contains(pattern) => original.replace(pattern, replacement),
        None => return None,
    };

    (transformed != original).then_some(transformed)
}

pub fn validate(directory: &Path, operations: &[RenameOp]) -> Result<(), NomiError> {
    let sources: HashSet<&Path> = operations
        .iter()
        .map(|operation| operation.from.as_path())
        .collect();
    let mut destinations: HashMap<String, &Path> = HashMap::new();

    for operation in operations {
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
