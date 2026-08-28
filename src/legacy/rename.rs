mod execute;
mod preview;

use std::{ffi::OsString, fs, path::Path};

use crate::error::NomiError;

pub use preview::{RenameOp, RenamePreview};

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
