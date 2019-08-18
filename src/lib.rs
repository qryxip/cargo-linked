macro_rules! lazy_regex {
    ($regex:expr $(,)?) => {
        ::once_cell::sync::Lazy::new(|| ::regex::Regex::new($regex).unwrap())
    };
}

pub mod re_exports {
    //! Re-exports.
    pub use {cargo_metadata, indexmap};
}

mod error {
    use cargo_metadata::PackageId;
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
        #[display(fmt = "`{}` failed", "arg0.to_string_lossy()")]
        Command { arg0: OsString },
        #[display(
            fmt = "`{}` produced non UTF-8 output",
            "arg0_filename.to_string_lossy()"
        )]
        NonUtf8Output { arg0_filename: OsString },
        #[display(fmt = "Failed to read {}", "path.display()")]
        ReadFile { path: PathBuf },
        #[display(fmt = "Failed to deserialize {}", what)]
        Deserialize { what: &'static str },
        #[display(fmt = "{:?} does not match {:?}", text, regex)]
        Regex { text: String, regex: &'static str },
        #[display(fmt = "Failed to parse {:?} to a SPEC", "id.repr")]
        ParsePackageIdToSpec { id: PackageId },
        #[display(fmt = "Failed to parse\n===STDERR===\n{}============", stderr)]
        ParseCargoBuildVvStderr { stderr: String },
        #[display(fmt = "Failed to parse {:?}", args)]
        ParseRustcOptions { args: Vec<String> },
        #[display(fmt = "Unexpected `src_path`: {}", "src_path.display()")]
        UnexpectedSrcPath { src_path: PathBuf },
    }
}

mod parse {
    use crate::process::RustcOpts;

    use cargo_metadata::PackageId;
    use either::Either;
    use failure::ResultExt as _;
    use maplit::btreemap;
    use once_cell::sync::Lazy;
    use regex::Regex;
    use structopt::StructOpt as _;
    use url::Url;

    use std::collections::{BTreeMap, HashMap};
    use std::path::PathBuf;
    use std::{iter, mem};

