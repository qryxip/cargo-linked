use cargo_unused::{CargoMetadata, CargoUnused, ExecutableTarget};

use failure::{Fallible, ResultExt as _};
use log::LevelFilter;
use structopt::StructOpt;
use strum::{EnumString, IntoStaticStr};
use termcolor::{BufferedStandardStream, WriteColor as _};

use std::io::{self, Write as _};
use std::path::PathBuf;
use std::{env, process};

fn main() -> io::Result<()> {
    let Opt::Unused(opt) = Opt::from_args();

    let (termcolor_color, env_logger_color) = if opt.color.should_color(atty::Stream::Stderr) {
        (
            termcolor::ColorChoice::Always,
            env_logger::WriteStyle::Always,
        )
    } else {
        (termcolor::ColorChoice::Never, env_logger::WriteStyle::Never)
    };

    env_logger::Builder::new()
        .filter_module("cargo_unused", LevelFilter::Info)
        .write_style(env_logger_color)
        .format(|buf, record| {
            let mut black_intense = buf.style();
            black_intense
                .set_color(env_logger::fmt::Color::Black)
                .set_intense(true);
            writeln!(
                buf,
                "{}{}{} {}",
                black_intense.value('['),
                buf.default_styled_level(record.level()),
                black_intense.value(']'),
                record.args(),
            )
        })
        .init();

    match opt.run() {
        Ok(output) => io::stdout().write_all(output.as_ref()),
        Err(err) => {
            let mut stderr = BufferedStandardStream::stderr(termcolor_color);
            writeln!(stderr)?;
            for (i, cause) in err.as_fail().iter_chain().enumerate() {
                let head = if i == 0 && err.as_fail().cause().is_none() {
                    "error: "
                } else if i == 0 {
                    "    error: "
                } else {
                    "caused by: "
                };
                stderr.set_color(
                    termcolor::ColorSpec::new()
                        .set_fg(Some(termcolor::Color::Red))
                        .set_bold(true),
                )?;
                stderr.write_all(head.as_ref())?;
                stderr.reset()?;
                for (i, line) in cause.to_string().lines().enumerate() {
                    if i > 0 {
                        (0..head.len()).try_for_each(|_| stderr.write_all(b" "))?;
                    }
                    writeln!(stderr, "{}", line)?;
                }
            }
            let backtrace = err.backtrace().to_string();
            if backtrace.is_empty() {
                stderr.write_all(
                    &b"note: Run with `RUST_BACKTRACE=1` environment varialbe to display a \
                       backtrace\n"[..],
                )?;
            } else {
                writeln!(stderr, "{}", backtrace)?;
            }
            stderr.flush()?;
            process::exit(1)
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
enum Opt {
    #[structopt(name = "unused")]
    Unused(OptUnused),
}

#[derive(Debug, StructOpt)]
struct OptUnused {
    #[structopt(long = "debug", help = "Run in debug mode")]
    debug: bool,
    #[structopt(
        long = "bin",
        value_name = "NAME",
        help = "Target `bin`",
        raw(
            display_order = "1",
            conflicts_with_all = r#"&["example", "test", "bench"]"#
        )
    )]
    bin: Option<String>,
    #[structopt(
        long = "example",
        value_name = "NAME",
        help = "Target `example`",
        raw(
            display_order = "2",
            conflicts_with_all = r#"&["bin", "test", "bench"]"#
        )
    )]
    example: Option<String>,
    #[structopt(
        long = "test",
        value_name = "NAME",
        help = "Target `test`",
        raw(
            display_order = "3",
            conflicts_with_all = r#"&["bin", "example", "bench"]"#
        )
    )]
    test: Option<String>,
    #[structopt(
        long = "bench",
        value_name = "NAME",
        help = "Target `bench`",
        raw(
            display_order = "4",
            conflicts_with_all = r#"&["bin", "example", "test"]"#
        )
    )]
    bench: Option<String>,
    #[structopt(
        long = "manifest-path",
        value_name = "PATH",
        parse(from_os_str),
        help = "Path to Cargo.toml",
        raw(display_order = "5")
    )]
    manifest_path: Option<PathBuf>,
    #[structopt(
        long = "color",
        value_name = "WHEN",
        help = "Coloring",
        raw(
            display_order = "6",
            default_value = "<&str>::from(ColorChoice::default())",
            possible_values = "&ColorChoice::variants()"
        )
    )]
    color: ColorChoice,
}

impl OptUnused {
    fn run(&self) -> Fallible<String> {
        let ctrl_c = tokio_signal::ctrl_c();
        let mut ctrl_c = tokio::runtime::current_thread::Runtime::new()?.block_on(ctrl_c)?;

        let cwd = env::current_dir().with_context(|_| failure::err_msg("Failed to getcwd"))?;
        let cargo =
            env::var_os("CARGO").ok_or_else(|| failure::err_msg("$CARGO is not present"))?;

        let metadata = CargoMetadata::new()
            .cargo(Some(&cargo))
            .manifest_path(self.manifest_path.as_ref())
            .cwd(Some(&cwd))
            .ctrl_c(Some(&mut ctrl_c))
            .run()?;

        let target =
            ExecutableTarget::try_from_options(&self.bin, &self.example, &self.test, &self.bench);

        let outcome = CargoUnused::new(&metadata)
            .target(target)
            .cargo(Some(cargo))
            .debug(self.debug)
            .ctrl_c(Some(&mut ctrl_c))
            .run()?;
        Ok(miniserde::json::to_string(&outcome))
    }
}

#[derive(Debug, Clone, Copy, EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab_case")]
enum ColorChoice {
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
    fn variants() -> [&'static str; 3] {
        ["auto", "always", "never"]
    }

    fn should_color(self, stream: atty::Stream) -> bool {
        #[cfg(windows)]
        static BLACKLIST: &[&str] = &["cygwin", "dumb"];

        #[cfg(not(windows))]
        static BLACKLIST: &[&str] = &["dumb"];

        match self {
            Self::Auto => {
                atty::is(stream)
                    && env::var("TERM")
                        .ok()
                        .map_or(false, |v| !BLACKLIST.contains(&v.as_ref()))
            }
            Self::Always => true,
            Self::Never => false,
        }
    }
}
