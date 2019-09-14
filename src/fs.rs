use cargo::util::FileLock;
use failure::ResultExt as _;
use serde::de::DeserializeOwned;

use std::io::{Read as _, Seek as _, SeekFrom, Write as _};
use std::marker::PhantomData;
use std::path::Path;

pub(crate) struct JsonFileLock<T: Default + miniserde::Serialize + DeserializeOwned> {
    lock: FileLock,
    phantom: PhantomData<fn() -> T>,
}

impl<T: Default + miniserde::Serialize + DeserializeOwned> JsonFileLock<T> {
    pub(crate) fn path(&self) -> &Path {
        self.lock.path()
    }

    pub(crate) fn read(&mut self) -> crate::Result<T> {
        let mut value = "".to_owned();
        self.lock
            .seek(SeekFrom::Start(0))
            .and_then(|_| self.lock.read_to_string(&mut value))
            .with_context(|_| crate::ErrorKind::ReadFile {
                path: self.lock.path().to_owned(),
            })?;
        if value.is_empty() {
            Ok(T::default())
        } else {
            serde_json::from_str(&value)
                .with_context(|_| crate::ErrorKind::ReadFile {
                    path: self.lock.path().to_owned(),
                })
                .map_err(Into::into)
        }
    }

    pub(crate) fn write(&mut self, value: &T) -> crate::Result<()> {
        let value = miniserde::json::to_string(&value);
        self.lock
            .seek(SeekFrom::Start(0))
            .and_then(|_| self.lock.file().set_len(0))
            .and_then(|_| self.lock.write_all(value.as_ref()))
            .with_context(|_| crate::ErrorKind::WriteFile {
                path: self.lock.path().to_owned(),
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