    type BySrcPath<'a> = HashMap<PathBuf, (BTreeMap<&'a str, String>, RustcOpts)>;

    pub(crate) fn package_id_to_sepc(id: &PackageId) -> crate::Result<Url> {
        static ID: Lazy<Regex> =
            lazy_regex!(r"\A([a-zA-Z0-9_-]+) ([a-zA-Z0-9\._-]+) \([a-z]+\+([^)]+)\)\z");

        let caps = ID
            .captures(&id.repr)
            .ok_or_else(|| {
                let (text, regex) = (id.repr.clone(), ID.as_str());
                crate::ErrorKind::Regex { text, regex }
            })
            .with_context(|_| crate::ErrorKind::ParsePackageIdToSpec { id: id.clone() })?;

        let mut spec = caps[3]
            .parse::<Url>()
            .with_context(|_| crate::ErrorKind::ParsePackageIdToSpec { id: id.clone() })?;
        spec.set_query(None);
        spec.set_fragment(Some(&format!("{}:{}", &caps[1], &caps[2])));
        Ok(spec)
    }

    pub(crate) fn cargo_build_vv_stderr_to_opts_and_envs<'a>(
        stderr: &'a str,
    ) -> crate::Result<BySrcPath<'a>> {
        // https://github.com/rust-lang/cargo/blob/5218d04b3160c62b99f3decbcda77f73d321bf58/src/cargo/util/process_builder.rs#L34-L59
        // https://github.com/sfackler/shell-escape/blob/81621d00297d89c98fb4d5ceb55ad3cd7c1fa69c/src/lib.rs

        use combine::char::{char, string};
        use combine::easy::{self, Info};
        use combine::parser::choice::or;
        use combine::parser::range::recognize;
        use combine::stream::state::{SourcePosition, State};
        use combine::{choice, eof, many, none_of, satisfy, skip_many, skip_many1, Parser};

        type Input<'a> = easy::Stream<State<&'a str, SourcePosition>>;

        #[cfg(windows)]
        fn maybe_escaped<'a>() -> impl Parser<Input = Input<'a>, Output = String> {
            use combine::parser;

            many(or(
                char('"')
                    .with(parser(|input| {
                        let mut acc = "".to_owned();
                        let mut num_backslashes = 0;
                        skip_many(satisfy(|c| match c {
                            '\\' => {
                                num_backslashes += 1;
                                true
                            }
                            '"' if num_backslashes % 2 == 1 => {
                                let num_backslashes = mem::replace(&mut num_backslashes, 0);
                                (0..num_backslashes / 2).for_each(|_| acc.push('\\'));
                                acc.push('"');
                                true
                            }
                            '"' => {
                                (0..num_backslashes / 2).for_each(|_| acc.push('\\'));
                                false
                            }
                            c => {
                                let num_backslashes = mem::replace(&mut num_backslashes, 0);
                                (0..num_backslashes).for_each(|_| acc.push('\\'));
                                acc.push(c);
                                true
                            }
                        }))
                        .parse_stream(input)
                        .map(|((), consumed)| (acc, consumed))
                    }))
                    .skip(char('"')),
                recognize(skip_many1(satisfy(|c| match c {
                    '"' | '\t' | '\n' | ' ' => false,
                    _ => true,
                })))
                .map(ToOwned::to_owned),
            ))
        }

        #[cfg(unix)]
        fn maybe_escaped<'a>() -> impl Parser<Input = Input<'a>, Output = String> {
            many(choice((
                char('\'')
                    .with(recognize(skip_many(none_of("'!".chars()))))
                    .skip(char('\'')),
                char('\\').with(or(string("'"), string("!"))).map(|s| s),
                recognize(skip_many1(satisfy(|c| match c {
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '=' | '/' | ',' | '.' | '+' => {
                        true
                    }
                    _ => false,
                }))),
            )))
        }

        let (mut envs_and_args, mut envs, mut args) = (vec![], btreemap!(), vec![]);

        skip_many(
            skip_many(char(' '))
                .with(choice((
                    char('[')
                        .with(skip_many1(none_of("]\n".chars())))
                        .skip(string("] "))
                        .skip(skip_many(none_of(iter::once('\n')))),
                    or(
                        char('C').with(or(string("hecking"), string("ompiling"))),
                        char('F').with(or(string("inished"), string("resh"))),
                    )
                    .with(skip_many1(none_of(iter::once('\n')))),
                    string("Running `").with(
                        skip_many(
                            skip_many(char(' '))
                                .with(recognize(skip_many(satisfy(|c| match c {
                                    'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => true,
                                    _ => false,
                                }))))
                                .and(or(
                                    char('=').with(maybe_escaped()).map(Either::Left),
                                    maybe_escaped().map(Either::Right),
                                ))
                                .and_then(|(fst, rest)| {
                                    let rest_is_empty = match &rest {
                                        Either::Left(rest) | Either::Right(rest) => rest.is_empty(),
                                    };
                                    if fst.is_empty() && rest_is_empty {
                                        envs_and_args.push((
                                            mem::replace(&mut envs, btreemap!()),
                                            mem::replace(&mut args, vec![]),
                                        ));
                                        Err(easy::Error::Expected(Info::Borrowed("`")))
                                    } else {
                                        // https://github.com/rust-lang/cargo/blob/5218d04b3160c62b99f3decbcda77f73d321bf58/src/cargo/util/process_builder.rs#L43
                                        match rest {
                                            Either::Left(mut rest) => {
                                                if !fst.is_empty()
                                                    && (!cfg!(windows) || rest.ends_with("&&"))
                                                    && args.is_empty()
                                                {
                                                    if cfg!(windows) {
                                                        rest.pop();
                                                        rest.pop();
                                                    }
                                                    envs.insert(fst, rest);
                                                } else {
                                                    args.push(format!("{}={}", fst, rest));
                                                }
                                                Ok(())
                                            }
                                            Either::Right(rest) => {
                                                if !(cfg!(windows)
                                                    && rest == "set"
                                                    && args.is_empty())
                                                {
                                                    args.push(format!("{}{}", fst, rest));
                                                }
                                                Ok(())
                                            }
                                        }
                                    }
                                }),
                        )
                        .skip(char('`')),
                    ),
                )))
                .skip(char('\n')),
        )
        .skip(eof())
        .easy_parse(State::with_positioner(stderr, SourcePosition::new()))
        .map_err(|e| e.map_range(ToOwned::to_owned))
        .with_context(|_| crate::ErrorKind::ParseCargoBuildVvStderr {
            stderr: stderr.to_owned(),
        })?;

        envs_and_args
            .into_iter()
            .filter(|(_, args)| args.len() > 1) // build-script-build
            .map(|(envs, args)| {
                let opts = RustcOpts::from_iter_safe(&args)
                    .with_context(|_| crate::ErrorKind::ParseRustcOptions { args })?;
                Ok((opts.input().to_owned().into(), (envs, opts)))
            })
            .collect()
    }
}

