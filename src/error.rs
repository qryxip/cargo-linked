use derive_more::{Display, From};
use failure::{Backtrace, Fail};

use std::ffi::OsString;
use std::path::PathBuf;

/// Error.
#[derive(From, Display, Debug)]
#[display(fmt = "{}", _0)]
pub struct Error(failure::Context<ErrorKind>);

impl Error {
    /// Gets the error kind.
    pub fn kind(&self) -> &ErrorKind {
        self.0.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self(kind.into())
    }
}

impl Fail for Error {
    fn name(&self) -> Option<&str> {
        self.0.name()
    }

    fn cause(&self) -> Option<&dyn Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }
}

/// Error kind.
#[derive(Display, Debug, Fail)]
pub enum ErrorKind {
    #[display(fmt = "Could not determine which binary to run")]
    AmbiguousTarget,
    #[display(
        fmt = "No such `{}`{}",
        kind,
        r#"name.as_ref().map(|s| format!(": {}", s)).unwrap_or_default()"#
    )]
    NoSuchTarget {
        kind: &'static str,
        name: Option<String>,
    },
    #[display(fmt = "Failed to parse {:?}", args)]
    ParseRustcOptions { args: Vec<OsString> },
    #[display(fmt = "Cargo error")]
    Cargo,
    #[display(fmt = "{:?} does not match {:?}", text, regex)]
    Regex { text: String, regex: &'static str },
    #[display(fmt = "Failed to read {}", "path.display()")]
    ReadFile { path: PathBuf },
    #[display(fmt = "Failed to write {}", "path.display()")]
    WriteFile { path: PathBuf },
}
