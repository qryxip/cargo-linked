use cargo::core::manifest::Target;
use cargo::core::package_id::PackageId;
use cargo::util::errors::{CargoResult, ProcessError};
use cargo::util::process_builder::ProcessBuilder;
use derive_more::Display;
use failure::ResultExt as _;
use fixedbitset::FixedBitSet;
use once_cell::sync::Lazy;
use regex::Regex;
use structopt::StructOpt;

use std::ffi::{OsStr, OsString};
use std::ops::Range;
use std::process::Output;
use std::str::{self, FromStr};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ErrorMessage {
    pub(crate) message: String,
    pub(crate) code: Option<ErrorMessageCode>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ErrorMessageCode {
    pub(crate) code: String,
}

#[derive(Debug)]
pub(crate) struct Rustc<'a> {
    cmd: ProcessBuilder,
    opts: RustcOpts,
    id: PackageId,
    target: &'a Target,
}

impl<'a> Rustc<'a> {
    pub(crate) fn new(cmd: ProcessBuilder, id: PackageId, target: &'a Target) -> CargoResult<Self> {
        let mut args = vec![cmd.get_program()];
        args.extend(cmd.get_args());
        let opts = RustcOpts::from_iter_safe(&args)
            .with_context(|_| failure::err_msg(format!("Failed to parse {:?}", args)))?;
        Ok(Self {
            cmd,
            opts,
            id,
            target,
        })
    }

    pub(crate) fn externs(&self) -> &[Extern] {
        &self.opts.r#extern
    }

    pub(crate) fn capture_error_messages(
        &mut self,
        exclude: &FixedBitSet,
        on_stdout_line: &mut dyn FnMut(&str) -> CargoResult<()>,
        on_stderr_line: &mut dyn FnMut(&str) -> CargoResult<()>,
    ) -> CargoResult<Option<Vec<ErrorMessage>>> {
        self.eprint_exclusion(exclude, on_stderr_line)?;
        self.cmd.args_replace(&self.opts.to_args(exclude, true));

        if let Err(err) = self
            .cmd
            .exec_with_streaming(on_stdout_line, &mut |_| Ok(()), true)
        {
            let output = err
                .iter_chain()
                .flat_map(|e| e.downcast_ref::<ProcessError>())
                .flat_map(|ProcessError { output, .. }| output)
                .next();
            let stderr = match output {
                None => return Err(err),
                Some(Output { stderr, .. }) => str::from_utf8(stderr)?,
            };
            stderr
                .lines()
                .map(serde_json::from_str)
                .collect::<serde_json::Result<_>>()
                .map(Some)
                .map_err(Into::into)
        } else {
            Ok(None)
        }
    }

    pub(crate) fn run(
        &mut self,
        exclude: &FixedBitSet,
        on_stdout_line: &mut dyn FnMut(&str) -> CargoResult<()>,
        on_stderr_line: &mut dyn FnMut(&str) -> CargoResult<()>,
    ) -> CargoResult<()> {
        self.eprint_exclusion(exclude, on_stderr_line)?;
        self.cmd.args_replace(&self.opts.to_args(exclude, true));

        self.cmd
            .exec_with_streaming(on_stdout_line, on_stderr_line, false)
            .map(|_| ())
    }

