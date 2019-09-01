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

### `bin`

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
{
  "cloudabi 0.0.3 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "parking_lot_core 0.6.2 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(target_os = \"cloudabi\")"
      },
      "rand_os 0.1.3 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(target_os = \"cloudabi\")"
      }
    }
  },
  "fuchsia-cprng 0.1.1 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "rand_os 0.1.3 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(target_os = \"fuchsia\")"
      }
    }
  },
  "fuchsia-zircon 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "mio 0.6.19 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(target_os = \"fuchsia\")"
      }
    }
  },
  "fuchsia-zircon-sys 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "fuchsia-zircon 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": null
      },
      "mio 0.6.19 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(target_os = \"fuchsia\")"
      }
    }
  },
  "kernel32-sys 0.2.2 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "mio 0.6.19 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "miow 0.2.1 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": null
      }
    }
  },
  "mio-named-pipes 0.1.6 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "tokio-process 0.2.4 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      }
    }
  },
  "miow 0.2.1 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "mio 0.6.19 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      }
    }
  },
  "miow 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "mio-named-pipes 0.1.6 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      }
    }
  },
  "rdrand 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "rand_os 0.1.3 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(target_env = \"sgx\")"
      }
    }
  },
  "redox_syscall 0.1.56 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "filetime 0.2.7 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(target_os = \"redox\")"
      },
      "parking_lot_core 0.6.2 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(target_os = \"redox\")"
      },
      "socket2 0.3.11 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(target_os = \"redox\")"
      }
    }
  },
  "socket2 0.3.11 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "miow 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": null
      }
    }
  },
  "winapi 0.2.8 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "iovec 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "kernel32-sys 0.2.2 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": null
      },
      "mio 0.6.19 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "miow 0.2.1 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": null
      },
      "ws2_32-sys 0.2.1 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": null
      }
    }
  },
  "winapi 0.3.8 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "ansi_term 0.11.0 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(target_os = \"windows\")"
      },
      "atty 0.2.13 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "filetime 0.2.7 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "fs2 0.4.3 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "mio-named-pipes 0.1.6 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "miow 0.3.3 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": null
      },
      "net2 0.2.33 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "parking_lot_core 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "parking_lot_core 0.6.2 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "rand 0.6.5 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "rand_jitter 0.1.4 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(target_os = \"windows\")"
      },
      "rand_os 0.1.3 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "remove_dir_all 0.5.2 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "socket2 0.3.11 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "tokio-process 0.2.4 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "tokio-signal 0.2.7 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "winapi-util 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      },
      "wincolor 1.0.2 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": null
      }
    }
  },
  "winapi-build 0.1.1 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "kernel32-sys 0.2.2 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": null
      },
      "ws2_32-sys 0.2.1 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": null
      }
    }
  },
  "winapi-i686-pc-windows-gnu 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "winapi 0.3.8 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "i686-pc-windows-gnu"
      }
    }
  },
  "winapi-util 0.1.2 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "wincolor 1.0.2 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": null
      }
    }
  },
  "winapi-x86_64-pc-windows-gnu 0.4.0 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "winapi 0.3.8 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "x86_64-pc-windows-gnu"
      }
    }
  },
  "wincolor 1.0.2 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "termcolor 1.0.5 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": "cfg(windows)"
      }
    }
  },
  "ws2_32-sys 0.2.1 (registry+https://github.com/rust-lang/crates.io-index)": {
    "by": {
      "miow 0.2.1 (registry+https://github.com/rust-lang/crates.io-index)": {
        "optional": false,
        "platform": null
      }
    }
  }
}

```

### `lib`

```rust
use cargo_unused::{CargoMetadata, CargoUnused, ExecutableTarget};

let ctrl_c = tokio_signal::ctrl_c();
let mut ctrl_c = tokio::runtime::current_thread::Runtime::new()?.block_on(ctrl_c)?;

let metadata = CargoMetadata::new()
    .cargo(Some("cargo"))
    .manifest_path(Some("./Cargo.toml"))
    .cwd(Some("."))
    .ctrl_c(Some(&mut ctrl_c))
    .run()?;

let cargo_unused::Outcome { used, unused } = CargoUnused::new(&metadata)
    .target(Some(ExecutableTarget::Bin("main".to_owned())))
    .cargo(Some("cargo"))
    .debug(true)
    .ctrl_c(Some(&mut ctrl_c))
    .run()?;
```

## License

Licensed under <code>[MIT](https://opensource.org/licenses/MIT) OR [Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0)</code>.
