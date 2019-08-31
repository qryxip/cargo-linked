use crate::ExecutableTarget;

use cargo_metadata::Metadata;
use derive_more::Display;
use failure::{Fail as _, ResultExt as _};
use fixedbitset::FixedBitSet;
use if_chain::if_chain;
use itertools::Itertools as _;
use log::info;
use once_cell::sync::Lazy;
use regex::Regex;
use structopt::StructOpt;
use tokio_process::{CommandExt as _, OutputAsync};

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::ops::{Deref, Range};
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use std::str::FromStr;
use std::{fmt, iter};

pub(crate) fn cargo_metadata(
    cargo: impl AsRef<OsStr>,
    manifest_path: Option<impl AsRef<Path>>,
    cwd: Option<impl AsRef<Path>>,
    rt: &mut tokio::runtime::current_thread::Runtime,
    ctrl_c: Option<&mut tokio_signal::IoStream<()>>,
) -> crate::Result<Metadata> {
    let mut args = vec![
        OsStr::new("metadata"),
        OsStr::new("--format-version"),
        OsStr::new("1"),
    ];
    if let Some(manifest_path) = &manifest_path {
        args.push("--manifest-path".as_ref());
        args.push(manifest_path.as_ref().as_ref());
    }

    let (_, stdout, _) = await_command::<(ExitStatusSuccess, String, ()), _, _, _, _, _, _, _>(
        cargo,
        &args,
        iter::empty::<(&'static str, &'static str)>(),
        cwd,
        rt,
        ctrl_c,
    )?;

    crate::fs::from_json(&stdout, "`cargo metadata` output")
}

pub(crate) fn cargo_build_vv(
    cargo: &Path,
    target: Option<&ExecutableTarget>,
    target_dir: &Path,
    manifest_dir: &Path,
    debug: bool,
    rt: &mut tokio::runtime::current_thread::Runtime,
    ctrl_c: Option<&mut tokio_signal::IoStream<()>>,
) -> crate::Result<String> {
    let mut args = vec![OsStr::new("build"), OsStr::new("-vv")];
    if !debug {
        args.push("--release".as_ref());
    }
    args.push("--target-dir".as_ref());
    args.push(target_dir.as_ref());
    args.push("--message-format".as_ref());
    args.push("json".as_ref());
    args.push("--color".as_ref());
    args.push("never".as_ref());
    match target {
        None => {}
        Some(ExecutableTarget::Bin(bin)) => {
            args.extend_from_slice(&["--bin".as_ref(), bin.as_ref()]);
        }
        Some(ExecutableTarget::Example(example)) => {
            args.extend_from_slice(&["--example".as_ref(), example.as_ref()]);
        }
        Some(ExecutableTarget::Test(test)) => {
            args.extend_from_slice(&["--test".as_ref(), test.as_ref()]);
        }
        Some(ExecutableTarget::Bench(bench)) => {
            args.extend_from_slice(&["--bench".as_ref(), bench.as_ref()]);
        }
    }

    let (ExitStatusSuccess, (), stderr) = await_command(
        cargo,
        &args,
        iter::empty::<(&'static str, &'static str)>(),
        Some(manifest_dir),
        rt,
        ctrl_c,
    )?;
    Ok(stderr)
}

#[derive(Debug)]
pub(crate) struct Rustc {
    arg0: OsString,
    opts: RustcOpts,
    envs: BTreeMap<String, String>,
    workspace_root: PathBuf,
}

impl Rustc {
    pub(crate) fn new(
        arg0: &OsStr,
        opts: RustcOpts,
        envs: BTreeMap<String, String>,
        workspace_root: &Path,
    ) -> Self {
        Self {
            arg0: arg0.to_owned(),
            opts,
            envs,
            workspace_root: workspace_root.to_owned(),
        }
    }

    pub(crate) fn externs(&self) -> &[Extern] {
        &self.opts.r#extern
    }

    pub(crate) fn input_abs(&self) -> PathBuf {
        if Path::new(&self.opts.input).is_absolute() {
            self.opts.input.clone().into()
        } else {
            self.workspace_root.join(&self.opts.input)
        }
    }

