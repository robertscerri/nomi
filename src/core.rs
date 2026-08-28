use std::{fs, path::Path};

use crate::error::{Error, Result};

pub fn read_entries(directory: &Path) -> Result<Vec<String>> {
    if !directory.is_dir() {
        return Err(Error::Validation(format!(
            "'{}' is not a directory",
            directory.display()
        )));
    }

    let mut entries: Vec<String> = fs::read_dir(directory)?
        .map(|item| {
            let file_name = item?.file_name();
            Ok(file_name.to_string_lossy().to_string())
        })
        .collect::<Result<Vec<_>>>()?;

    entries.sort_by_key(|entry| entry.to_ascii_lowercase());
    Ok(entries)
}
