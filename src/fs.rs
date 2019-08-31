use failure::ResultExt as _;
use filetime::FileTime;
use fs2::FileExt as _;
use serde::de::DeserializeOwned;

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read as _, Seek as _, SeekFrom, Write as _};
use std::marker::PhantomData;
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

pub(crate) struct ExclusivelyLockedJsonFile<T: Default + miniserde::Serialize + DeserializeOwned> {
    file: File,
    path: PathBuf,
    phantom: PhantomData<fn() -> T>,
}

impl<T: Default + miniserde::Serialize + DeserializeOwned> ExclusivelyLockedJsonFile<T> {
    pub(crate) fn open<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let path = path.as_ref().to_owned();
        let new = !path.exists();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .with_context(|_| crate::ErrorKind::OpenRw { path: path.clone() })?;
        file.try_lock_exclusive()
            .with_context(|_| crate::ErrorKind::LockFile { path: path.clone() })?;
        if new {
            let default = miniserde::json::to_string(&T::default());
            file.write_all(default.as_ref())
                .with_context(|_| crate::ErrorKind::WriteFile { path: path.clone() })?;
        }
        Ok(Self {
            file,
            path,
            phantom: PhantomData,
        })
    }

    pub(crate) fn read(&mut self) -> crate::Result<T> {
        let mut value = "".to_owned();
        self.file
            .seek(SeekFrom::Start(0))
            .and_then(|_| self.file.read_to_string(&mut value))
            .with_context(|_| crate::ErrorKind::ReadFile {
                path: self.path.clone(),
            })?;
        serde_json::from_str(&value)
            .with_context(|_| crate::ErrorKind::ReadFile {
                path: self.path.clone(),
            })
            .map_err(Into::into)
    }

    pub(crate) fn write(&mut self, value: &T) -> crate::Result<()> {
        let value = miniserde::json::to_string(&value);
        self.file
            .seek(SeekFrom::Start(0))
            .and_then(|_| self.file.set_len(0))
            .and_then(|_| self.file.write_all(value.as_ref()))
            .with_context(|_| crate::ErrorKind::WriteFile {
                path: self.path.clone(),
            })
            .map_err(Into::into)
    }
}

impl<T: Default + miniserde::Serialize + DeserializeOwned> Drop for ExclusivelyLockedJsonFile<T> {
    fn drop(&mut self) {
        // maybe unnecessary
        let _ = self.file.unlock();
    }
}