    fn eprint_exclusion(
        &self,
        exclude: &FixedBitSet,
        on_stderr_line: &mut dyn FnMut(&str) -> CargoResult<()>,
    ) -> CargoResult<()> {
        let width = self
            .opts
            .r#extern
            .iter()
            .map(|e| e.name().len())
            .max()
            .unwrap_or(0);

        on_stderr_line(&format!("`{}`", self.id))?;
        on_stderr_line(&format!("└─── {}", self.target))?;
        for (i, r#extern) in self.opts.r#extern.iter().enumerate() {
            let mut msg = if i < self.opts.r#extern.len() - 1 {
                format!("    ├─── {}: ", r#extern.name())
            } else {
                format!("    └─── {}: ", r#extern.name())
            };
            (0..width - r#extern.name().len()).for_each(|_| msg.push(' '));
            msg += if exclude[i] { "off" } else { "on" };
            on_stderr_line(&msg)?;
        }
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct RustcOpts {
    #[structopt(long, parse(from_os_str))]
    cfg: Vec<OsString>,
    #[structopt(short = "L", parse(from_os_str))]
    link_path: Vec<OsString>,
    #[structopt(short = "l", parse(from_os_str))]
    link_crate: Vec<OsString>,
    #[structopt(long, parse(from_os_str))]
    crate_type: Option<OsString>,
    #[structopt(long, parse(from_os_str))]
    crate_name: Option<OsString>,
    #[structopt(long, parse(from_os_str))]
    edition: Option<OsString>,
    #[structopt(long, parse(from_os_str))]
    emit: Option<OsString>,
    #[structopt(long, parse(from_os_str))]
    print: Option<OsString>,
    #[structopt(short = "g")]
    debuginfo_2: bool,
    #[structopt(short = "O")]
    opt_level_2: bool,
    #[structopt(short = "o", parse(from_os_str))]
    output: Option<OsString>,
    #[structopt(long)]
    test: bool,
    #[structopt(long, parse(from_os_str))]
    out_dir: Option<OsString>,
    #[structopt(long, parse(from_os_str))]
    explain: Vec<OsString>,
    #[structopt(long, parse(from_os_str))]
    target: Option<OsString>,
    #[structopt(short = "W", parse(from_os_str))]
    warn: Vec<OsString>,
    #[structopt(short = "A", parse(from_os_str))]
    allow: Vec<OsString>,
    #[structopt(short = "D", parse(from_os_str))]
    deny: Vec<OsString>,
    #[structopt(short = "F", parse(from_os_str))]
    forbid: Vec<OsString>,
    #[structopt(long, parse(from_os_str))]
    cap_lints: Option<OsString>,
    #[structopt(short = "C", parse(from_os_str))]
    codegen: Vec<OsString>,
    #[structopt(short = "v")]
    verbose: bool,
    #[structopt(long = "extern")]
    r#extern: Vec<Extern>,
    #[structopt(long, parse(from_os_str))]
    extern_private: Vec<OsString>,
    #[structopt(long, parse(from_os_str))]
    sysroot: Option<OsString>,
    #[structopt(long, parse(from_os_str))]
    error_format: Option<OsString>,
    #[structopt(long, parse(from_os_str))]
    color: Option<OsString>,
    #[structopt(long, parse(from_os_str))]
    remap_path_prefix: Option<OsString>,
    #[structopt(parse(from_os_str))]
    input: OsString,
}

impl RustcOpts {
    #[allow(clippy::cognitive_complexity)]
    fn to_args(&self, exclude: &FixedBitSet, error_format_json: bool) -> Vec<&OsStr> {
        let mut args = Vec::<&OsStr>::new();
        for cfg in &self.cfg {
            args.push("--cfg".as_ref());
            args.push(cfg);
        }
        for l in &self.link_path {
            args.push("-L".as_ref());
            args.push(l);
        }
        for l in &self.link_crate {
            args.push("-l".as_ref());
            args.push(l);
        }
        if let Some(crate_type) = &self.crate_type {
            args.push("--crate-type".as_ref());
            args.push(crate_type);
        }
        if let Some(crate_name) = &self.crate_name {
            args.push("--crate-name".as_ref());
            args.push(crate_name);
        }
        if let Some(edition) = &self.edition {
            args.push("--edition".as_ref());
            args.push(edition);
        }
        if let Some(emit) = &self.emit {
            args.push("--emit".as_ref());
            args.push(emit);
        }
        if let Some(print) = &self.print {
            args.push("--print".as_ref());
            args.push(print);
        }
        if self.debuginfo_2 {
            args.push("-g".as_ref());
        }
        if self.opt_level_2 {
            args.push("-O".as_ref());
        }
        if let Some(o) = &self.output {
            args.push("-o".as_ref());
            args.push(o);
        }
        if let Some(out_dir) = &self.out_dir {
            args.push("--out-dir".as_ref());
            args.push(out_dir);
        }
        for explain in &self.explain {
            args.push("--explain".as_ref());
            args.push(explain);
        }
        if self.test {
            args.push("--test".as_ref());
        }
        if let Some(target) = &self.target {
            args.push("--target".as_ref());
            args.push(target);
        }
        for warn in &self.warn {
            args.push("--warn".as_ref());
            args.push(warn);
        }
        for allow in &self.allow {
            args.push("--allow".as_ref());
            args.push(allow);
        }
        for deny in &self.deny {
            args.push("--deny".as_ref());
            args.push(deny);
        }
        for forbid in &self.forbid {
            args.push("--forbid".as_ref());
            args.push(forbid);
        }
        if let Some(cap_lints) = &self.cap_lints {
            args.push("--cap-lints".as_ref());
            args.push(cap_lints);
        }
        for codegen in &self.codegen {
            args.push("--codegen".as_ref());
            args.push(codegen);
        }
        if self.verbose {
            args.push("--verbose".as_ref());
        }
        for (i, r#extern) in self.r#extern.iter().enumerate() {
            if !exclude[i] {
                args.push("--extern".as_ref());
                args.push(r#extern.as_ref());
            }
        }
        for extern_private in &self.extern_private {
            args.push("--extern-private".as_ref());
            args.push(extern_private);
        }
        if let Some(sysroot) = &self.sysroot {
            args.push("--sysroot".as_ref());
            args.push(sysroot);
        }
        if error_format_json {
            args.push("--error-format".as_ref());
            args.push("json".as_ref());
        } else if let Some(error_format) = &self.error_format {
            args.push("--error-format".as_ref());
            args.push(error_format);
        }
        if let Some(color) = &self.color {
            args.push("--color".as_ref());
            args.push(color);
        }
        if let Some(remap_path_prefix) = &self.remap_path_prefix {
            args.push("--remap-path-prefix".as_ref());
            args.push(remap_path_prefix);
        }
        args.push(&self.input);
        args
    }
}

#[derive(Display, Debug, PartialEq, Eq, Hash)]
#[display(fmt = "{}", string)]
pub(crate) struct Extern {
    string: String,
    name: Range<usize>,
}

impl Extern {
    pub(crate) fn name(&self) -> &str {
        &self.string[self.name.clone()]
    }
}

impl FromStr for Extern {
    type Err = crate::Error;

    fn from_str(s: &str) -> crate::Result<Self> {
        static EXTERN: Lazy<Regex> = lazy_regex!(r"\A([a-zA-Z0-9_]+)=.*\z");

        let caps = EXTERN.captures(s).ok_or_else(|| {
            let (text, regex) = (s.to_owned(), EXTERN.as_str());
            crate::ErrorKind::Regex { text, regex }
        })?;
        Ok(Self {
            string: s.to_owned(),
            name: 0..caps[1].len(),
        })
    }
}

impl AsRef<OsStr> for Extern {
    fn as_ref(&self) -> &OsStr {
        self.string.as_ref()
    }
}