mod process {
    use crate::ExecutableTarget;

    use cargo_metadata::Metadata;
    use derive_more::Display;
    use failure::ResultExt as _;
    use fixedbitset::FixedBitSet;
    use itertools::Itertools as _;
    use log::info;
    use maplit::btreemap;
    use once_cell::sync::Lazy;
    use regex::Regex;
    use structopt::StructOpt;
    use url::Url;

    use std::collections::{BTreeMap, BTreeSet};
    use std::ffi::{OsStr, OsString};
    use std::fmt;
    use std::ops::{Deref, Range};
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output, Stdio};
    use std::str::FromStr;

    /// Runs `cargo metadata`.
    pub fn cargo_metadata<S: AsRef<OsStr>, P1: AsRef<Path>, P2: AsRef<Path>>(
        cargo: S,
        cwd: P1,
        manifest_path: Option<P2>,
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
            .args(&args)
            .cwd(Some(cwd))
            .wait::<(ExitStatusSuccess, String, ())>()?;

        crate::from_json(&stdout, "`cargo metadata` output")
    }

    pub(crate) fn cargo_clean(
        cargo: &Path,
        manifest_path: &Path,
        specs: &BTreeSet<&Url>,
        target_dir: &Path,
        cwd: &Path,
    ) -> crate::Result<()> {
        let mut args = vec![
            OsStr::new("clean"),
            OsStr::new("-q"),
            OsStr::new("--manifest-path"),
            OsStr::new(manifest_path),
            OsStr::new("--target-dir"),
            OsStr::new(target_dir),
        ];
        if !specs.is_empty() {
            for spec in specs {
                args.push("-p".as_ref());
                args.push(spec.as_str().as_ref());
            }
        }

        ProcessBuilder::new(cargo)
            .args(&args)
            .cwd(Some(cwd))
            .wait::<(ExitStatusSuccess, (), ())>()
            .map(|_| ())
    }

    pub(crate) fn cargo_build_vv(
        cargo: &Path,
        manifest_path: &Path,
        target: Option<&ExecutableTarget>,
        target_dir: &Path,
        cwd: &Path,
        debug: bool,
    ) -> crate::Result<String> {
        let mut args = vec![OsStr::new("build"), OsStr::new("-vv")];
        if !debug {
            args.push("--release".as_ref());
        }
        args.push("--target-dir".as_ref());
        args.push(target_dir.as_ref());
        args.push("--manifest-path".as_ref());
        args.push(manifest_path.as_ref());
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

        let (ExitStatusSuccess, (), stderr) = ProcessBuilder::new(cargo)
            .args(&args)
            .cwd(Some(cwd))
            .wait()?;
        Ok(stderr)
    }

    pub(crate) struct ProcessBuilder {
        arg0: OsString,
        args: Vec<OsString>,
        envs: BTreeMap<String, String>,
        cwd: Option<PathBuf>,
    }

    impl ProcessBuilder {
        pub(crate) fn new(arg0: impl AsRef<OsStr>) -> Self {
            Self {
                arg0: arg0.as_ref().to_owned(),
                args: vec![],
                envs: btreemap!(),
                cwd: None,
            }
        }

        pub(crate) fn args(mut self, args: &[impl AsRef<OsStr>]) -> Self {
            self.args.extend(args.iter().map(|s| s.as_ref().to_owned()));
            self
        }

        pub(crate) fn envs<K: AsRef<str>, V: AsRef<str>, I: IntoIterator<Item = (K, V)>>(
            mut self,
            envs: I,
        ) -> Self {
            let envs = envs
                .into_iter()
                .map(|(k, v)| (k.as_ref().to_owned(), v.as_ref().to_owned()));
            self.envs.extend(envs);
            self
        }

        pub(crate) fn cwd(self, cwd: Option<impl AsRef<Path>>) -> Self {
            Self {
                cwd: cwd.map(|cwd| cwd.as_ref().to_owned()),
                ..self
            }
        }

        pub(crate) fn wait<O: ProcessedOutput>(&self) -> crate::Result<O> {
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
                .unwrap_or_else(crate::current_dir)?;

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

            let output = Command::new(&self.arg0)
                .args(&self.args)
                .envs(&self.envs)
                .current_dir(cwd)
                .stdin(Stdio::null())
                .stdout(null_or_piped(O::IGNORE_STDOUT))
                .stderr(null_or_piped(O::IGNORE_STDERR))
                .output()
                .with_context(|_| crate::ErrorKind::Command {
                    arg0: self.arg0.clone(),
                })?;

            info!("{}", output.status);

            O::process(&self.arg0, output)
        }
    }

    pub(crate) struct ExitStatusSuccess;

    pub(crate) trait ProcessedOutput: Sized {
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
        pub(crate) fn input(&self) -> &str {
            &self.input
        }

        pub(crate) fn r#extern(&self) -> &[Extern] {
            &self.r#extern
        }

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
                    args.push(r#extern);
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
}