    pub(crate) fn run(
        &self,
        exclude: &FixedBitSet,
        rt: &mut tokio::runtime::current_thread::Runtime,
        ctrl_c: Option<&mut tokio_signal::IoStream<()>>,
    ) -> crate::Result<(bool, String)> {
        await_command(
            &self.arg0,
            &self.opts.to_args(&exclude),
            &self.envs,
            Some(&self.workspace_root),
            rt,
            ctrl_c,
        )
        .map(|(success, (), stderr)| (success, stderr))
    }
}

#[derive(Debug, StructOpt)]
pub(crate) struct RustcOpts {
    #[structopt(long = "cfg")]
    cfg: Vec<String>,
    #[structopt(short = "L")]
    link_path: Vec<String>,
    #[structopt(short = "l")]
    link_crate: Vec<String>,
    #[structopt(long = "crate-type")]
    crate_type: Option<String>,
    #[structopt(long = "crate-name")]
    crate_name: Option<String>,
    #[structopt(long = "edition")]
    edition: Option<String>,
    #[structopt(long = "emit")]
    emit: Option<String>,
    #[structopt(long = "print")]
    print: Option<String>,
    #[structopt(short = "g")]
    debuginfo_2: bool,
    #[structopt(short = "O")]
    opt_level_2: bool,
    #[structopt(short = "o")]
    output: Option<String>,
    #[structopt(long = "test")]
    test: bool,
    #[structopt(long = "out-dir")]
    out_dir: Option<String>,
    #[structopt(long = "explain")]
    explain: Vec<String>,
    #[structopt(long = "target")]
    target: Option<String>,
    #[structopt(short = "W", long = "warn")]
    warn: Vec<String>,
    #[structopt(short = "A", long = "allow")]
    allow: Vec<String>,
    #[structopt(short = "D", long = "deny")]
    deny: Vec<String>,
    #[structopt(short = "F", long = "forbid")]
    forbid: Vec<String>,
    #[structopt(long = "cap-lints")]
    cap_lints: Option<String>,
    #[structopt(short = "C", long = "codegen")]
    codegen: Vec<String>,
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
    #[structopt(long = "extern")]
    r#extern: Vec<Extern>,
    #[structopt(long = "extern-private")]
    extern_private: Vec<String>,
    #[structopt(long = "sysroot")]
    sysroot: Option<String>,
    #[structopt(long = "error-format")]
    error_format: Option<String>,
    #[structopt(long = "color")]
    color: Option<String>,
    #[structopt(long = "remap-path-prefix")]
    remap_path_prefix: Option<String>,
    input: String,
}

