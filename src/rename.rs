use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::error::NomiError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchMode {
    Literal,
    Regex,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub name: OsString,
    pub selected: bool,
    pub is_dir: bool,
}

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

pub fn read_entries(directory: &Path) -> Result<Vec<Entry>, NomiError> {
    if !directory.is_dir() {
        return Err(NomiError::Validation(format!(
            "'{}' is not a directory",
            directory.display()
        )));
    }

    let mut entries = fs::read_dir(directory)?
        .map(|item| {
            let item = item?;
            let file_type = item.file_type()?;
            Ok(Entry {
                name: item.file_name(),
                selected: true,
                is_dir: file_type.is_dir(),
            })
        })
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    entries.sort_by_key(|entry| entry.name.to_string_lossy().to_lowercase());
    Ok(entries)
}

pub fn build_preview(
    directory: &Path,
    entries: &[Entry],
    pattern: &str,
    replacement: &str,
    mode: MatchMode,
) -> Preview {
    let mut preview = Preview {
        names: vec![None; entries.len()],
        operations: Vec::new(),
        error: None,
    };
    if pattern.is_empty() {
        return preview;
    }

    let regex = if mode == MatchMode::Regex {
        match Regex::new(pattern) {
            Ok(regex) => Some(regex),
            Err(error) => {
                preview.error = Some(format!("Invalid regex: {error}"));
                return preview;
            }
        }
    } else {
        None
    };

    for (index, entry) in entries.iter().enumerate() {
        if !entry.selected {
            continue;
        }
        let original = entry.name.to_string_lossy();
        let transformed = match &regex {
            Some(regex) if regex.is_match(&original) => {
                regex.replace_all(&original, replacement).into_owned()
            }
            Some(_) => continue,
            None if original.contains(pattern) => original.replace(pattern, replacement),
            None => continue,
        };
        if transformed == original {
            continue;
        }
        preview.names[index] = Some(transformed.clone());
        preview.operations.push(RenameOp {
            from: directory.join(&entry.name),
            to: directory.join(transformed),
        });
    }

    if let Err(error) = validate(directory, &preview.operations) {
        preview.error = Some(error.to_string());
    }
    preview
}

pub fn validate(directory: &Path, operations: &[RenameOp]) -> Result<(), NomiError> {
    let sources: HashSet<&Path> = operations.iter().map(|op| op.from.as_path()).collect();
    let mut destinations: HashMap<String, &Path> = HashMap::new();

    for operation in operations {
        let Some(name) = operation.to.file_name() else {
            return Err(NomiError::Validation("a destination name is empty".into()));
        };
        let name = name.to_string_lossy();
        if name.is_empty() || name == "." || name == ".." {
            return Err(NomiError::Validation(format!(
                "'{}' is not a valid filename",
                name
            )));
        }
        if operation.to.parent() != Some(directory) {
            return Err(NomiError::Validation(format!(
                "'{}' contains a path separator",
                name
            )));
        }

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

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn destination_key(name: &str) -> String {
    name.to_lowercase()
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn destination_key(name: &str) -> String {
    name.to_owned()
}

pub fn execute(directory: &Path, operations: &[RenameOp]) -> Result<(), NomiError> {
    validate(directory, operations)?;
    if operations.is_empty() {
        return Ok(());
    }

    let nonce = std::process::id();
    let temporary: Vec<PathBuf> = operations
        .iter()
        .enumerate()
        .map(|(index, _)| unique_temp_path(directory, nonce, index))
        .collect();

    for (staged, (operation, temp)) in operations.iter().zip(&temporary).enumerate() {
        if let Err(source) = fs::rename(&operation.from, temp) {
            let rollback_errors = rollback_staged(operations, &temporary, staged);
            return if rollback_errors.is_empty() {
                Err(NomiError::Rename {
                    from: operation.from.clone(),
                    to: temp.clone(),
                    source,
                })
            } else {
                Err(NomiError::Rollback {
                    message: format!("{source}; {}", rollback_errors.join("; ")),
                })
            };
        }
    }

    let mut committed = 0;
    for ((operation, temp), index) in operations.iter().zip(&temporary).zip(0..) {
        if let Err(source) = fs::rename(temp, &operation.to) {
            let mut errors = Vec::new();
            for completed in (0..committed).rev() {
                if let Err(error) =
                    fs::rename(&operations[completed].to, &operations[completed].from)
                {
                    errors.push(error.to_string());
                }
            }
            errors.extend(rollback_staged(operations, &temporary, operations.len()));
            return if errors.is_empty() {
                Err(NomiError::Rename {
                    from: temp.clone(),
                    to: operation.to.clone(),
                    source,
                })
            } else {
                Err(NomiError::Rollback {
                    message: format!("{source}; {}", errors.join("; ")),
                })
            };
        }
        committed = index + 1;
    }
    Ok(())
}

fn unique_temp_path(directory: &Path, nonce: u32, index: usize) -> PathBuf {
    let mut attempt = 0;
    loop {
        let candidate = directory.join(format!(".nomi-{nonce}-{index}-{attempt}.tmp"));
        if !candidate.exists() {
            return candidate;
        }
        attempt += 1;
    }
}

fn rollback_staged(operations: &[RenameOp], temporary: &[PathBuf], staged: usize) -> Vec<String> {
    let mut errors = Vec::new();
    for index in (0..staged).rev() {
        if temporary[index].exists()
            && let Err(error) = fs::rename(&temporary[index], &operations[index].from)
        {
            errors.push(format!(
                "could not restore '{}': {error}",
                operations[index].from.display()
            ));
        }
    }
    errors
}
