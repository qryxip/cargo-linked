# cargo-linked

![CI](https://github.com/qryxip/cargo-linked/workflows/CI/badge.svg)
![Maintenance](https://img.shields.io/maintenance/yes/2019)
![license](https://img.shields.io/badge/license-MIT%20OR%20Apache%202.0-blue)
[![dependency status](https://deps.rs/repo/github/qryxip/cargo-linked/status.svg)](https://deps.rs/repo/github/qryxip/cargo-linked)

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
        --lib                    Target the `lib`
        --debug                  Run in debug mode
        --all-features           Activate all available features
        --no-default-features    Do not activate the `default` config
        --frozen                 Require Cargo.lock and cache are up to date
        --locked                 Require Cargo.lock is up to date
        --offline                Run without accessing the network
    -h, --help                   Prints help information
    -V, --version                Prints version information

OPTIONS:
    -j, --jobs <N>                  Number of parallel jobs, defaults to # of CPUs
        --bin <NAME>                Target the `bin`
        --example <NAME>            Target the `example`
        --test <NAME>               Target the `test`
        --bench <NAME>              Target the `bench`
        --features <FEATURES>...    Space-separated list of features to activate
        --manifest-path <PATH>      Path to Cargo.toml
        --color <WHEN>              Coloring: auto, always, never
```

```
$ cargo linked --debug 2>&- | jq
{
  "used": [
    "aho-corasick 0.7.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "ansi_term 0.11.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "ansi_term 0.12.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "atty 0.2.13 (registry+https://github.com/rust-lang/crates.io-index)",
    "autocfg 0.1.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "backtrace 0.3.40 (registry+https://github.com/rust-lang/crates.io-index)",
    "backtrace-sys 0.1.32 (registry+https://github.com/rust-lang/crates.io-index)",
    "bitflags 1.2.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "bstr 0.2.8 (registry+https://github.com/rust-lang/crates.io-index)",
    "byteorder 1.3.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "bytes 0.4.12 (registry+https://github.com/rust-lang/crates.io-index)",
    "bytesize 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "c2-chacha 0.2.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "cargo 0.39.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "cargo-linked 0.0.0 (path+file:///home/ryo/src/cargo-linked)",
    "cc 1.0.46 (registry+https://github.com/rust-lang/crates.io-index)",
    "cfg-if 0.1.10 (registry+https://github.com/rust-lang/crates.io-index)",
    "clap 2.33.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "crates-io 0.27.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "crc32fast 1.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "crossbeam-channel 0.3.9 (registry+https://github.com/rust-lang/crates.io-index)",
    "crossbeam-utils 0.6.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "crypto-hash 0.3.4 (registry+https://github.com/rust-lang/crates.io-index)",
    "curl 0.4.25 (registry+https://github.com/rust-lang/crates.io-index)",
    "curl-sys 0.4.23 (registry+https://github.com/rust-lang/crates.io-index)",
    "derive_more 0.15.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "failure 0.1.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "failure_derive 0.1.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "filetime 0.2.7 (registry+https://github.com/rust-lang/crates.io-index)",
    "fixedbitset 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "flate2 1.0.12 (registry+https://github.com/rust-lang/crates.io-index)",
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
    "http 0.1.19 (registry+https://github.com/rust-lang/crates.io-index)",
    "idna 0.1.5 (registry+https://github.com/rust-lang/crates.io-index)",
    "idna 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "if_chain 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "ignore 0.4.10 (registry+https://github.com/rust-lang/crates.io-index)",
    "im-rc 13.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "iovec 0.1.4 (registry+https://github.com/rust-lang/crates.io-index)",
    "itoa 0.4.4 (registry+https://github.com/rust-lang/crates.io-index)",
    "jobserver 0.1.17 (registry+https://github.com/rust-lang/crates.io-index)",
    "lazy_static 1.4.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "lazycell 1.2.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "libc 0.2.65 (registry+https://github.com/rust-lang/crates.io-index)",
    "libgit2-sys 0.8.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "libssh2-sys 0.2.12 (registry+https://github.com/rust-lang/crates.io-index)",
    "libz-sys 1.0.25 (registry+https://github.com/rust-lang/crates.io-index)",
    "log 0.4.8 (registry+https://github.com/rust-lang/crates.io-index)",
    "maplit 1.0.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "matches 0.1.8 (registry+https://github.com/rust-lang/crates.io-index)",
    "memchr 2.2.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "mini-internal 0.1.12 (registry+https://github.com/rust-lang/crates.io-index)",
    "miniserde 0.1.12 (registry+https://github.com/rust-lang/crates.io-index)",
    "num_cpus 1.10.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "once_cell 1.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "opener 0.4.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "openssl 0.10.25 (registry+https://github.com/rust-lang/crates.io-index)",
    "openssl-probe 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "openssl-sys 0.9.51 (registry+https://github.com/rust-lang/crates.io-index)",
    "percent-encoding 1.0.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "percent-encoding 2.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "pkg-config 0.3.16 (registry+https://github.com/rust-lang/crates.io-index)",
    "ppv-lite86 0.2.5 (registry+https://github.com/rust-lang/crates.io-index)",
    "proc-macro-error 0.2.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "proc-macro2 0.4.30 (registry+https://github.com/rust-lang/crates.io-index)",
    "proc-macro2 1.0.5 (registry+https://github.com/rust-lang/crates.io-index)",
    "quote 0.6.13 (registry+https://github.com/rust-lang/crates.io-index)",
    "quote 1.0.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "rand 0.7.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "rand_chacha 0.2.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "rand_core 0.5.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "regex 1.3.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "regex-syntax 0.6.12 (registry+https://github.com/rust-lang/crates.io-index)",
    "remove_dir_all 0.5.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "rustc-demangle 0.1.16 (registry+https://github.com/rust-lang/crates.io-index)",
    "rustc_version 0.2.3 (registry+https://github.com/rust-lang/crates.io-index)",
    "rustfix 0.4.6 (registry+https://github.com/rust-lang/crates.io-index)",
    "ryu 1.0.2 (registry+https://github.com/rust-lang/crates.io-index)",
    "same-file 1.0.5 (registry+https://github.com/rust-lang/crates.io-index)",
    "semver 0.9.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "semver-parser 0.7.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "serde 1.0.101 (registry+https://github.com/rust-lang/crates.io-index)",
    "serde_derive 1.0.101 (registry+https://github.com/rust-lang/crates.io-index)",
    "serde_ignored 0.0.4 (registry+https://github.com/rust-lang/crates.io-index)",
    "serde_json 1.0.41 (registry+https://github.com/rust-lang/crates.io-index)",
    "shell-escape 0.1.4 (registry+https://github.com/rust-lang/crates.io-index)",
    "sized-chunks 0.3.1 (registry+https://github.com/rust-lang/crates.io-index)",
    "smallvec 0.6.10 (registry+https://github.com/rust-lang/crates.io-index)",
    "socket2 0.3.11 (registry+https://github.com/rust-lang/crates.io-index)",
    "strip-ansi-escapes 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "strsim 0.8.0 (registry+https://github.com/rust-lang/crates.io-index)",
    "structopt 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)",
    "structopt-derive 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)",
    "syn 0.15.44 (registry+https://github.com/rust-lang/crates.io-index)",
    "syn 1.0.5 (registry+https://github.com/rust-lang/crates.io-index)",
    "synstructure 0.12.1 (registry+https://github.com/rust-lang/crates.io-index)",
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
      "commoncrypto 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "commoncrypto-sys 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "core-foundation 0.6.4 (registry+https://github.com/rust-lang/crates.io-index)",
      "core-foundation-sys 0.6.2 (registry+https://github.com/rust-lang/crates.io-index)",
      "fwdansi 1.0.1 (registry+https://github.com/rust-lang/crates.io-index)",
      "miow 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)",
      "rand_hc 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "redox_syscall 0.1.56 (registry+https://github.com/rust-lang/crates.io-index)",
      "schannel 0.1.16 (registry+https://github.com/rust-lang/crates.io-index)",
      "scopeguard 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)",
      "vcpkg 0.2.7 (registry+https://github.com/rust-lang/crates.io-index)",
      "wasi 0.7.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "winapi 0.3.8 (registry+https://github.com/rust-lang/crates.io-index)",
      "winapi-i686-pc-windows-gnu 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "winapi-util 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)",
      "winapi-x86_64-pc-windows-gnu 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "wincolor 1.0.2 (registry+https://github.com/rust-lang/crates.io-index)"
    ],
    "maybe_obsolete": [
      "adler32 1.0.4 (registry+https://github.com/rust-lang/crates.io-index)",
      "env_logger 0.6.2 (registry+https://github.com/rust-lang/crates.io-index)",
      "git2-curl 0.10.1 (registry+https://github.com/rust-lang/crates.io-index)",
      "humantime 1.3.0 (registry+https://github.com/rust-lang/crates.io-index)",
      "libnghttp2-sys 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)",
      "miniz_oxide 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)",
      "quick-error 1.2.2 (registry+https://github.com/rust-lang/crates.io-index)",
      "rustc-workspace-hack 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)"
    ]
  }
}
```

### `lib`

```rust
use std::path::PathBuf;

let jobs: Option<String> = unimplemented!();
let lib: bool = unimplemented!();
let bin: Option<String> = unimplemented!();
let example: Option<String> = unimplemented!();
let test: Option<String> = unimplemented!();
let bench: Option<String> = unimplemented!();
let debug: bool = unimplemented!();
let features: Vec<String> = unimplemented!();
let all_features: bool = unimplemented!();
let no_default_features: bool = unimplemented!();
let manifest_path: Option<PathBuf> = unimplemented!();
let color: Option<String> = unimplemented!();
let frozen: bool = unimplemented!();
let locked: bool = unimplemented!();
let offline: bool = unimplemented!();

let mut config = cargo::Config::default()?;

cargo_linked::util::Configure {
    manifest_path: &manifest_path,
    color: &color,
    frozen,
    locked,
    offline,
    modify_target_dir: |d| d.join("cargo_linked").join("target"),
}
.configure(&mut config)?;

let ws = cargo_linked::util::workspace(&config, &manifest_path)?;
let (compile_options, target) = cargo_linked::util::CompileOptionsForSingleTarget {
    ws: &ws,
    jobs: &jobs,
    lib,
    bin: &bin,
    example: &example,
    test: &test,
    bench: &bench,
    debug,
    features: &features,
    all_features,
    no_default_features,
    manifest_path: &manifest_path,
}
.compile_options_for_single_target()?;

let outcome = cargo_linked::LinkedPackages::find(&ws, &compile_options, target)?;
```

## License

Licensed under <code>[MIT](https://opensource.org/licenses/MIT) OR [Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0)</code>.
