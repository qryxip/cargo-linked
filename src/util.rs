use cargo::core::compiler::CompileMode;
use cargo::core::manifest::{Target, TargetKind};
use cargo::core::Workspace;
use cargo::ops::CompileOptions;
use cargo::util::command_prelude::ArgMatchesExt;
use cargo::{CargoResult, Config};
use maplit::hashmap;

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::ops::Deref;
use std::path::PathBuf;

pub struct Configure<'a, F: FnOnce(PathBuf) -> PathBuf> {
    pub manifest_path: &'a Option<PathBuf>,
    pub color: &'a Option<String>,
    pub frozen: bool,
    pub locked: bool,
    pub offline: bool,
    pub modify_target_dir: F,
}

impl<F: FnOnce(PathBuf) -> PathBuf> Configure<'_, F> {
    pub fn configure(self, config: &mut Config) -> CargoResult<()> {
        let Self {
            manifest_path,
            color,
            frozen,
            locked,
            offline,
            modify_target_dir,
        } = self;

        let mut args = hashmap!();
        if let Some(manifest_path) = manifest_path {
            args.insert("manifest-path", vec![manifest_path.into()]);
        }

        let target_dir = arg_matches_from(args)
            .workspace(&config)?
            .target_dir()
            .into_path_unlocked();
        let target_dir = modify_target_dir(target_dir);

        config.configure(
            0,
            None,
            color,
            frozen,
            locked,
            offline,
            &Some(target_dir),
            &[],
        )
    }
}

pub fn workspace<'a>(
    config: &'a Config,
    manifest_path: &Option<PathBuf>,
) -> CargoResult<Workspace<'a>> {
    let mut args = hashmap!();
    if let Some(manifest_path) = manifest_path {
        args.insert("manifest-path", vec![OsString::from(manifest_path)]);
    }
    arg_matches_from(args).workspace(config)
}

#[derive(Clone, Copy, Debug)]
pub struct CompileOptionsForSingleTarget<'a, 'b> {
    pub ws: &'a Workspace<'a>,
    pub jobs: &'b Option<String>,
    pub lib: bool,
    pub bin: &'b Option<String>,
    pub example: &'b Option<String>,
    pub test: &'b Option<String>,
    pub bench: &'b Option<String>,
    pub release: bool,
    pub features: &'b [String],
    pub all_features: bool,
    pub no_default_features: bool,
    pub manifest_path: &'b Option<PathBuf>,
}

impl<'a> CompileOptionsForSingleTarget<'a, '_> {
    pub fn compile_options_for_single_target(
        self,
    ) -> CargoResult<(CompileOptions<'a>, &'a Target)> {
        let Self {
            ws,
            jobs,
            lib,
            bin,
            example,
            test,
            bench,
            release,
            features,
            all_features,
            no_default_features,
            manifest_path,
        } = self;

        let mut args = hashmap!();
        if let Some(jobs) = jobs {
            args.insert("jobs", vec![jobs.into()]);
        }
        if !features.is_empty() {
            args.insert("features", features.iter().map(Into::into).collect());
        }
        if all_features {
            args.insert("all-features", vec![]);
        }
        if no_default_features {
            args.insert("no-default-features", vec![]);
        }
        if let Some(manifest_path) = manifest_path {
            args.insert("manifest-path", vec![manifest_path.into()]);
        }

        let current = ws.current()?;

        let find_by_name = |name: &str, kind: &'static str| -> _ {
            current
                .targets()
                .iter()
                .find(|t| t.name() == name && t.kind().description() == kind)
                .ok_or_else(|| failure::err_msg(format!("No such `{}`: {}", kind, name)))
        };

        if release {
            args.insert("release", vec![]);
        }

        let (arg_key, arg_val, target) = if lib {
            let target = current
                .targets()
                .iter()
                .find(|t| t.is_lib())
                .ok_or_else(|| {
                    failure::err_msg("Current workspace member does not contain `lib`")
                })?;
            ("lib", vec![], target)
        } else if let Some(bin) = bin {
            let target = find_by_name(bin, "bin")?;
            ("bin", vec![OsString::from(bin)], target)
        } else if let Some(test) = test {
            let target = find_by_name(test, "integration-test")?;
            ("test", vec![OsString::from(test)], target)
        } else if let Some(bench) = bench {
            let target = find_by_name(bench, "bench")?;
            ("bench", vec![OsString::from(bench)], target)
        } else if let Some(example) = example {
            let target = find_by_name(example, "example")?;
            ("example", vec![OsString::from(example)], target)
        } else {
            let bins = current
                .targets()
                .iter()
                .filter(|t| *t.kind() == TargetKind::Bin)
                .collect::<Vec<_>>();
            let target = if bins.len() == 1 {
                &bins[0]
            } else {
                let name = current
                    .manifest()
                    .default_run()
                    .ok_or_else(|| failure::err_msg("Could not determine which binary to run"))?;
                find_by_name(name, "bin")?
            };
            ("bin", vec![OsString::from(target.name())], target)
        };

        args.insert(arg_key, arg_val);

        let compile_options = arg_matches_from(args).compile_options(
            ws.config(),
            CompileMode::Check { test: false },
            Some(ws),
        )?;
        Ok((compile_options, target))
    }
}

fn arg_matches_from(map: HashMap<&'static str, Vec<OsString>>) -> impl ArgMatchesExt {
    struct DummyArgMatches(HashMap<&'static str, Vec<OsString>>);

    impl ArgMatchesExt for DummyArgMatches {
        fn _value_of(&self, name: &str) -> Option<&str> {
            let value = self
                .0
                .get(name)?
                .get(0)?
                .to_str()
                .expect("unexpected invalid UTF-8 code point");
            Some(value)
        }

        fn _value_of_os(&self, name: &str) -> Option<&OsStr> {
            self.0.get(name)?.get(0).map(Deref::deref)
        }

        fn _values_of(&self, name: &str) -> Vec<String> {
            self.0
                .get(name)
                .map(Deref::deref)
                .unwrap_or_default()
                .iter()
                .map(|value| {
                    value
                        .to_str()
                        .expect("unexpected invalid UTF-8 code point")
                        .to_owned()
                })
                .collect()
        }

        fn _values_of_os(&self, name: &str) -> Vec<OsString> {
            self.0
                .get(name)
                .map(Deref::deref)
                .unwrap_or_default()
                .iter()
                .map(Clone::clone)
                .collect()
        }

        fn _is_present(&self, name: &str) -> bool {
            self.0.contains_key(name)
        }
    }

    DummyArgMatches(map)
}
