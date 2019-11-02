use cargo_linked::LinkedPackages;

use cargo::core::compiler::CompileMode;
use cargo::core::shell::Shell;
use cargo::ops::Packages;
use cargo::CargoResult;
use structopt::clap::AppSettings;
use structopt::StructOpt;

use std::io::{self, Write};
use std::path::PathBuf;

fn main() {
    let Opt::Linked(opt) = Opt::from_args();
    let mut config = cargo::Config::default()
        .unwrap_or_else(|e| cargo::exit_with_error(e.into(), &mut Shell::new()));
    if let Err(err) = opt.run(&mut config, io::stdout()) {
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
    #[structopt(long, help("Build the target skipping the \"unused\" crates"))]
    demonstrate: bool,
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
        conflicts_with_all(&["lib", "bin", "test", "bench"]),
        help("Target the `example`")
    )]
    example: Option<String>,
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
    fn run(self, config: &mut cargo::Config, mut stdout: impl Write) -> CargoResult<()> {
        let Self {
            demonstrate,
            lib,
            debug,
            all_features,
            no_default_features,
            frozen,
            locked,
            offline,
            jobs,
            bin,
            example,
            test,
            bench,
            features,
            manifest_path,
            color,
        } = self;

        cargo_linked::util::Configure {
            manifest_path: &manifest_path,
            color: &color,
            frozen,
            locked,
            offline,
            modify_target_dir: |d| d.join("cargo_linked").join("check"),
        }
        .configure(config)?;

        let ws = cargo_linked::util::workspace(&config, &manifest_path)?;

        let (packages, resolve) = Packages::All.to_package_id_specs(&ws).and_then(|specs| {
            cargo::ops::resolve_ws_precisely(
                &ws,
                &features,
                all_features,
                no_default_features,
                &specs,
            )
        })?;

        let (compile_opts, target) = cargo_linked::util::CompileOptionsForSingleTarget {
            ws: &ws,
            jobs: &jobs,
            lib,
            bin: &bin,
            example: &example,
            test: &test,
            bench: &bench,
            release: !debug,
            features: &features,
            all_features,
            no_default_features,
            manifest_path: &manifest_path,
            compile_mode: CompileMode::Check {
                test: test.is_some(),
            },
        }
        .compile_options_for_single_target()?;

        let outcome = LinkedPackages::find(&ws, &packages, &resolve, &compile_opts, target)?;

        if demonstrate {
            drop(packages);

            cargo_linked::util::Configure {
                manifest_path: &manifest_path,
                color: &color,
                frozen,
                locked,
                offline,
                modify_target_dir: |d| d.parent().unwrap().join("demonstrate"),
            }
            .configure(config)?;

            let ws = cargo_linked::util::workspace(&config, &manifest_path)?;

            let (compile_opts, _) = cargo_linked::util::CompileOptionsForSingleTarget {
                ws: &ws,
                jobs: &jobs,
                lib,
                bin: &bin,
                example: &example,
                test: &test,
                bench: &bench,
                release: !debug,
                features: &features,
                all_features,
                no_default_features,
                manifest_path: &manifest_path,
                compile_mode: CompileMode::Build,
            }
            .compile_options_for_single_target()?;

            cargo_linked::demonstrate(&ws, &compile_opts, outcome.used.clone())?;
        }

        let outcome = miniserde::json::to_string(&outcome);
        stdout.write_all(outcome.as_ref())?;
        stdout.flush().map_err(Into::into)
    }
}
