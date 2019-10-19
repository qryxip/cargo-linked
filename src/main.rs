use cargo_linked::util::CompileOptionsForSingleTargetArgs;

use cargo::core::shell::Shell;
use failure::Fallible;
use structopt::clap::AppSettings;
use structopt::StructOpt;

use std::io::{self, Write as _};
use std::path::PathBuf;

fn main() {
    let opt = Opt::from_args();
    let mut config = cargo::Config::default()
        .unwrap_or_else(|e| cargo::exit_with_error(e.into(), &mut Shell::new()));
    if let Err(err) = opt.run(&mut config) {
        cargo::exit_with_error(err.into(), &mut config.shell())
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    author,
    about,
    bin_name = "cargo",
    global_settings(&[AppSettings::ColoredHelp, AppSettings::DeriveDisplayOrder])
)]
enum Opt {
    #[structopt(author, about, name = "linked")]
    Linked {
        #[structopt(long, help = "Run in debug mode")]
        debug: bool,
        #[structopt(long, help = "Target the `lib`")]
        lib: bool,
        #[structopt(
            long,
            value_name = "NAME",
            conflicts_with_all(&["lib", "example", "test", "bench"]),
            help = "Target `bin`"
        )]
        bin: Option<String>,
        #[structopt(
            long,
            value_name = "NAME",
            conflicts_with_all(&["lib", "bin", "example", "bench"]),
            help = "Target `test`"
        )]
        test: Option<String>,
        #[structopt(
            long,
            value_name = "NAME",
            conflicts_with_all(&["lib", "bin", "example", "test"]),
            help = "Target `bench`"
        )]
        bench: Option<String>,
        #[structopt(
            long,
            value_name = "NAME",
            conflicts_with_all(&["lib", "bin", "test", "bench"]),
            help = "Target `example`"
        )]
        example: Option<String>,
        #[structopt(
            long,
            value_name = "PATH",
            parse(from_os_str),
            help = "Path to Cargo.toml"
        )]
        manifest_path: Option<PathBuf>,
        #[structopt(
            long,
            value_name("WHEN"),
            default_value("auto"),
            possible_values(&["auto", "always", "never"]),
            help("Coloring")
        )]
        color: String,
    },
}

impl Opt {
    fn run(&self, config: &mut cargo::Config) -> Fallible<()> {
        let Self::Linked {
            debug,
            lib,
            bin,
            test,
            bench,
            example,
            manifest_path,
            color,
        } = self;

        cargo_linked::util::configure(config, manifest_path, color, |target_dir| {
            target_dir.join("cargo_linked").join("target")
        })?;

        let ws = cargo_linked::util::workspace(config, manifest_path)?;
        let (compile_options, target) = cargo_linked::util::compile_options_for_single_target(
            CompileOptionsForSingleTargetArgs {
                ws: &ws,
                debug: *debug,
                lib: *lib,
                bin,
                test,
                bench,
                example,
                manifest_path,
            },
        )?;

        let outcome = cargo_linked::LinkedPackages::find(&ws, &compile_options, target)?;
        let outcome = miniserde::json::to_string(&outcome);
        io::stdout().write_all(outcome.as_ref()).map_err(Into::into)
    }
}