#[doc(inline)]
pub use crate::{
    error::{Error, ErrorKind},
    process::cargo_metadata,
};

use crate::process::{ProcessBuilder, RustcOpts};

use cargo_metadata::{Metadata, NodeDep, PackageId};
use failure::ResultExt as _;
use fixedbitset::FixedBitSet;
use if_chain::if_chain;
use indexmap::IndexSet;
use log::{info, warn};
use maplit::hashset;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::{env, fs};

fn current_dir() -> crate::Result<PathBuf> {
    env::current_dir()
        .with_context(|_| crate::ErrorKind::Getcwd)
        .map_err(Into::into)
}

fn read_toml<T: DeserializeOwned>(path: &Path) -> crate::Result<T> {
    let toml = fs::read_to_string(path).with_context(|_| crate::ErrorKind::ReadFile {
        path: path.to_owned(),
    })?;
    toml::from_str(&toml)
        .with_context(|_| crate::ErrorKind::Deserialize {
            what: "a TOML file ",
        })
        .map_err(Into::into)
}

fn from_json<T: DeserializeOwned>(json: &str, what: &'static str) -> crate::Result<T> {
    serde_json::from_str(json)
        .with_context(|_| crate::ErrorKind::Deserialize { what })
        .map_err(Into::into)
}

/// Result.
pub type Result<T> = std::result::Result<T, crate::Error>;

/// Outcome.
#[derive(Serialize)]
pub struct Outcome {
    pub used: IndexSet<PackageId>,
    pub unused: IndexSet<PackageId>,
}

