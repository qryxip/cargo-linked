use cargo::core::shell::Shell;
use failure::Fallible;
use structopt::clap::AppSettings;
use structopt::StructOpt;

use std::io::{self, Write as _};
use std::path::PathBuf;

fn main() {
    let Opt::Linked(opt) = Opt::from_args();
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
    bin_name("cargo"),
    global_settings(&[AppSettings::ColoredHelp, AppSettings::DeriveDisplayOrder])
)]
enum Opt {
    #[structopt(author, about, name = "linked")]
    Linked(OptLinked),
}

#[derive(Debug, StructOpt)]
struct OptLinked {
    #[structopt(long, help("Target the `lib`"))]
    lib: bool,
    #[structopt(long, help("Run in debug mode"))]
    debug: bool,
    #[structopt(long, help("Activate all available features"))]
    all_features: bool,
    #[structopt(long, help("Do not activate the `default` config"))]
    no_default_features: bool,
    #[structopt(long, help("Require Cargo.lock and cache are up to date"))]
    frozen: bool,
    #[structopt(long, help("Require Cargo.lock is up to date"))]
    locked: bool,
    #[structopt(long, help("Run without accessing the network"))]
    offline: bool,
    #[structopt(
        short,
        long,
        value_name("N"),
        help("Number of parallel jobs, defaults to # of CPUs")
    )]
    jobs: Option<String>,
    #[structopt(
        long,
        value_name("NAME"),
        conflicts_with_all(&["lib", "example", "test", "bench"]),
        help("Target the `bin`")
    )]
    bin: Option<String>,
    #[structopt(
        long,
        value_name("NAME"),
        conflicts_with_all(&["lib", "bin", "example", "bench"]),
        help("Target the `test`")
    )]
    test: Option<String>,
    #[structopt(
        long,
        value_name("NAME"),
        conflicts_with_all(&["lib", "bin", "example", "test"]),
        help("Target the `bench`")
    )]
    bench: Option<String>,
    #[structopt(
        long,
        value_name("NAME"),
        conflicts_with_all(&["lib", "bin", "test", "bench"]),
        help("Target the `example`")
    )]
    example: Option<String>,
    #[structopt(
        long,
        value_name("FEATURES"),
        min_values(1),
        help("Space-separated list of features to activate")
    )]
    features: Vec<String>,
    #[structopt(long, value_name("PATH"), help("Path to Cargo.toml"))]
    manifest_path: Option<PathBuf>,
    #[structopt(long, value_name("WHEN"), help("Coloring: auto, always, never"))]
    color: Option<String>,
}

impl OptLinked {
    fn run(&self, config: &mut cargo::Config) -> Fallible<()> {
        let Self {
            lib,
            debug,
            all_features,
            no_default_features,
            frozen,
            locked,
            offline,
            jobs,
            bin,
            test,
            bench,
            example,
            features,
            manifest_path,
            color,
        } = self;

        cargo_linked::util::Configure {
            manifest_path,
            color,
            frozen: *frozen,
            locked: *locked,
            offline: *offline,
            modify_target_dir: |d| d.join("cargo_linked").join("target"),
        }
        .configure(config)?;

        let ws = cargo_linked::util::workspace(config, manifest_path)?;
        let (compile_options, target) = cargo_linked::util::CompileOptionsForSingleTarget {
            ws: &ws,
            jobs,
            lib: *lib,
            bin,
            test,
            bench,
            example,
            debug: *debug,
            features,
            all_features: *all_features,
            no_default_features: *no_default_features,
            manifest_path,
        }
        .compile_options_for_single_target()?;

        let outcome = cargo_linked::LinkedPackages::find(&ws, &compile_options, target)?;
        let outcome = miniserde::json::to_string(&outcome);
        io::stdout().write_all(outcome.as_ref()).map_err(Into::into)
    }
}
