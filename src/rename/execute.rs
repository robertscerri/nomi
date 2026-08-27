use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::NomiError;

use super::preview::{RenameOp, validate};

pub fn execute(directory: &Path, operations: &[RenameOp]) -> Result<(), NomiError> {
    validate(directory, operations)?;
    if operations.is_empty() {
        return Ok(());
    }

    let temporary = temporary_paths(directory, operations.len());
    stage_operations(operations, &temporary)?;
    commit_operations(operations, &temporary)
}

fn temporary_paths(directory: &Path, count: usize) -> Vec<PathBuf> {
    let process_id = std::process::id();
    (0..count)
        .map(|index| unique_temp_path(directory, process_id, index))
        .collect()
}

fn stage_operations(operations: &[RenameOp], temporary: &[PathBuf]) -> Result<(), NomiError> {
    for (staged, (operation, temp)) in operations.iter().zip(temporary).enumerate() {
        if let Err(source) = fs::rename(&operation.from, temp) {
            let rollback_errors = restore_staged(operations, temporary, staged);
            return rename_failure(&operation.from, temp, source, rollback_errors);
        }
    }
    Ok(())
}

fn commit_operations(operations: &[RenameOp], temporary: &[PathBuf]) -> Result<(), NomiError> {
    for (committed, (operation, temp)) in operations.iter().zip(temporary).enumerate() {
        if let Err(source) = fs::rename(temp, &operation.to) {
            let mut rollback_errors = restore_committed(operations, committed);
            rollback_errors.extend(restore_staged(operations, temporary, operations.len()));
            return rename_failure(temp, &operation.to, source, rollback_errors);
        }
    }
    Ok(())
}

fn restore_committed(operations: &[RenameOp], committed: usize) -> Vec<String> {
    let mut errors = Vec::new();
    for operation in operations[..committed].iter().rev() {
        if let Err(error) = fs::rename(&operation.to, &operation.from) {
            errors.push(format!(
                "could not restore '{}': {error}",
                operation.from.display()
            ));
        }
    }
    errors
}

fn restore_staged(operations: &[RenameOp], temporary: &[PathBuf], staged: usize) -> Vec<String> {
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

fn rename_failure(
    from: &Path,
    to: &Path,
    source: std::io::Error,
    rollback_errors: Vec<String>,
) -> Result<(), NomiError> {
    if rollback_errors.is_empty() {
        Err(NomiError::Rename {
            from: from.to_owned(),
            to: to.to_owned(),
            source,
        })
    } else {
        Err(NomiError::Rollback {
            message: format!("{source}; {}", rollback_errors.join("; ")),
        })
    }
}

fn unique_temp_path(directory: &Path, process_id: u32, index: usize) -> PathBuf {
    let mut attempt = 0;
    loop {
        let candidate = directory.join(format!(".nomi-{process_id}-{index}-{attempt}.tmp"));
        if !candidate.exists() {
            return candidate;
        }
        attempt += 1;
    }
}
