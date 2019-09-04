use crate::path::{Utf8Path, Utf8PathBuf};
use crate::ExecutableTarget;

use cargo_metadata::Metadata;
use derive_more::Display;
use failure::{Fail as _, ResultExt as _};
use fixedbitset::FixedBitSet;
use futures01::Stream as _;
use if_chain::if_chain;
use itertools::Itertools as _;
use log::info;
use maplit::btreemap;
use once_cell::sync::Lazy;
use regex::Regex;
use structopt::StructOpt;
use tokio::io::AsyncRead as _;
use tokio_process::{CommandExt as _, OutputAsync};

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::ops::{Deref, Range};
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Output, Stdio};
use std::str::FromStr;
use std::{fmt, mem, str, thread};

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

    let (_, stdout, _) = ProcessBuilder::new(cargo)
        .args(args)
        .cwd(cwd)
        .wait_with_output::<(ExitStatusSuccess, String, ())>(rt, ctrl_c)?;

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

    ProcessBuilder::new(cargo)
        .args(args)
        .cwd(Some(manifest_dir))
        .wait_stderr_broadcasting(rt, ctrl_c)
}

#[derive(Debug)]
pub(crate) struct Rustc {
    arg0: OsString,
    opts: RustcOpts,
    envs: BTreeMap<String, String>,
    workspace_root: Utf8PathBuf,
}

impl Rustc {
    pub(crate) fn new(
        arg0: &OsStr,
        opts: RustcOpts,
        envs: BTreeMap<String, String>,
        workspace_root: impl AsRef<Utf8Path>,
    ) -> Self {
        Self {
            arg0: arg0.to_owned(),
            opts,
            envs,
            workspace_root: workspace_root.as_ref().to_owned(),
        }
    }

    pub(crate) fn externs(&self) -> &[Extern] {
        &self.opts.r#extern
    }

