use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::NomiError;

use super::preview::RenameOp;

pub(super) struct RenameTransaction<'a> {
    operations: &'a [RenameOp],
    temporary: Vec<PathBuf>,
}

impl<'a> RenameTransaction<'a> {
    pub(super) fn new(directory: &Path, operations: &'a [RenameOp]) -> Self {
        let process_id = std::process::id();
        let temporary = (0..operations.len())
            .map(|index| unique_temp_path(directory, process_id, index))
            .collect();

        Self {
            operations,
            temporary,
        }
    }

    pub(super) fn execute(&self) -> Result<(), NomiError> {
        if self.operations.is_empty() {
            return Ok(());
        }

        self.stage()?;
        self.commit()
    }

    fn stage(&self) -> Result<(), NomiError> {
        for (staged, (operation, temp)) in self.operations.iter().zip(&self.temporary).enumerate() {
            if let Err(source) = fs::rename(&operation.from, temp) {
                let rollback_errors = self.restore_staged(staged);
                return rename_failure(&operation.from, temp, source, rollback_errors);
            }
        }
        Ok(())
    }

    fn commit(&self) -> Result<(), NomiError> {
        for (committed, (operation, temp)) in
            self.operations.iter().zip(&self.temporary).enumerate()
        {
            if let Err(source) = fs::rename(temp, &operation.to) {
                let mut rollback_errors = self.restore_committed(committed);
                rollback_errors.extend(self.restore_staged(self.operations.len()));
                return rename_failure(temp, &operation.to, source, rollback_errors);
            }
        }
        Ok(())
    }

    fn restore_committed(&self, committed: usize) -> Vec<String> {
        let mut errors = Vec::new();
        for operation in self.operations[..committed].iter().rev() {
            if let Err(error) = fs::rename(&operation.to, &operation.from) {
                errors.push(format!(
                    "could not restore '{}': {error}",
                    operation.from.display()
                ));
            }
        }
        errors
    }

    fn restore_staged(&self, staged: usize) -> Vec<String> {
        let mut errors = Vec::new();
        for index in (0..staged).rev() {
            if self.temporary[index].exists()
                && let Err(error) = fs::rename(&self.temporary[index], &self.operations[index].from)
            {
                errors.push(format!(
                    "could not restore '{}': {error}",
                    self.operations[index].from.display()
                ));
            }
        }
        errors
    }
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
