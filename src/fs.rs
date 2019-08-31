use failure::ResultExt as _;
use filetime::FileTime;
use serde::de::DeserializeOwned;

use std::env;
use std::path::{Path, PathBuf};

pub(crate) fn current_dir() -> crate::Result<PathBuf> {
    env::current_dir()
        .with_context(|_| crate::ErrorKind::Getcwd)
        .map_err(Into::into)
}

pub(crate) fn move_dir_with_timestamps(
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
) -> crate::Result<()> {
    let (from, to) = (from.as_ref(), to.as_ref());
    from.metadata()
        .and_then(|metadata| {
            let atime = FileTime::from_last_access_time(&metadata);
            let mtime = FileTime::from_last_modification_time(&metadata);
            std::fs::rename(from, to)?;
            filetime::set_file_times(to, atime, mtime)
        })
        .with_context(|_| crate::ErrorKind::MoveDir {
            from: from.to_owned(),
            to: to.to_owned(),
        })
        .map_err(Into::into)
}

pub(crate) fn copy_dir(
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
    options: &fs_extra::dir::CopyOptions,
) -> crate::Result<u64> {
    let (from, to) = (from.as_ref(), to.as_ref());
    fs_extra::dir::copy(from, to, options)
        .with_context(|_| crate::ErrorKind::CopyDir {
            from: from.to_owned(),
            to: to.to_owned(),
        })
        .map_err(Into::into)
}

pub(crate) fn remove_dir_all(dir: impl AsRef<Path>) -> crate::Result<()> {
    let dir = dir.as_ref();
    remove_dir_all::remove_dir_all(dir)
        .with_context(|_| crate::ErrorKind::RemoveDir {
            dir: dir.to_owned(),
        })
        .map_err(Into::into)
}

pub(crate) fn read_toml<T: DeserializeOwned>(path: &Path) -> crate::Result<T> {
    let toml = std::fs::read_to_string(path).with_context(|_| crate::ErrorKind::ReadFile {
        path: path.to_owned(),
    })?;
    toml::from_str(&toml)
        .with_context(|_| crate::ErrorKind::Deserialize {
            what: path.display().to_string(),
        })
        .map_err(Into::into)
}

pub(crate) fn from_json<T: DeserializeOwned>(json: &str, what: &str) -> crate::Result<T> {
    serde_json::from_str(json)
        .with_context(|_| crate::ErrorKind::Deserialize {
            what: what.to_owned(),
        })
        .map_err(Into::into)
}