    pub(crate) fn input_abs(&self) -> Utf8PathBuf {
        if self.opts.input.is_absolute() {
            self.opts.input.clone()
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
        ProcessBuilder::new(&self.arg0)
            .args(&self.opts.to_args(&exclude))
            .envs(&self.envs)
            .cwd(Some(&self.workspace_root))
            .wait_with_output::<(bool, (), String)>(rt, ctrl_c)
            .map(|(success, _, stderr)| (success, stderr))
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
    input: Utf8PathBuf,
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
        args.push(self.input.as_ref());
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

struct ProcessBuilder {
    arg0: OsString,
    args: Vec<OsString>,
    envs: BTreeMap<String, String>,
    cwd: Option<PathBuf>,
}

impl ProcessBuilder {
    fn new(arg0: impl AsRef<OsStr>) -> Self {
        Self {
            arg0: arg0.as_ref().to_owned(),
            args: vec![],
            envs: btreemap!(),
            cwd: None,
        }
    }

    fn args<S: AsRef<OsStr>, I: IntoIterator<Item = S>>(mut self, args: I) -> Self {
        let args = args.into_iter().map(|s| s.as_ref().to_owned());
        self.args.extend(args);
        self
    }

    fn envs<K: AsRef<str>, V: AsRef<str>, I: IntoIterator<Item = (K, V)>>(
        mut self,
        envs: I,
    ) -> Self {
        let envs = envs
            .into_iter()
            .map(|(k, v)| (k.as_ref().to_owned(), v.as_ref().to_owned()));
        self.envs.extend(envs);
        self
    }

    fn cwd(self, cwd: Option<impl AsRef<Path>>) -> Self {
        Self {
            cwd: cwd.map(|p| p.as_ref().to_owned()),
            ..self
        }
    }

    fn wait_with_output<O: ProcessedOutput>(
        self,
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
                    let ctrl_c = ctrl_c.poll().with_context(|_| crate::ErrorKind::Tokio)?;
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

        let output = self
            .command(O::IGNORE_STDOUT, O::IGNORE_STDERR)?
            .output_async();
        let output = rt.block_on(OutputWithCtrlC {
            arg0: &self.arg0,
            output,
            ctrl_c,
        })?;

        info!("{}", output.status);

        O::process(&self.arg0, output)
    }

    fn wait_stderr_broadcasting(
        self,
        rt: &mut tokio::runtime::current_thread::Runtime,
        ctrl_c: Option<&mut tokio_signal::IoStream<()>>,
    ) -> crate::Result<String> {
        struct Wait<'a, 'b> {
            arg0: &'a OsStr,
            child: tokio_process::Child,
            child_stderr: tokio_process::ChildStderr,
            stderr_buf: Vec<u8>,
            stderr_pos: usize,
            lines_tx: futures01::sync::mpsc::UnboundedSender<String>,
            ctrl_c: Option<&'b mut tokio_signal::IoStream<()>>,
        }

        impl<'a, 'b> futures01::Future for Wait<'a, 'b> {
            type Item = (ExitStatus, Vec<u8>);
            type Error = crate::Error;

            fn poll(&mut self) -> futures01::Poll<(ExitStatus, Vec<u8>), crate::Error> {
                if let Some(ctrl_c) = &mut self.ctrl_c {
                    let ctrl_c = ctrl_c.poll().with_context(|_| crate::ErrorKind::Tokio)?;
                    if ctrl_c.is_ready() {
                        return Err(crate::ErrorKind::CtrlC.into());
                    }
                }

                let _ = futures01::try_ready!(self
                    .child_stderr
                    .read_buf(&mut self.stderr_buf)
                    .with_context(|_| crate::ErrorKind::Command {
                        arg0: self.arg0.to_owned()
                    }));

                if let Some(lf_pos) =
                    (self.stderr_pos..self.stderr_buf.len()).find(|&i| self.stderr_buf[i] == b'\n')
                {
                    let line = str::from_utf8(&self.stderr_buf[self.stderr_pos..lf_pos])
                        .unwrap_or_else(|e| unimplemented!("{:?}", e))
                        .to_owned();
                    let _ = self.lines_tx.unbounded_send(line);
                    self.stderr_pos = lf_pos + 1;
                }

                let status = futures01::try_ready!(self.child.poll().with_context(|_| {
                    crate::ErrorKind::Command {
                        arg0: self.arg0.to_owned(),
                    }
                }));
                let stderr = mem::replace(&mut self.stderr_buf, vec![]);
                Ok(futures01::Async::Ready((status, stderr)))
            }
        }

        let (lines_tx, lines_rx) = futures01::sync::mpsc::unbounded();

        let handle = thread::Builder::new()
            .name("cargo-unused-process-wait-stderr-broadcasting".to_owned())
            .spawn(move || {
                info!("==========STDERR==========");
                if let Ok(mut rt) = tokio::runtime::current_thread::Runtime::new() {
                    let _ = rt.block_on(lines_rx.for_each(|line| {
                        info!("{}", line);
                        Ok(())
                    }));
                }
                info!("==========================");
            });

        let mut child = self
            .command(true, false)?
            .spawn_async()
            .unwrap_or_else(|e| unimplemented!("{:?}", e));

        let (status, stderr) = rt.block_on(Wait {
            arg0: &self.arg0,
            child_stderr: child.stderr().take().unwrap(),
            child,
            stderr_buf: vec![],
            stderr_pos: 0,
            lines_tx,
            ctrl_c,
        })?;

        if let Ok(handle) = handle {
            let _ = handle.join();
        }

        info!("{}", status);

        if status.success() {
            String::from_utf8(stderr)
                .with_context(|_| {
                    let arg0_filename = Path::new(&self.arg0)
                        .file_name()
                        .unwrap_or_default()
                        .to_owned();
                    crate::ErrorKind::NonUtf8Output { arg0_filename }
                })
                .map_err(Into::into)
        } else {
            let arg0 = self.arg0.clone();
            Err(failure::err_msg(status)
                .context(crate::ErrorKind::Command { arg0 })
                .into())
        }
    }

    fn command(
        &self,
        ignore_stdout: bool,
        ignore_stderr: bool,
    ) -> crate::Result<std::process::Command> {
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

        let cwd = self
            .cwd
            .clone()
            .map(Ok)
            .unwrap_or_else(crate::fs::current_dir)?;

        info!(
            "`{}{}{}` in {}",
            self.envs
                .iter()
                .format_with("", |(k, v), f| fmt_env(k, v, f)),
            self.arg0.to_string_lossy(),
            self.args
                .iter()
                .format_with("", |s, f| f(&format_args!(" {}", s.to_string_lossy()))),
            cwd.display(),
        );

        let mut cmd = std::process::Command::new(&self.arg0);
        cmd.args(&self.args)
            .envs(&self.envs)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(null_or_piped(ignore_stdout))
            .stderr(null_or_piped(ignore_stderr));
        Ok(cmd)
    }
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