impl RustcOpts {
    #[allow(clippy::cognitive_complexity)]
    pub(crate) fn to_args(&self, exclude: &FixedBitSet) -> Vec<&str> {
        let mut args = vec![];
        for cfg in &self.cfg {
            args.push("--cfg");
            args.push(cfg);
        }
        for l in &self.link_path {
            args.push("-L");
            args.push(l);
        }
        for l in &self.link_crate {
            args.push("-l");
            args.push(l);
        }
        if let Some(crate_type) = &self.crate_type {
            args.push("--crate-type");
            args.push(crate_type);
        }
        if let Some(crate_name) = &self.crate_name {
            args.push("--crate-name");
            args.push(crate_name);
        }
        if let Some(edition) = &self.edition {
            args.push("--edition");
            args.push(edition);
        }
        if let Some(emit) = &self.emit {
            args.push("--emit");
            args.push(emit);
        }
        if let Some(print) = &self.print {
            args.push("--print");
            args.push(print);
        }
        if self.debuginfo_2 {
            args.push("-g");
        }
        if self.opt_level_2 {
            args.push("-O");
        }
        if let Some(o) = &self.output {
            args.push("-o");
            args.push(o);
        }
        if let Some(out_dir) = &self.out_dir {
            args.push("--out-dir");
            args.push(out_dir);
        }
        for explain in &self.explain {
            args.push("--explain");
            args.push(explain);
        }
        if self.test {
            args.push("--test");
        }
        if let Some(target) = &self.target {
            args.push("--target");
            args.push(target);
        }
        for warn in &self.warn {
            args.push("--warn");
            args.push(warn);
        }
        for allow in &self.allow {
            args.push("--allow");
            args.push(allow);
        }
        for deny in &self.deny {
            args.push("--deny");
            args.push(deny);
        }
        for forbid in &self.forbid {
            args.push("--forbid");
            args.push(forbid);
        }
        if let Some(cap_lints) = &self.cap_lints {
            args.push("--cap-lints");
            args.push(cap_lints);
        }
        for codegen in &self.codegen {
            args.push("--codegen");
            args.push(codegen);
        }
        if self.verbose {
            args.push("--verbose");
        }
        for (i, r#extern) in self.r#extern.iter().enumerate() {
            if !exclude[i] {
                args.push("--extern");
                args.push(r#extern.deref());
            }
        }
        for extern_private in &self.extern_private {
            args.push("--extern-private");
            args.push(extern_private);
        }
        if let Some(sysroot) = &self.sysroot {
            args.push("--sysroot");
            args.push(sysroot);
        }
        if let Some(error_format) = &self.error_format {
            args.push("--error-format");
            args.push(error_format);
        }
        if let Some(color) = &self.color {
            args.push("--color");
            args.push(color);
        }
        if let Some(remap_path_prefix) = &self.remap_path_prefix {
            args.push("--remap-path-prefix");
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

impl Deref for Extern {
    type Target = str;

    fn deref(&self) -> &str {
        &self.string
    }
}

fn await_command<
    O: ProcessedOutput,
    S1: AsRef<OsStr>,
    S2: AsRef<OsStr>,
    A: Clone + IntoIterator<Item = S2>,
    K: AsRef<str> + AsRef<OsStr>,
    V: AsRef<str> + AsRef<OsStr>,
    E: Clone + IntoIterator<Item = (K, V)>,
    P: AsRef<Path>,
>(
    arg0: S1,
    args: A,
    envs: E,
    cwd: Option<P>,
    rt: &mut tokio::runtime::current_thread::Runtime,
    ctrl_c: Option<&mut tokio_signal::IoStream<()>>,
) -> crate::Result<O> {
    struct OutputWithCtrlC<'a, 'b> {
        arg0: &'a OsStr,
        output: OutputAsync,
        ctrl_c: Option<&'b mut tokio_signal::IoStream<()>>,
    }

    impl futures01::Future for OutputWithCtrlC<'_, '_> {
        type Item = std::process::Output;
        type Error = crate::Error;

        fn poll(&mut self) -> futures01::Poll<std::process::Output, crate::Error> {
            if_chain! {
                if let Some(ctrl_c) = &mut self.ctrl_c;
                let ctrl_c = ctrl_c
                    .poll()
                    .map_err(|e| e.context(crate::ErrorKind::Tokio))?;
                if ctrl_c.is_ready();
                then {
                    Err(crate::ErrorKind::CtrlC.into())
                } else {
                    match self.output.poll() {
                        Ok(futures01::Async::NotReady) => Ok(futures01::Async::NotReady),
                        Ok(futures01::Async::Ready(output)) => Ok(futures01::Async::Ready(output)),
                        Err(err) => {
                            let arg0 = self.arg0.to_owned();
                            Err(err.context(crate::ErrorKind::Command{ arg0 }).into())
                        }
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    fn fmt_env(
        key: &str,
        val: &str,
        f: &mut dyn FnMut(&dyn fmt::Display) -> fmt::Result,
    ) -> fmt::Result {
        let val = shell_escape::escape(val.into());
        f(&format_args!("set {}={}&& ", key, val))
    }

    #[cfg(unix)]
    fn fmt_env(
        key: &str,
        val: &str,
        f: &mut dyn FnMut(&dyn fmt::Display) -> fmt::Result,
    ) -> fmt::Result {
        let val = shell_escape::escape(val.into());
        f(&format_args!("{}={} ", key, val))
    }

    fn null_or_piped(ignore: bool) -> Stdio {
        if ignore {
            Stdio::null()
        } else {
            Stdio::piped()
        }
    }

    let arg0 = arg0.as_ref();
    let cwd = cwd
        .map(|cwd| Ok(cwd.as_ref().to_owned()))
        .unwrap_or_else(crate::fs::current_dir)?;

    info!(
        "`{}{}{}` in {}",
        envs.clone()
            .into_iter()
            .format_with("", |(k, v), f| fmt_env(k.as_ref(), v.as_ref(), f)),
        arg0.to_string_lossy(),
        args.clone()
            .into_iter()
            .format_with("", |s, f| f(&format_args!(
                " {}",
                s.as_ref().to_string_lossy(),
            ))),
        cwd.display(),
    );

    let output = std::process::Command::new(arg0)
        .args(args)
        .envs(envs)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(null_or_piped(O::IGNORE_STDOUT))
        .stderr(null_or_piped(O::IGNORE_STDERR))
        .output_async();

    let output = rt.block_on(OutputWithCtrlC {
        arg0,
        output,
        ctrl_c,
    })?;

    info!("{}", output.status);

    O::process(arg0, output)
}

struct ExitStatusSuccess;

trait ProcessedOutput: Sized {
    const IGNORE_STDOUT: bool;
    const IGNORE_STDERR: bool;

    fn process(arg0: &OsStr, output: Output) -> crate::Result<Self>;
}

impl ProcessedOutput for (ExitStatusSuccess, (), ()) {
    const IGNORE_STDOUT: bool = true;
    const IGNORE_STDERR: bool = true;

    fn process(arg0: &OsStr, output: Output) -> crate::Result<Self> {
        if output.status.success() {
            Ok((ExitStatusSuccess, (), ()))
        } else {
            let arg0 = arg0.to_owned();
            Err(failure::err_msg(output.status)
                .context(crate::ErrorKind::Command { arg0 })
                .into())
        }
    }
}

impl ProcessedOutput for (ExitStatusSuccess, String, ()) {
    const IGNORE_STDOUT: bool = false;
    const IGNORE_STDERR: bool = true;

    fn process(arg0: &OsStr, output: Output) -> crate::Result<Self> {
        if output.status.success() {
            let stdout = String::from_utf8(output.stdout).with_context(|_| {
                let arg0_filename = Path::new(arg0).file_name().unwrap_or_default().to_owned();
                crate::ErrorKind::NonUtf8Output { arg0_filename }
            })?;
            Ok((ExitStatusSuccess, stdout, ()))
        } else {
            let arg0 = arg0.to_owned();
            Err(failure::err_msg(output.status)
                .context(crate::ErrorKind::Command { arg0 })
                .into())
        }
    }
}

impl ProcessedOutput for (ExitStatusSuccess, (), String) {
    const IGNORE_STDOUT: bool = true;
    const IGNORE_STDERR: bool = false;

    fn process(arg0: &OsStr, output: Output) -> crate::Result<Self> {
        if output.status.success() {
            let stderr = String::from_utf8(output.stderr).with_context(|_| {
                let arg0_filename = Path::new(arg0).file_name().unwrap_or_default().to_owned();
                crate::ErrorKind::NonUtf8Output { arg0_filename }
            })?;
            Ok((ExitStatusSuccess, (), stderr))
        } else {
            let arg0 = arg0.to_owned();
            Err(failure::err_msg(output.status)
                .context(crate::ErrorKind::Command { arg0 })
                .into())
        }
    }
}

impl ProcessedOutput for (bool, (), ()) {
    const IGNORE_STDOUT: bool = true;
    const IGNORE_STDERR: bool = true;

    fn process(_: &OsStr, output: Output) -> crate::Result<Self> {
        Ok((output.status.success(), (), ()))
    }
}

impl ProcessedOutput for (bool, (), String) {
    const IGNORE_STDOUT: bool = true;
    const IGNORE_STDERR: bool = false;

    fn process(arg0: &OsStr, output: Output) -> crate::Result<Self> {
        let stderr = String::from_utf8(output.stderr).with_context(|_| {
            let arg0_filename = Path::new(arg0).file_name().unwrap_or_default().to_owned();
            crate::ErrorKind::NonUtf8Output { arg0_filename }
        })?;
        Ok((output.status.success(), (), stderr))
    }
}
