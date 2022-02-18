use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::Write as _;
use std::path::Path;

use crate::errors::EmptyResult;

/// Copy a directory and all its contents to the destination directory.
/// <https://stackoverflow.com/a/65192210>
pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn save<S>(path: &S, text: String) -> EmptyResult
where
    S: AsRef<OsStr>,
{
    let file_path = Path::new(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    match fs::remove_file(file_path) {
        Ok(_) => (),
        Err(_) => (),
    };
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(file_path)?;

    file.write_all(text.as_bytes())?;
    Ok(())
}
