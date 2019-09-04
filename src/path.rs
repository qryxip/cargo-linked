use miniserde::ser::Fragment;

use std::borrow::Borrow;
use std::convert::Infallible;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct Utf8Path(str);

impl Utf8Path {
    pub(crate) fn new(s: &str) -> &Self {
        unsafe { &*(s as *const str as *const Self) }
    }

    pub(crate) fn is_absolute(&self) -> bool {
        Path::new(&self.0).is_absolute()
    }

    pub(crate) fn join(&self, path: impl AsRef<Self>) -> Utf8PathBuf {
        let inner = Path::new(&self.0)
            .join(path.as_ref())
            .into_os_string()
            .into_string()
            .expect("<utf-8 path><utf-8 separator><utf-8 path> should be UTF-8");
        Utf8PathBuf(inner)
    }
}

impl ToOwned for Utf8Path {
    type Owned = Utf8PathBuf;

    fn to_owned(&self) -> Utf8PathBuf {
        Utf8PathBuf(self.0.to_owned())
    }
}

impl AsRef<Path> for Utf8Path {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<Utf8Path> for str {
    fn as_ref(&self) -> &Utf8Path {
        Utf8Path::new(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize)]
#[serde(transparent)]
pub(crate) struct Utf8PathBuf(String);

impl Deref for Utf8PathBuf {
    type Target = Utf8Path;

    fn deref(&self) -> &Utf8Path {
        Utf8Path::new(&self.0)
    }
}

impl Borrow<Utf8Path> for Utf8PathBuf {
    fn borrow(&self) -> &Utf8Path {
        self
    }
}

impl AsRef<Utf8Path> for Utf8PathBuf {
    fn as_ref(&self) -> &Utf8Path {
        self
    }
}

impl AsRef<Path> for Utf8PathBuf {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<str> for Utf8PathBuf {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<Utf8PathBuf> for PathBuf {
    fn from(path: Utf8PathBuf) -> Self {
        path.0.into()
    }
}

impl From<String> for Utf8PathBuf {
    fn from(string: String) -> Self {
        Self(string)
    }
}

impl FromStr for Utf8PathBuf {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Infallible> {
        Ok(Self(s.to_owned()))
    }
}

impl Display for Utf8PathBuf {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        self.0.fmt(fmt)
    }
}

impl miniserde::Serialize for Utf8PathBuf {
    fn begin(&self) -> Fragment {
        Fragment::Str((&self.0).into())
    }
}
