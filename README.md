# cargo-unused

[![Build Status](https://img.shields.io/travis/com/qryxip/cargo-unused/master.svg?label=windows%20%26%20macos%20%26%20linux)](https://travis-ci.com/qryxip/cargo-unused)
![Maintenance](https://img.shields.io/maintenance/yes/2019)
![license](https://img.shields.io/badge/license-MIT%20OR%20Apache%202.0-blue)

A Cargo subcommand to find unused crates.

## Usage

```
cargo-unused 0.0.0
Ryo Yamashita <qryxip@gmail.com>
Find unused crates.

USAGE:
    cargo unused [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --color <WHEN>    Coloring [default: auto]  [possible values: auto, always, never]
```

## Installation

`cargo-unused` is not yet uploaded to [crates.io](https://crates.io).

```console
$ cargo install --git https://github.com/qryxip/cargo-unused
```

## License

Dual-licensed under [MIT](https://opensource.org/licenses/MIT) and [Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0).