impl Outcome {
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).expect("should not fail")
    }

    pub fn to_json_string_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("should not fail")
    }
}

/// `bin`, `example`, `test`, or `bench`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ExecutableTarget {
    Bin(String),
    Example(String),
    Test(String),
    Bench(String),
}

impl ExecutableTarget {
    pub fn try_from_options(
        bin: &Option<String>,
        example: &Option<String>,
        test: &Option<String>,
        bench: &Option<String>,
    ) -> Option<Self> {
        if let Some(bin) = bin {
            Some(Self::Bin(bin.clone()))
        } else if let Some(example) = example {
            Some(Self::Example(example.clone()))
        } else if let Some(test) = test {
            Some(Self::Test(test.clone()))
        } else if let Some(bench) = bench {
            Some(Self::Bench(bench.clone()))
        } else {
            None
        }
    }
}

pub struct CargoUnused<'a> {
    metadata: &'a Metadata,
    target: Option<ExecutableTarget>,
    cargo: Option<PathBuf>,
    cwd: Option<PathBuf>,
    debug: bool,
}

impl<'a> CargoUnused<'a> {
    pub fn new(metadata: &'a Metadata) -> Self {
        Self {
            metadata,
            target: None,
            cargo: None,
            cwd: None,
            debug: false,
        }
    }

    pub fn target(self, target: Option<ExecutableTarget>) -> Self {
        Self { target, ..self }
    }

    pub fn cargo<P: AsRef<Path>>(self, cargo: Option<P>) -> Self {
        let cargo = cargo.map(|p| p.as_ref().to_owned());
        Self { cargo, ..self }
    }

    pub fn cwd<P: AsRef<Path>>(self, cwd: Option<P>) -> Self {
        let cwd = cwd.map(|p| p.as_ref().to_owned());
        Self { cwd, ..self }
    }

    pub fn debug(self, debug: bool) -> Self {
        Self { debug, ..self }
    }

