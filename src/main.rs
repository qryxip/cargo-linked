use cargo_linked::Cargo;

use cargo::core::shell::Shell;
use structopt::StructOpt as _;

use std::io;

fn main() {
    let Cargo::Linked(opt) = Cargo::from_args();
    let mut config = cargo::Config::default()
        .unwrap_or_else(|e| cargo::exit_with_error(e.into(), &mut Shell::new()));
    if let Err(err) = opt.run(&mut config, io::stdout()) {
        cargo::exit_with_error(err, &mut config.shell())
    }
}
