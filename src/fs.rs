use cargo::util::FileLock;
use failure::Fallible;
use failure::ResultExt as _;
use serde::de::DeserializeOwned;

use std::io::{Read as _, Seek as _, SeekFrom, Write as _};
use std::marker::PhantomData;
use std::path::Path;

pub(crate) fn read_src(path: &Path) -> Fallible<syn::File> {
    let src = std::fs::read_to_string(path)
        .with_context(|_| failure::err_msg(format!("Failed to read {}", path.display())))?;
    syn::parse_file(&src)
        .with_context(|_| failure::err_msg(format!("Failed to parse {}", path.display())))
        .map_err(Into::into)
}

pub(crate) struct JsonFileLock<T: Default + miniserde::Serialize + DeserializeOwned> {
    lock: FileLock,
    phantom: PhantomData<fn() -> T>,
}

impl<T: Default + miniserde::Serialize + DeserializeOwned> JsonFileLock<T> {
    pub(crate) fn read(&mut self) -> Fallible<T> {
        let mut value = "".to_owned();
        self.lock
            .seek(SeekFrom::Start(0))
            .and_then(|_| self.lock.read_to_string(&mut value))
            .with_context(|_| {
                failure::err_msg(format!("Failed to read {}", self.lock.path().display()))
            })?;
        if value.is_empty() {
            Ok(T::default())
        } else {
            serde_json::from_str(&value)
                .with_context(|_| {
                    failure::err_msg(format!("Failed to read {}", self.lock.path().display()))
                })
                .map_err(Into::into)
        }
    }

    pub(crate) fn write(&mut self, value: &T) -> Fallible<()> {
        let value = miniserde::json::to_string(&value);
        self.lock
            .seek(SeekFrom::Start(0))
            .and_then(|_| self.lock.file().set_len(0))
            .and_then(|_| self.lock.write_all(value.as_ref()))
            .with_context(|_| {
                failure::err_msg(format!("Failed to write {}", self.lock.path().display()))
            })
            .map_err(Into::into)
    }
}

impl<T: Default + miniserde::Serialize + DeserializeOwned> From<FileLock> for JsonFileLock<T> {
    fn from(lock: FileLock) -> Self {
        Self {
            lock,
            phantom: PhantomData,
        }
    }
}