    pub fn run(&self) -> crate::Result<Outcome> {
        let metadata = self.metadata;
        let target = self.target.as_ref();
        let cwd = self.cwd.clone().map(Ok).unwrap_or_else(current_dir)?;
        let debug = self.debug;
        let cargo = self
            .cargo
            .clone()
            .or_else(|| env::var_os("CARGO").map(Into::into))
            .ok_or_else(|| crate::ErrorKind::CargoEnvVarNotPresent)?;

        let packages = metadata
            .packages
            .iter()
            .map(|p| (&p.id, p))
            .collect::<HashMap<_, _>>();

        let specs = metadata
            .packages
            .iter()
            .map(|p| parse::package_id_to_sepc(&p.id).map(|s| (&p.id, s)))
            .collect::<crate::Result<HashMap<_, _>>>()?;

        let resolve = metadata
            .resolve
            .as_ref()
            .ok_or(crate::ErrorKind::ResolveNotPresent)?;
        let root = resolve
            .root
            .as_ref()
            .ok_or(crate::ErrorKind::RootNotFound)?;
        let manifest_path = &packages[root].manifest_path;

        let nodes = resolve
            .nodes
            .iter()
            .map(|n| (&n.id, n))
            .collect::<HashMap<_, _>>();

        let src_paths = {
            let mut src_paths = metadata
                .packages
                .iter()
                .map(|package| {
                    let values = package
                        .targets
                        .iter()
                        .filter(|target| {
                            target
                                .kind
                                .iter()
                                .any(|k| ["lib", "proc-macro", "custom-build"].contains(&k.deref()))
                        })
                        .map(|t| &t.src_path)
                        .collect();
                    (&package.id, values)
                })
                .collect::<HashMap<_, SmallVec<[_; 1]>>>();
            let root_bin_src_path = match target {
                Some(target) => {
                    let (name, kind) = match target {
                        ExecutableTarget::Bin(name) => (name, "bin"),
                        ExecutableTarget::Example(name) => (name, "example"),
                        ExecutableTarget::Test(name) => (name, "test"),
                        ExecutableTarget::Bench(name) => (name, "bench"),
                    };
                    &packages[root]
                        .targets
                        .iter()
                        .find(|t| t.name == *name && t.kind.contains(&kind.to_owned()))
                        .ok_or_else(|| {
                            let name = name.clone();
                            crate::ErrorKind::NoSuchTarget { kind, name }
                        })?
                        .src_path
                }
                None => {
                    let bins = packages[root]
                        .targets
                        .iter()
                        .filter(|t| t.kind.iter().any(|k| k == "bin"))
                        .collect::<Vec<_>>();
                    if bins.len() == 1 {
                        &bins[0].src_path
                    } else {
                        let default_run = read_toml::<CargoToml>(&packages[root].manifest_path)?
                            .package
                            .default_run
                            .ok_or_else(|| crate::ErrorKind::AmbiguousTarget)?;
                        &packages[root]
                            .targets
                            .iter()
                            .find(|t| t.name == default_run && t.kind.contains(&"bin".to_owned()))
                            .ok_or_else(|| {
                                let (kind, name) = ("bin", default_run.clone());
                                crate::ErrorKind::NoSuchTarget { kind, name }
                            })?
                            .src_path
                    }
                }
            };
            src_paths
                .entry(root)
                .and_modify(|ps| ps.push(root_bin_src_path))
                .or_insert_with(|| [root_bin_src_path].into());

            src_paths
        };

        let target_dir = metadata.workspace_root.join("target").join("cargo_unused");

        let specs_to_clean = specs.values().collect();
        process::cargo_clean(&cargo, manifest_path, &specs_to_clean, &target_dir, &cwd)?;

        let stderr =
            process::cargo_build_vv(&cargo, manifest_path, target, &target_dir, &cwd, debug)?;
        let by_src_path = parse::cargo_build_vv_stderr_to_opts_and_envs(&stderr)?
            .into_iter()
            .map(|(src_path, values)| {
                if src_path.is_absolute() {
                    (src_path, values)
                } else {
                    (metadata.workspace_root.join(&src_path), values)
                }
            })
            .collect::<HashMap<_, _>>();

        let used = {
            let arg0 = cargo
                .with_file_name("rustc")
                .with_extension(cargo.extension().unwrap_or_default());

            let mut used = hashset!(root);
            let mut cur = hashset!(root);

            while !cur.is_empty() {
                let mut next = hashset!();
                for cur in cur {
                    for &src_path in &src_paths[cur] {
                        let (envs, opts) = by_src_path.get(src_path).ok_or_else(|| {
                            crate::ErrorKind::UnexpectedSrcPath {
                                src_path: src_path.clone(),
                            }
                        })?;
                        let output = filter_actually_used_crates(
                            &arg0,
                            &opts,
                            &envs,
                            &metadata.workspace_root,
                            &nodes[cur].deps,
                        )?;
                        for &output in &output {
                            if used.insert(output) {
                                next.insert(output);
                            }
                        }
                    }
                }
                cur = next;
            }
            used
        };

        let mut used = used.into_iter().collect::<Vec<_>>();
        let mut unused = metadata
            .packages
            .iter()
            .map(|p| &p.id)
            .filter(|id| !used.contains(&id))
            .collect::<Vec<_>>();
        used.sort_by_key(|&id| (&packages[id].name, &packages[id].version, id));
        unused.sort_by_key(|&id| (&packages[id].name, &packages[id].version, id));

        return Ok(Outcome {
            used: used.into_iter().cloned().collect(),
            unused: unused.into_iter().cloned().collect(),
        });

        #[derive(Deserialize)]
        struct CargoToml {
            package: CargoTomlPackage,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct CargoTomlPackage {
            default_run: Option<String>,
        }

        fn filter_actually_used_crates<'a>(
            rustc: &Path,
            opts: &RustcOpts,
            envs: &BTreeMap<&str, String>,
            cwd: &Path,
            deps: &'a [NodeDep],
        ) -> crate::Result<HashSet<&'a PackageId>> {
            #[derive(Deserialize)]
            struct ErrorMessage {
                message: String,
                code: Option<ErrorMessageCode>,
            }

            #[derive(Deserialize)]
            struct ErrorMessageCode {
                code: String,
            }

            let mut exclusion = FixedBitSet::with_capacity(opts.r#extern().len());
            exclusion.insert_range(0..opts.r#extern().len());

            let something_wrong = 'run: loop {
                static E0432_SINGLE_MOD: Lazy<Regex> =
                    lazy_regex!(r"\Aunresolved import `([a-zA-Z0-9_]+)`\z");
                static E0433_SINGLE_MOD: Lazy<Regex> = lazy_regex!(
                    r"\Afailed to resolve: [a-z ]+`([a-zA-Z0-9_]+)`( in `\{\{root\}\}`)?\z",
                );
                static E0463_SINGLE_MOD: Lazy<Regex> =
                    lazy_regex!(r"\Acan't find crate for `([a-zA-Z0-9_]+)`\z");

                let (success, _, stderr) = ProcessBuilder::new(rustc)
                    .args(&opts.to_args(&exclusion))
                    .envs(envs)
                    .cwd(Some(cwd))
                    .wait::<(bool, (), String)>()?;
                if success {
                    break false;
                } else {
                    let mut updated = false;
                    let mut num_e0432 = 0;
                    let mut num_e0433 = 0;
                    let mut num_e0463 = 0;
                    let mut num_others = 0;

                    for line in stderr.lines() {
                        let msg = from_json::<ErrorMessage>(line, "an error message")?;

                        if_chain! {
                            if let Some(code) = &msg.code;
                            if let Some(regex) = match &*code.code {
                                "E0432" => {
                                    num_e0432 += 1;
                                    Some(&E0432_SINGLE_MOD)
                                }
                                "E0433" => {
                                    num_e0433 += 1;
                                    Some(&E0433_SINGLE_MOD)
                                }
                                "E0463" => {
                                    num_e0463 += 1;
                                    Some(&E0463_SINGLE_MOD)
                                }
                                "E0658" => {
                                    warn!("Found E0658. Trying to exclude crates one by one");
                                    break 'run true;
                                }
                                _ => {
                                    num_others += 1;
                                    None
                                }
                            };
                            if let Some(caps) = regex.captures(&msg.message);
                            if let Some(pos) = opts
                                .r#extern()
                                .iter()
                                .position(|e| *e.name() == caps[1]);
                            then {
                                updated |= exclusion[pos];
                                exclusion.set(pos, false);
                            }
                        }
                    }

                    info!(
                        "E0432: {}, E0433: {}, E0483: {}, other error(s): {}",
                        num_e0432, num_e0433, num_e0463, num_others,
                    );

                    if !updated {
                        warn!("Something is wrong. Trying to exclude crates one by one");
                        break true;
                    }
                }
            };

            if something_wrong {
                exclusion.clear();
                for i in 0..opts.r#extern().len() {
                    exclusion.insert(i);
                    let (success, (), ()) = ProcessBuilder::new(rustc)
                        .args(&opts.to_args(&exclusion))
                        .cwd(Some(cwd))
                        .wait()?;
                    exclusion.set(i, success);
                }
            }

            let deps = deps
                .iter()
                .map(|d| (&*d.name, &d.pkg))
                .collect::<HashMap<_, _>>();
            Ok(opts
                .r#extern()
                .iter()
                .enumerate()
                .filter(|&(i, _)| !exclusion[i])
                .flat_map(|(_, e)| deps.get(&e.name()).cloned())
                .collect())
        }
    }
}
