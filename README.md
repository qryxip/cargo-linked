# cargo-linked

![CI](https://github.com/qryxip/cargo-linked/workflows/CI/badge.svg)
![Maintenance](https://img.shields.io/maintenance/yes/2019)
![license](https://img.shields.io/badge/license-MIT%20OR%20Apache%202.0-blue)

A Cargo subcommand to list actually used crates.

## Installation

`cargo-linked` is not yet uploaded to [crates.io](https://crates.io).

```
$ cargo install --git https://github.com/qryxip/cargo-linked
```

## Usage

### `bin`

```
cargo-linked 0.0.0
Ryo Yamashita <qryxip@gmail.com>
List actually used crates.

USAGE:
    cargo linked [FLAGS] [OPTIONS]

FLAGS:
        --debug      Run in debug mode
        --lib        Target the `lib`
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --bin <NAME>              Target `bin`
        --test <NAME>             Target `test`
        --bench <NAME>            Target `bench`
        --example <NAME>          Target `example`
        --manifest-path <PATH>    Path to Cargo.toml
        --color <WHEN>            Coloring [default: auto]  [possible values: auto, always, never]
```

```
$ cargo linked --debug 2>&- | jq
{
  "used": [
    "aho-corasick 0.7.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "ansi_term 0.11.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "atty 0.2.13 (registry+https://github.com/rust-lang/crates.io-index)",
    "autocfg 0.1.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "backtrace 0.3.37 (registry+https://github.com/rust-lang/crates.io-index)",
    "backtrace-sys 0.1.31 (registry+https://github.com/rust-lang/crates.io-index)",
    "bitflags 1.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "bstr 0.2.8 (registry+https://github.com/rust-lang/crates.io-index)",
    "byteorder 1.3.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "bytes 0.4.12 (registry+https://github.com/rust-lang/crates.io-index)",
    "bytesize 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "c2-chacha 0.2.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "cargo 0.38.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "cargo-linked 0.0.0 (path+file:///home/ryo/src/cargo-linked)",
    "cc 1.0.45 (registry+https://github.com/rust-lang/crates.io-index)",
    "cfg-if 0.1.9 (registry+https://github.com/rust-lang/crates.io-index)",
    "clap 2.33.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "crates-io 0.26.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "crc32fast 1.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "crossbeam-channel 0.3.9 (registry+https://github.com/rust-lang/crates.io-index)",
    "crossbeam-utils 0.6.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "crypto-hash 0.3.4 (registry+https://github.com/rust-lang/crates.io-index)",
    "curl 0.4.23 (registry+https://github.com/rust-lang/crates.io-index)",
    "curl-sys 0.4.21 (registry+https://github.com/rust-lang/crates.io-index)",
    "derive_more 0.15.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "failure 0.1.5 (registry+https://github.com/rust-lang/crates.io-index)",
    "failure_derive 0.1.5 (registry+https://github.com/rust-lang/crates.io-index)",
    "filetime 0.2.7 (registry+https://github.com/rust-lang/crates.io-index)",
    "fixedbitset 0.1.9 (registry+https://github.com/rust-lang/crates.io-index)",
    "flate2 1.0.11 (registry+https://github.com/rust-lang/crates.io-index)",
    "fnv 1.0.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "foreign-types 0.3.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "foreign-types-shared 0.1.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "fs2 0.4.3 (registry+https://github.com/rust-lang/crates.io-index)",
    "getrandom 0.1.12 (registry+https://github.com/rust-lang/crates.io-index)",
    "git2 0.9.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "glob 0.3.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "globset 0.4.4 (registry+https://github.com/rust-lang/crates.io-index)",
    "heck 0.3.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "hex 0.3.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "home 0.3.4 (registry+https://github.com/rust-lang/crates.io-index)",
    "http 0.1.18 (registry+https://github.com/rust-lang/crates.io-index)",
    "humantime 1.3.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "idna 0.1.5 (registry+https://github.com/rust-lang/crates.io-index)",
    "idna 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "if_chain 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "ignore 0.4.10 (registry+https://github.com/rust-lang/crates.io-index)",
    "im-rc 13.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "iovec 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "itoa 0.4.4 (registry+https://github.com/rust-lang/crates.io-index)",
    "jobserver 0.1.17 (registry+https://github.com/rust-lang/crates.io-index)",
    "lazy_static 1.4.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "lazycell 1.2.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "libc 0.2.62 (registry+https://github.com/rust-lang/crates.io-index)",
    "libgit2-sys 0.8.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "libssh2-sys 0.2.11 (registry+https://github.com/rust-lang/crates.io-index)",
    "libz-sys 1.0.25 (registry+https://github.com/rust-lang/crates.io-index)",
    "log 0.4.8 (registry+https://github.com/rust-lang/crates.io-index)",
    "maplit 1.0.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "matches 0.1.8 (registry+https://github.com/rust-lang/crates.io-index)",
    "memchr 2.2.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "mini-internal 0.1.12 (registry+https://github.com/rust-lang/crates.io-index)",
    "miniserde 0.1.12 (registry+https://github.com/rust-lang/crates.io-index)",
    "num_cpus 1.10.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "once_cell 1.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "opener 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "openssl 0.10.24 (registry+https://github.com/rust-lang/crates.io-index)",
    "openssl-probe 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "openssl-sys 0.9.49 (registry+https://github.com/rust-lang/crates.io-index)",
    "percent-encoding 1.0.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "percent-encoding 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "pkg-config 0.3.16 (registry+https://github.com/rust-lang/crates.io-index)",
    "ppv-lite86 0.2.5 (registry+https://github.com/rust-lang/crates.io-index)",
    "proc-macro-error 0.2.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "proc-macro2 0.4.30 (registry+https://github.com/rust-lang/crates.io-index)",
    "proc-macro2 1.0.3 (registry+https://github.com/rust-lang/crates.io-index)",
    "quick-error 1.2.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "quote 0.6.13 (registry+https://github.com/rust-lang/crates.io-index)",
    "quote 1.0.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "rand 0.7.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "rand_chacha 0.2.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "rand_core 0.5.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "regex 1.3.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "regex-syntax 0.6.12 (registry+https://github.com/rust-lang/crates.io-index)",
    "remove_dir_all 0.5.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "rustc-demangle 0.1.16 (registry+https://github.com/rust-lang/crates.io-index)",
    "rustc_version 0.2.3 (registry+https://github.com/rust-lang/crates.io-index)",
    "rustfix 0.4.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "ryu 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "same-file 1.0.5 (registry+https://github.com/rust-lang/crates.io-index)",
    "semver 0.9.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "semver-parser 0.7.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "serde 1.0.100 (registry+https://github.com/rust-lang/crates.io-index)",
    "serde_derive 1.0.100 (registry+https://github.com/rust-lang/crates.io-index)",
    "serde_ignored 0.0.4 (registry+https://github.com/rust-lang/crates.io-index)",
    "serde_json 1.0.40 (registry+https://github.com/rust-lang/crates.io-index)",
    "shell-escape 0.1.4 (registry+https://github.com/rust-lang/crates.io-index)",
    "sized-chunks 0.3.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "smallvec 0.6.10 (registry+https://github.com/rust-lang/crates.io-index)",
    "socket2 0.3.11 (registry+https://github.com/rust-lang/crates.io-index)",
    "strip-ansi-escapes 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "strsim 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "structopt 0.3.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "structopt-derive 0.3.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "syn 0.15.44 (registry+https://github.com/rust-lang/crates.io-index)",
    "syn 1.0.5 (registry+https://github.com/rust-lang/crates.io-index)",
    "synstructure 0.10.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "tar 0.4.26 (registry+https://github.com/rust-lang/crates.io-index)",
    "tempfile 3.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "termcolor 1.0.5 (registry+https://github.com/rust-lang/crates.io-index)",
    "textwrap 0.11.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "thread_local 0.3.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "toml 0.5.3 (registry+https://github.com/rust-lang/crates.io-index)",
    "typenum 1.11.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "unicode-bidi 0.3.4 (registry+https://github.com/rust-lang/crates.io-index)",
    "unicode-normalization 0.1.8 (registry+https://github.com/rust-lang/crates.io-index)",
    "unicode-segmentation 1.3.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "unicode-width 0.1.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "unicode-xid 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "unicode-xid 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "url 1.7.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "url 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "url_serde 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "utf8parse 0.1.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "vec_map 0.8.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "vte 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)",
    "walkdir 2.2.9 (registry+https://github.com/rust-lang/crates.io-index)"
  ],
  "unused": {
    "trivial": [
      "adler32 1.0.3 (registry+https://github.com/rust-lang/crates.io-index)",
      "commoncrypto 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "commoncrypto-sys 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "core-foundation 0.6.4 (registry+https://github.com/rust-lang/crates.io-index)",
      "core-foundation-sys 0.6.2 (registry+https://github.com/rust-lang/crates.io-index)",
      "fwdansi 1.0.1 (registry+https://github.com/rust-lang/crates.io-index)",
      "miniz_oxide 0.3.2 (registry+https://github.com/rust-lang/crates.io-index)",
      "miow 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)",
      "rand_hc 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "redox_syscall 0.1.56 (registry+https://github.com/rust-lang/crates.io-index)",
      "schannel 0.1.15 (registry+https://github.com/rust-lang/crates.io-index)",
      "scopeguard 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)",
      "vcpkg 0.2.7 (registry+https://github.com/rust-lang/crates.io-index)",
      "wasi 0.7.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "winapi 0.2.8 (registry+https://github.com/rust-lang/crates.io-index)",
      "winapi 0.3.8 (registry+https://github.com/rust-lang/crates.io-index)",
      "winapi-i686-pc-windows-gnu 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "winapi-util 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)",
      "winapi-x86_64-pc-windows-gnu 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "wincolor 1.0.2 (registry+https://github.com/rust-lang/crates.io-index)"
    ],
    "maybe_obsolete": [
      "env_logger 0.6.2 (registry+https://github.com/rust-lang/crates.io-index)",
      "git2-curl 0.10.1 (registry+https://github.com/rust-lang/crates.io-index)",
      "libnghttp2-sys 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)",
      "miniz-sys 0.1.12 (registry+https://github.com/rust-lang/crates.io-index)",
      "rustc-workspace-hack 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)"
    ]
  }
}
```

### `lib`

```rust
use cargo::CliError;
use cargo_linked::{CompileOptionsForSingleTarget, LinkedPackages};
use structopt::StructOpt;

use std::path::PathBuf;

#[derive(StructOpt)]
#[structopt(author, about, bin_name = "cargo")]
enum Opt {
    #[structopt(author, about, name = "subcommand")]
    Subcommand {
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
    fn configure(&self) -> cargo_linked::Result<cargo::Config> {
        let Self::Subcommand {
            manifest_path,
            color,
            ..
        } = self;
        cargo_linked::configure(&manifest_path, &color)
    }

    fn run(&self, config: &cargo::Config) -> cargo_linked::Result<String> {
        let Self::Subcommand {
            debug,
            lib,
            bin,
            test,
            bench,
            example,
            manifest_path,
            ..
        } = self;

        let ws = cargo_linked::workspace(config, manifest_path)?;
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
        let outcome = LinkedPackages::find(&ws, &compile_options, target)?;
        Ok(miniserde::json::to_string(&outcome))
    }
}

let opt = Opt::from_args();
let config = opt.configure()?;
match opt.run(&config) {
    Ok(output) => {
        println!("{}", output);
    }
    Err(err) => {
        cargo::exit_with_error(CliError::new(err.into(), 1), &mut config.shell());
    }
}
```

## License

Licensed under <code>[MIT](https://opensource.org/licenses/MIT) OR [Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0)</code>.
