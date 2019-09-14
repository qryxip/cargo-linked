use cargo_unused::CompileOptionsForSingleTarget;

use cargo::util::config::Config;
use failure::Fallible;
use structopt::StructOpt;
use termcolor::{BufferedStandardStream, WriteColor as _};

use std::io::{self, Write as _};
use std::path::PathBuf;
use std::process;

fn main() -> Fallible<()> {
    let opt = Opt::from_args();

    let config = opt.configure()?;

    match opt.run(&config) {
        Ok(output) => io::stdout().write_all(output.as_ref()).map_err(Into::into),
        Err(err) => {
            let mut stderr = BufferedStandardStream::stderr(if config.shell().supports_color() {
                termcolor::ColorChoice::Always
            } else {
                termcolor::ColorChoice::Never
            });
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
#[structopt(author, about, bin_name = "cargo")]
enum Opt {
    #[structopt(author, about, name = "unused")]
    Unused {
        #[structopt(long, help = "Run in debug mode", display_order(1))]
        debug: bool,
        #[structopt(long, help = "Target the `lib`", display_order(2))]
        lib: bool,
        #[structopt(
            long,
            value_name = "NAME",
            conflicts_with_all(&["lib", "example", "test", "bench"]),
            help = "Target `bin`",
            display_order(1)
        )]
        bin: Option<String>,
        #[structopt(
            long,
            value_name = "NAME",
            conflicts_with_all(&["lib", "bin", "example", "bench"]),
            help = "Target `test`",
            display_order(2)
        )]
        test: Option<String>,
        #[structopt(
            long,
            value_name = "NAME",
            conflicts_with_all(&["lib", "bin", "example", "test"]),
            help = "Target `bench`",
            display_order(3)
        )]
        bench: Option<String>,
        #[structopt(
            long,
            value_name = "NAME",
            conflicts_with_all(&["lib", "bin", "test", "bench"]),
            help = "Target `example`",
            display_order(4)
        )]
        example: Option<String>,
        #[structopt(
            long,
            value_name = "PATH",
            parse(from_os_str),
            help = "Path to Cargo.toml",
            display_order(5)
        )]
        manifest_path: Option<PathBuf>,
        #[structopt(
            long,
            value_name("WHEN"),
            default_value("auto"),
            possible_values(&["auto", "always", "never"]),
            help("Coloring"),
            display_order(6)
        )]
        color: String,
    },
}

impl Opt {
    fn configure(&self) -> cargo_unused::Result<Config> {
        let Self::Unused {
            manifest_path,
            color,
            ..
        } = self;
        cargo_unused::configure(manifest_path, color)
    }

    fn run(&self, config: &Config) -> Fallible<String> {
        let Self::Unused {
            debug,
            lib,
            bin,
            test,
            bench,
            example,
            manifest_path,
            ..
        } = self;

        let ws = cargo_unused::workspace(config, manifest_path)?;
        let (compile_options, target) = CompileOptionsForSingleTarget {
            ws: &ws,
            debug: *debug,
            lib: *lib,
            bin,
            test,
            bench,
            example,
            manifest_path,
        }
        .find()?;

        let outcome = cargo_unused::LinkedPackages::find(&ws, &compile_options, target)?;
        Ok(miniserde::json::to_string(&outcome))
    }
}
