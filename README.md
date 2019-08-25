# cargo-unused

[![Build Status](https://img.shields.io/travis/com/qryxip/cargo-unused/master.svg?label=windows%20%26%20macos%20%26%20linux)](https://travis-ci.com/qryxip/cargo-unused)
![Maintenance](https://img.shields.io/maintenance/yes/2019)
![license](https://img.shields.io/badge/license-MIT%20OR%20Apache%202.0-blue)

A Cargo subcommand to find unused crates.

## Installation

`cargo-unused` is not yet uploaded to [crates.io](https://crates.io).

```
$ cargo install --git https://github.com/qryxip/cargo-unused
```

## Usage

```
cargo-unused 0.0.0
Ryo Yamashita <qryxip@gmail.com>
Find unused crates.

USAGE:
    cargo unused [FLAGS] [OPTIONS]

FLAGS:
        --debug      Run in debug mode
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --bin <NAME>              Target `bin`
        --example <NAME>          Target `example`
        --test <NAME>             Target `test`
        --bench <NAME>            Target `bench`
        --manifest-path <PATH>    Path to Cargo.toml
        --color <WHEN>            Coloring [default: auto]  [possible values: auto, always, never]
```

```
$ cargo unused --debug 2>&- | jq .unused
[
  "cloudabi 0.0.3 (registry+https://github.com/rust-lang/crates.io-index)",
  "redox_syscall 0.1.56 (registry+https://github.com/rust-lang/crates.io-index)",
  "winapi 0.3.7 (registry+https://github.com/rust-lang/crates.io-index)",
  "winapi-i686-pc-windows-gnu 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)",
  "winapi-util 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)",
  "winapi-x86_64-pc-windows-gnu 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)",
  "wincolor 1.0.2 (registry+https://github.com/rust-lang/crates.io-index)"
]

```

## License

Licensed under <code>[MIT](https://opensource.org/licenses/MIT) OR [Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0)</code>.