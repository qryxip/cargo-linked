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
    #[display(fmt = "Maybe invalid metadata")]
    MaybeInvalidMetadata,
    #[display(fmt = "Root package not found")]
    RootNotFound,
    #[display(fmt = "`.resolve` is not present")]
    ResolveNotPresent,
    #[display(fmt = "Could not determine which binary to run")]
    AmbiguousTarget,
    #[display(fmt = "No such `{}`: {:?}", kind, name)]
    NoSuchTarget { kind: &'static str, name: String },
    #[display(fmt = "$CARGO is not present")]
    CargoEnvVarNotPresent,
    #[display(fmt = "Failed to getcwd")]
    Getcwd,
    #[display(fmt = "Interrupted")]
    CtrlC,
    #[display(fmt = "tokio error")]
    Tokio,
    #[display(fmt = "`{}` failed", "arg0.to_string_lossy()")]
    Command { arg0: OsString },
    #[display(
        fmt = "`{}` produced non UTF-8 output",
        "arg0_filename.to_string_lossy()"
    )]
    NonUtf8Output { arg0_filename: OsString },
    #[display(fmt = "Failed to read {}", "path.display()")]
    ReadFile { path: PathBuf },
    #[display(fmt = "Failed to write {}", "path.display()")]
    WriteFile { path: PathBuf },
    #[display(fmt = "Failed to open {}", "path.display()")]
    OpenRw { path: PathBuf },
    #[display(fmt = "Failed to lock {}", "path.display()")]
    LockFile { path: PathBuf },
    #[display(fmt = "Failed to copy {} to {}", "from.display()", "to.display()")]
    CopyDir { from: PathBuf, to: PathBuf },
    #[display(fmt = "Failed to move {} to {}", "from.display()", "to.display()")]
    MoveDir { from: PathBuf, to: PathBuf },
    #[display(fmt = "Failed to remove {}", "dir.display()")]
    RemoveDir { dir: PathBuf },
    #[display(fmt = "Failed to deserialize {}", what)]
    Deserialize { what: String },
    #[display(fmt = "{:?} does not match {:?}", text, regex)]
    Regex { text: String, regex: &'static str },
    #[display(fmt = "Failed to parse\n===STDERR===\n{}============", stderr)]
    ParseCargoBuildVvStderr { stderr: String },
    #[display(fmt = "Failed to parse {:?}", args)]
    ParseRustcOptions { args: Vec<String> },
    #[display(
        fmt = "Missing rustc options for {:?}. Touch the file or remove {:?}",
        src_path,
        target_dir_with_mode
    )]
    MissingRustcOptions {
        src_path: PathBuf,
        target_dir_with_mode: PathBuf,
    },
}
