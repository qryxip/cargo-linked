use derive_more::{Display, From};
use failure::{Backtrace, Fail};
use structopt::StructOpt;
use strum_macros::{EnumString, IntoStaticStr};

use std::env;

/// Result.
pub type Result<T> = std::result::Result<T, crate::Error>;

/// Error.
#[derive(From, Display, Debug)]
#[display(fmt = "{}", _0)]
pub struct Error(failure::Context<crate::ErrorKind>);

impl Error {
    /// Gets the error kind.
    pub fn kind(&self) -> &crate::ErrorKind {
        self.0.get_context()
    }
}

impl From<crate::ErrorKind> for Error {
    fn from(kind: crate::ErrorKind) -> Self {
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
pub enum ErrorKind {}

/// Options.
#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
pub enum Opt {
    #[structopt(name = "unused")]
    Unused(OptUnused),
}

/// Options.
#[derive(Debug, StructOpt)]
pub struct OptUnused {
    #[structopt(
        long = "color",
        value_name = "WHEN",
        help = "Coloring",
        raw(
            default_value = "<&str>::from(ColorChoice::default())",
            possible_values = "&ColorChoice::variants()"
        )
    )]
    pub color: ColorChoice,
}

/// Coloring.
#[derive(Debug, Clone, Copy, EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab_case")]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl Default for ColorChoice {
    fn default() -> Self {
        ColorChoice::Auto
    }
}

impl ColorChoice {
    /// Variants.
    pub fn variants() -> [&'static str; 3] {
        ["auto", "always", "never"]
    }

    /// Whether to color output.
    pub fn should_color(self, stream: atty::Stream) -> bool {
        #[cfg(windows)]
        static BLACKLIST: &[&str] = &["cygwin", "dumb"];

        #[cfg(not(windows))]
        static BLACKLIST: &[&str] = &["dumb"];

        match self {
            ColorChoice::Auto => {
                atty::is(stream)
                    && env::var("TERM")
                        .ok()
                        .map_or(false, |v| !BLACKLIST.contains(&v.as_ref()))
            }
            ColorChoice::Always => true,
            ColorChoice::Never => false,
        }
    }
}

/// Context.
pub struct App {}

impl App {
    pub fn try_new() -> crate::Result<Self> {
        Ok(Self {})
    }

    pub fn run(&self, _: &OptUnused) -> crate::Result<String> {
        unimplemented!()
    }
}
