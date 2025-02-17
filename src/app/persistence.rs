use std::{
    fs::File,
    io::{BufReader, BufWriter, Error as IoError, ErrorKind as IoErrorKind},
    path::Path,
};

use ron::ser::PrettyConfig;
use serde::{de::DeserializeOwned, Serialize};

/// An error related to persistent application state
#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("IO error: {0}")]
    IoError(#[from] IoError),
    #[error("serialization error: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
    #[error("serialization error: {0}")]
    RonError(#[from] ron::error::Error),
}

/// Attempt to load persistent data from the given path
///
/// This will return `Ok(None)` if the file does not exist yet
pub fn load_data<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, PersistenceError> {
    match File::open(path) {
        Ok(file) => Ok(Some(ron::de::from_reader(BufReader::new(file))?)),
        Err(err) if err.kind() == IoErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

/// Attempt to save persistent data to the given path, creating parent
/// directories and the data file itself as necessary
pub fn save_data<T: Serialize>(data: &T, path: &Path) -> Result<(), PersistenceError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = BufWriter::new(File::create(path)?);
    ron::ser::to_writer_pretty(file, data, PrettyConfig::new())?;
    Ok(())
}
