pub mod config;
pub mod filename;

use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn default_receive_dir() -> io::Result<PathBuf> {
    let base = dirs_next::download_dir()
        .map(|path| path.join("DropLite"))
        .or_else(|| {
            dirs_next::data_local_dir()
                .map(|path| path.join("Easy-Use").join("DropLite").join("received"))
        })
        .unwrap_or_else(|| PathBuf::from("received"));

    ensure_dir(&base)?;
    Ok(base)
}

pub fn ensure_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}
