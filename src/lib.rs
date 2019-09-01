//! Find unused crates.
//!
//! ```no_run
//! use cargo_unused::{CargoMetadata, CargoUnused, ExecutableTarget};
//!
//! let ctrl_c = tokio_signal::ctrl_c();
//! let mut ctrl_c = tokio::runtime::current_thread::Runtime::new()?.block_on(ctrl_c)?;
//!
//! let metadata = CargoMetadata::new()
//!     .cargo(Some("cargo"))
//!     .manifest_path(Some("./Cargo.toml"))
//!     .cwd(Some("."))
//!     .ctrl_c(Some(&mut ctrl_c))
//!     .run()?;
//!
//! let cargo_unused::Outcome { used, unused } = CargoUnused::new(&metadata)
//!     .target(Some(ExecutableTarget::Bin("main".to_owned())))
//!     .cargo(Some("cargo"))
//!     .debug(true)
//!     .ctrl_c(Some(&mut ctrl_c))
//!     .run()?;
//! # failure::Fallible::Ok(())
//! ```

macro_rules! lazy_regex {
    ($regex:expr $(,)?) => {
        ::once_cell::sync::Lazy::new(|| ::regex::Regex::new($regex).unwrap())
    };
}

mod error;
mod fs;
mod parse;
mod process;
mod ser;

#[doc(inline)]
pub use crate::error::{Error, ErrorKind};

pub use cargo_metadata as cargo_metadata_0_8;
pub use indexmap as indexmap_1;
pub use log as log_0_4;
pub use miniserde as miniserde_0_1;
pub use serde as serde_1;
pub use tokio_signal as tokio_signal_0_2;

use crate::fs::ExclusivelyLockedJsonFile;
use crate::process::Rustc;

use cargo_metadata::{Metadata, Node, NodeDep, Package, PackageId};
use fixedbitset::FixedBitSet;
use if_chain::if_chain;
use indexmap::{indexset, IndexMap, IndexSet};
use log::{info, warn};
use maplit::{hashmap, hashset};
use once_cell::sync::Lazy;
use regex::Regex;
use semver::Version;
use smallvec::SmallVec;

use std::borrow::{BorrowMut, Cow};
use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::{OsStr, OsString};
use std::ops::Deref;
use std::path::{Path, PathBuf};

/// Result.
pub type Result<T> = std::result::Result<T, crate::Error>;

/// Outcome.
#[derive(serde::Serialize)]
pub struct Outcome {
    pub used: IndexSet<PackageId>,
    pub unused: IndexMap<PackageId, OutcomeUnused>,
}

impl miniserde::Serialize for Outcome {
    fn begin(&self) -> miniserde::ser::Fragment {
        struct Map<V1, V2> {
            used: V1,
            unused: V2,
            pos: usize,
        }

        impl<V1: miniserde::Serialize, V2: miniserde::Serialize> miniserde::ser::Map for Map<V1, V2> {
            fn next(&mut self) -> Option<(Cow<str>, &dyn miniserde::Serialize)> {
                match self.pos {
                    0 => {
                        self.pos += 1;
                        Some((
                            Cow::Borrowed("used"),
                            &self.used as &dyn miniserde::Serialize,
                        ))
                    }
                    1 => {
                        self.pos += 1;
                        Some((
                            Cow::Borrowed("unused"),
                            &self.unused as &dyn miniserde::Serialize,
                        ))
                    }
                    _ => None,
                }
            }
        }

        miniserde::ser::Fragment::Map(Box::new(Map {
            used: crate::ser::miniser_package_id_set(&self.used),
            unused: crate::ser::miniser_package_id_x_indexmap(&self.unused),
            pos: 0,
        }))
    }
}
#[derive(Debug, serde::Serialize)]
pub struct OutcomeUnused {
    pub by: IndexMap<PackageId, OutcomeUnusedBy>,
}

impl miniserde::Serialize for OutcomeUnused {
    fn begin(&self) -> miniserde::ser::Fragment {
        struct Map<V> {
            by: V,
            pos: usize,
        }

        impl<V: miniserde::Serialize> miniserde::ser::Map for Map<V> {
            fn next(&mut self) -> Option<(Cow<str>, &dyn miniserde::Serialize)> {
                if self.pos == 0 {
                    self.pos = 1;
                    Some(("by".into(), &self.by))
                } else {
                    None
                }
            }
        }

        miniserde::ser::Fragment::Map(Box::new(Map {
            by: crate::ser::miniser_package_id_x_indexmap(&self.by),
            pos: 0,
        }))
    }
}

#[derive(Debug, miniserde::Serialize, serde::Serialize)]
pub struct OutcomeUnusedBy {
    pub optional: bool,
    pub platform: Option<String>,
}

/// `bin`, `example`, `test`, or `bench`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ExecutableTarget {
    Bin(String),
    Example(String),
    Test(String),
    Bench(String),
}

impl ExecutableTarget {
    pub fn try_from_options(
        bin: &Option<String>,
        example: &Option<String>,
        test: &Option<String>,
        bench: &Option<String>,
    ) -> Option<Self> {
        if let Some(bin) = bin {
            Some(Self::Bin(bin.clone()))
        } else if let Some(example) = example {
            Some(Self::Example(example.clone()))
        } else if let Some(test) = test {
            Some(Self::Test(test.clone()))
        } else if let Some(bench) = bench {
            Some(Self::Bench(bench.clone()))
        } else {
            None
        }
    }
}

/// Executes `cargo metadata`.
///
/// # Example
///
/// ```no_run
/// use cargo_unused::CargoMetadata;
///
/// let ctrl_c = tokio_signal::ctrl_c();
/// let mut ctrl_c = tokio::runtime::current_thread::Runtime::new()?.block_on(ctrl_c)?;
///
/// let metadata = CargoMetadata::new()
///     .cargo(Some("cargo"))
///     .manifest_path(Some("./Cargo.toml"))
///     .cwd(Some("."))
///     .ctrl_c(Some(&mut ctrl_c))
///     .run()?;
/// # failure::Fallible::Ok(())
/// ```
pub struct CargoMetadata<'a> {
    cargo: Option<OsString>,
    manifest_path: Option<PathBuf>,
    cwd: Option<PathBuf>,
    ctrl_c: Option<&'a mut tokio_signal::IoStream<()>>,
}

impl CargoMetadata<'static> {
    /// Constructs a new `CargoMetadata`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl CargoMetadata<'_> {
    /// Sets `cargo`.
    pub fn cargo<S: AsRef<OsStr>>(self, cargo: Option<S>) -> Self {
        Self {
            cargo: cargo.map(|s| s.as_ref().to_owned()),
            ..self
        }
    }

    /// Sets `manifest_path`.
    pub fn manifest_path<P: AsRef<Path>>(self, manifest_path: Option<P>) -> Self {
        Self {
            manifest_path: manifest_path.map(|p| p.as_ref().to_owned()),
            ..self
        }
    }

    /// Sets `cwd`.
    pub fn cwd<P: AsRef<Path>>(self, cwd: Option<P>) -> Self {
        Self {
            cwd: cwd.map(|p| p.as_ref().to_owned()),
            ..self
        }
    }

    /// Sets `ctrl_c`.
    pub fn ctrl_c<'a>(
        self,
        ctrl_c: Option<&'a mut tokio_signal::IoStream<()>>,
    ) -> CargoMetadata<'a> {
        CargoMetadata {
            cargo: self.cargo,
            manifest_path: self.manifest_path,
            cwd: self.cwd,
            ctrl_c,
        }
    }

    /// Executes `cargo metadata`.
    pub fn run(self) -> crate::Result<Metadata> {
        let mut rt =
            tokio::runtime::current_thread::Runtime::new().unwrap_or_else(|_| unimplemented!());
        let cargo = self
            .cargo
            .clone()
            .or_else(|| env::var_os("CARGO").map(Into::into))
            .ok_or_else(|| crate::ErrorKind::CargoEnvVarNotPresent)?;
        crate::process::cargo_metadata(&cargo, self.manifest_path, self.cwd, &mut rt, self.ctrl_c)
    }
}

impl Default for CargoMetadata<'static> {
    fn default() -> Self {
        Self {
            cargo: None,
            manifest_path: None,
            cwd: None,
            ctrl_c: None,
        }
    }
}

/// Finds unused packages.
///
/// # Example
///
/// ```no_run
/// use cargo_unused::{CargoMetadata, CargoUnused, ExecutableTarget};
///
/// let ctrl_c = tokio_signal::ctrl_c();
/// let mut ctrl_c = tokio::runtime::current_thread::Runtime::new()?.block_on(ctrl_c)?;
///
/// let metadata = CargoMetadata::new()
///     .cargo(Some("cargo"))
///     .manifest_path(Some("./Cargo.toml"))
///     .cwd(Some("."))
///     .ctrl_c(Some(&mut ctrl_c))
///     .run()?;
///
/// let cargo_unused::Outcome { used, unused } = CargoUnused::new(&metadata)
///     .target(Some(ExecutableTarget::Bin("main".to_owned())))
///     .cargo(Some("cargo"))
///     .debug(true)
///     .ctrl_c(Some(&mut ctrl_c))
///     .run()?;
/// # failure::Fallible::Ok(())
/// ```
pub struct CargoUnused<'a, 'b> {
    metadata: &'a Metadata,
    target: Option<ExecutableTarget>,
    cargo: Option<PathBuf>,
    debug: bool,
    ctrl_c: Option<&'b mut tokio_signal::IoStream<()>>,
}

impl<'a> CargoUnused<'a, 'static> {
    /// Constructs a new `CargoUnused`.
    pub fn new(metadata: &'a Metadata) -> Self {
        Self {
            metadata,
            target: None,
            cargo: None,
            debug: false,
            ctrl_c: None,
        }
    }
}

impl<'a> CargoUnused<'a, '_> {
    /// Sets `target`.
    pub fn target(self, target: Option<ExecutableTarget>) -> Self {
        Self { target, ..self }
    }

    /// Sets `cargo`.
    pub fn cargo<P: AsRef<Path>>(self, cargo: Option<P>) -> Self {
        let cargo = cargo.map(|p| p.as_ref().to_owned());
        Self { cargo, ..self }
    }

    /// Sets `debug`.
    pub fn debug(self, debug: bool) -> Self {
        Self { debug, ..self }
    }

    /// Sets `ctrl_c`.
    pub fn ctrl_c<'b>(
        self,
        ctrl_c: Option<&'b mut tokio_signal::IoStream<()>>,
    ) -> CargoUnused<'a, 'b> {
        CargoUnused {
            metadata: self.metadata,
            target: self.target,
            cargo: self.cargo,
            debug: self.debug,
            ctrl_c,
        }
    }

    /// Executes.
    pub fn run(self) -> crate::Result<Outcome> {
        let metadata = self.metadata;
        let target = self.target.as_ref();
        let debug = self.debug;
        let cargo = self
            .cargo
            .clone()
            .or_else(|| env::var_os("CARGO").map(Into::into))
            .ok_or_else(|| crate::ErrorKind::CargoEnvVarNotPresent)?;

        let mut ctrl_c = self.ctrl_c;
        let mut rt =
            tokio::runtime::current_thread::Runtime::new().unwrap_or_else(|_| unimplemented!());

        let packages = metadata
            .packages
            .iter()
            .map(|p| (&p.id, p))
            .collect::<HashMap<_, _>>();

        let resolve = metadata
            .resolve
            .as_ref()
            .ok_or(crate::ErrorKind::ResolveNotPresent)?;
        let root = resolve
            .root
            .as_ref()
            .ok_or(crate::ErrorKind::RootNotFound)?;

        let nodes = resolve
            .nodes
            .iter()
            .map(|n| (&n.id, n))
            .collect::<HashMap<_, _>>();

        let conds = {
            let mut conds = hashmap!();

            for package in &metadata.packages {
                let renamed = package
                    .dependencies
                    .iter()
                    .flat_map(|d| d.rename.as_ref().map(|r| (r, d)))
                    .collect::<HashMap<_, _>>();
                let unrenamed = package
                    .dependencies
                    .iter()
                    .filter(|d| d.rename.is_none())
                    .map(|d| (d.name.as_str(), d))
                    .collect::<HashMap<_, _>>();
                let node = &nodes[&package.id];

                for dep in &node.deps {
                    let dependency = if let Some(dependency) = renamed.get(&dep.name) {
                        dependency
                    } else {
                        static NAME: Lazy<Regex> = lazy_regex!(r"\A([a-zA-Z0-9_-]+).*\z");
                        let name = &NAME
                            .captures(&dep.pkg.repr)
                            .unwrap_or_else(|| unimplemented!())[1];
                        unrenamed.get(name).unwrap_or_else(|| unimplemented!())
                    };

                    let value = OutcomeUnusedBy {
                        optional: dependency.optional,
                        platform: dependency.target.as_ref().map(ToString::to_string),
                    };

                    conds
                        .entry(&dep.pkg)
                        .or_insert_with(IndexMap::new)
                        .insert(package.id.clone(), value);
                }
            }

            for value in conds.values_mut() {
                fn ordkey<'a>(package: &'a Package) -> impl Ord + 'a {
                    (&package.name, &package.version, &package.id)
                }

                value.sort_by(|id1, _, id2, _| ordkey(&packages[id1]).cmp(&ordkey(&packages[id2])));
            }

            conds
        };

        let src_paths = {
            let mut src_paths = metadata
                .packages
                .iter()
                .map(|package| {
                    let values = package
                        .targets
                        .iter()
                        .filter(|target| {
                            target
                                .kind
                                .iter()
                                .any(|k| ["lib", "proc-macro", "custom-build"].contains(&k.deref()))
                        })
                        .map(|t| &*t.src_path)
                        .collect();
                    (&package.id, values)
                })
                .collect::<HashMap<_, SmallVec<[_; 1]>>>();
            let root_bin_src_path = match target {
                Some(target) => {
                    let (name, kind) = match target {
                        ExecutableTarget::Bin(name) => (name, "bin"),
                        ExecutableTarget::Example(name) => (name, "example"),
                        ExecutableTarget::Test(name) => (name, "test"),
                        ExecutableTarget::Bench(name) => (name, "bench"),
                    };
                    &*packages[root]
                        .targets
                        .iter()
                        .find(|t| t.name == *name && t.kind.contains(&kind.to_owned()))
                        .ok_or_else(|| {
                            let name = name.clone();
                            crate::ErrorKind::NoSuchTarget { kind, name }
                        })?
                        .src_path
                }
                None => {
                    let bins = packages[root]
                        .targets
                        .iter()
                        .filter(|t| t.kind.iter().any(|k| k == "bin"))
                        .collect::<Vec<_>>();
                    if bins.len() == 1 {
                        &bins[0].src_path
                    } else {
                        let default_run =
                            crate::fs::read_toml::<CargoToml>(&packages[root].manifest_path)?
                                .package
                                .default_run
                                .ok_or_else(|| crate::ErrorKind::AmbiguousTarget)?;
                        &*packages[root]
                            .targets
                            .iter()
                            .find(|t| t.name == default_run && t.kind.contains(&"bin".to_owned()))
                            .ok_or_else(|| {
                                let (kind, name) = ("bin", default_run.clone());
                                crate::ErrorKind::NoSuchTarget { kind, name }
                            })?
                            .src_path
                    }
                }
            };
            src_paths
                .entry(root)
                .and_modify(|ps| ps.push(root_bin_src_path))
                .or_insert_with(|| [root_bin_src_path].into());

            src_paths
        };

        let root_manifest_dir = packages[root]
            .manifest_path
            .parent()
            .unwrap_or(&packages[root].manifest_path);

        let target_dir = root_manifest_dir
            .join("target")
            .join("cargo_unused")
            .join("target");
        let target_dir_with_mode = root_manifest_dir
            .join("target")
            .join("cargo_unused")
            .join("target")
            .join(if debug { "debug" } else { "release" });
        let target_dir_with_mode_bk = target_dir_with_mode.with_extension("bk");

        let rustcs = {
            let stderr = process::cargo_build_vv(
                &cargo,
                target,
                &target_dir,
                root_manifest_dir,
                debug,
                &mut rt,
                ctrl_c.as_mut().map(BorrowMut::borrow_mut),
            )?;
            let rustc = cargo
                .with_file_name("rustc")
                .with_extension(cargo.extension().unwrap_or_default());
            parse::cargo_build_vv_stderr_to_opts_and_envs(&stderr)?
                .into_iter()
                .map(|(envs, opts)| {
                    let rustc = Rustc::new(rustc.as_ref(), opts, envs, &metadata.workspace_root);
                    (rustc.input_abs().to_owned(), rustc)
                })
                .collect::<HashMap<_, _>>()
        };

        crate::fs::move_dir_with_timestamps(&target_dir_with_mode, &target_dir_with_mode_bk)?;
        crate::fs::copy_dir(
            &target_dir_with_mode_bk,
            &target_dir_with_mode,
            &fs_extra::dir::CopyOptions {
                overwrite: false,
                skip_exist: true,
                buffer_size: 64 * 1024,
                copy_inside: true,
                depth: 16,
            },
        )?;

        let result = Context {
            rt,
            ctrl_c,
            debug,
            packages: &packages,
            root,
            nodes: &nodes,
            conds,
            src_paths: &src_paths,
            root_manifest_dir,
            rustcs: &rustcs,
        }
        .run();

        crate::fs::remove_dir_all(&target_dir_with_mode)?;
        crate::fs::move_dir_with_timestamps(&target_dir_with_mode_bk, &target_dir_with_mode)?;

        return result;

        struct Context<'a, 'b> {
            rt: tokio::runtime::current_thread::Runtime,
            ctrl_c: Option<&'b mut tokio_signal::IoStream<()>>,
            debug: bool,
            packages: &'b HashMap<&'a PackageId, &'a Package>,
            root: &'a PackageId,
            nodes: &'b HashMap<&'a PackageId, &'a Node>,
            conds: HashMap<&'a PackageId, IndexMap<PackageId, OutcomeUnusedBy>>,
            src_paths: &'b HashMap<&'a PackageId, SmallVec<[&'a Path; 1]>>,
            root_manifest_dir: &'a Path,
            rustcs: &'b HashMap<PathBuf, Rustc>,
        }

        impl Context<'_, '_> {
            fn run(self) -> crate::Result<Outcome> {
                let Context {
                    mut rt,
                    mut ctrl_c,
                    debug,
                    packages,
                    root,
                    nodes,
                    mut conds,
                    src_paths,
                    root_manifest_dir,
                    rustcs,
                } = self;

                let cache_path = root_manifest_dir
                    .join("target")
                    .join("cargo_unused")
                    .join("cache.json");
                let mut cache_file = ExclusivelyLockedJsonFile::<Cache>::open(&cache_path)?;
                let mut cache = cache_file.read()?;

                let mut used = hashset!(root.clone());
                let mut cur = hashset!(root.clone());

                while !cur.is_empty() {
                    let mut next = hashset!();
                    for cur in cur {
                        if src_paths[&cur].iter().any(|&p| rustcs.contains_key(p)) {
                            cache.get_mut(debug).remove(&cur);
                        }
                        match cache.entry(debug, &cur) {
                            indexmap::map::Entry::Occupied(cache) => {
                                for id in cache.get() {
                                    if used.insert(id.clone()) {
                                        next.insert(id.clone());
                                    }
                                }
                            }
                            indexmap::map::Entry::Vacant(cache) => {
                                let cache = cache.insert(indexset!());
                                for &src_path in &src_paths[&cur] {
                                    let rustc = rustcs.get(src_path).ok_or_else(|| {
                                        crate::ErrorKind::UnexpectedSrcPath {
                                            src_path: src_path.to_owned(),
                                        }
                                    })?;
                                    let output = filter_actually_used_crates(
                                        rustc,
                                        &nodes[&cur].deps,
                                        &mut rt,
                                        ctrl_c.as_mut().map(BorrowMut::borrow_mut),
                                    )?;
                                    for &output in &output {
                                        if used.insert(output.clone()) {
                                            next.insert(output.clone());
                                        }
                                        cache.insert(output.clone());
                                    }
                                }
                            }
                        }
                    }
                    cur = next;
                }

                cache.sort(&packages);
                cache_file.write(&cache)?;

                let mut used = used.into_iter().collect::<Vec<_>>();
                let mut unused = packages
                    .keys()
                    .map(|&id| id.clone())
                    .filter(|id| !used.contains(&id))
                    .collect::<Vec<_>>();
                for list in &mut [&mut used, &mut unused] {
                    list.sort_by(|a, b| {
                        let a = (&packages[a].name, &packages[a].version, a);
                        let b = (&packages[b].name, &packages[b].version, b);
                        a.cmp(&b)
                    })
                }

                Ok(Outcome {
                    used: used.into_iter().collect(),
                    unused: unused
                        .into_iter()
                        .map(|unused| {
                            let value = OutcomeUnused {
                                by: conds.remove(&unused).unwrap_or_default(),
                            };
                            (unused, value)
                        })
                        .collect(),
                })
            }
        }

        fn filter_actually_used_crates<'a>(
            rustc: &Rustc,
            deps: &'a [NodeDep],
            mut rt: &mut tokio::runtime::current_thread::Runtime,
            mut ctrl_c: Option<&mut tokio_signal::IoStream<()>>,
        ) -> crate::Result<HashSet<&'a PackageId>> {
            #[derive(serde::Deserialize)]
            struct ErrorMessage {
                message: String,
                code: Option<ErrorMessageCode>,
            }

            #[derive(serde::Deserialize)]
            struct ErrorMessageCode {
                code: String,
            }

            let mut exclusion = FixedBitSet::with_capacity(rustc.externs().len());
            exclusion.insert_range(0..rustc.externs().len());

            let something_wrong = 'run: loop {
                static E0432_SINGLE_MOD: Lazy<Regex> =
                    lazy_regex!(r"\Aunresolved import `([a-zA-Z0-9_]+)`\z");
                static E0433_SINGLE_MOD: Lazy<Regex> = lazy_regex!(
                    r"\Afailed to resolve: [a-z ]+`([a-zA-Z0-9_]+)`( in `\{\{root\}\}`)?\z",
                );
                static E0463_SINGLE_MOD: Lazy<Regex> =
                    lazy_regex!(r"\Acan't find crate for `([a-zA-Z0-9_]+)`\z");

                let (success, stderr) = rustc.run(
                    &exclusion,
                    &mut rt,
                    ctrl_c.as_mut().map(BorrowMut::borrow_mut),
                )?;

                if success {
                    break false;
                } else {
                    let mut updated = false;
                    let mut num_e0432 = 0;
                    let mut num_e0433 = 0;
                    let mut num_e0463 = 0;
                    let mut num_others = 0;

                    for line in stderr.lines() {
                        let msg = crate::fs::from_json::<ErrorMessage>(line, "an error message")?;

                        if_chain! {
                            if let Some(code) = &msg.code;
                            if let Some(regex) = match &*code.code {
                                "E0432" => {
                                    num_e0432 += 1;
                                    Some(&E0432_SINGLE_MOD)
                                }
                                "E0433" => {
                                    num_e0433 += 1;
                                    Some(&E0433_SINGLE_MOD)
                                }
                                "E0463" => {
                                    num_e0463 += 1;
                                    Some(&E0463_SINGLE_MOD)
                                }
                                "E0658" => {
                                    warn!("Found E0658. Trying to exclude crates one by one");
                                    break 'run true;
                                }
                                _ => {
                                    num_others += 1;
                                    None
                                }
                            };
                            if let Some(caps) = regex.captures(&msg.message);
                            if let Some(pos) = rustc
                                .externs()
                                .iter()
                                .position(|e| *e.name() == caps[1]);
                            then {
                                updated |= exclusion[pos];
                                exclusion.set(pos, false);
                            }
                        }
                    }

                    info!(
                        "E0432: {}, E0433: {}, E0483: {}, other error(s): {}",
                        num_e0432, num_e0433, num_e0463, num_others,
                    );

                    if !updated {
                        warn!("Something is wrong. Trying to exclude crates one by one");
                        break true;
                    }
                }
            };

            if something_wrong {
                exclusion.clear();
                for i in 0..rustc.externs().len() {
                    exclusion.insert(i);
                    let (success, _) = rustc.run(
                        &exclusion,
                        &mut rt,
                        ctrl_c.as_mut().map(BorrowMut::borrow_mut),
                    )?;
                    exclusion.set(i, success);
                }
            }

            let deps = deps
                .iter()
                .map(|d| (&*d.name, &d.pkg))
                .collect::<HashMap<_, _>>();
            Ok(rustc
                .externs()
                .iter()
                .enumerate()
                .filter(|&(i, _)| !exclusion[i])
                .flat_map(|(_, e)| deps.get(&e.name()).cloned())
                .collect())
        }

        #[derive(serde::Deserialize)]
        struct CargoToml {
            package: CargoTomlPackage,
        }

        #[derive(serde::Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct CargoTomlPackage {
            default_run: Option<String>,
        }

        #[derive(Default, serde::Deserialize)]
        struct Cache {
            debug: IndexMap<PackageId, IndexSet<PackageId>>,
            release: IndexMap<PackageId, IndexSet<PackageId>>,
        }

        impl Cache {
            fn get_mut(&mut self, debug: bool) -> &mut IndexMap<PackageId, IndexSet<PackageId>> {
                if debug {
                    &mut self.debug
                } else {
                    &mut self.release
                }
            }

            fn entry<'a>(
                &'a mut self,
                debug: bool,
                key: &PackageId,
            ) -> indexmap::map::Entry<'a, PackageId, IndexSet<PackageId>> {
                self.get_mut(debug).entry(key.clone())
            }

            fn sort(&mut self, packages: &HashMap<&PackageId, &Package>) {
                fn sort(
                    map: &mut IndexMap<PackageId, IndexSet<PackageId>>,
                    packages: &HashMap<&PackageId, &Package>,
                ) {
                    for values in map.values_mut() {
                        values.sort_by(|x, y| ordkey(packages[x]).cmp(&ordkey(packages[y])));
                    }
                    map.sort_by(|x, _, y, _| ordkey(packages[x]).cmp(&ordkey(packages[y])));
                }

                fn ordkey(package: &Package) -> (&str, &Version, &PackageId) {
                    (&package.name, &package.version, &package.id)
                }

                sort(&mut self.debug, packages);
                sort(&mut self.release, packages);
            }
        }

        impl miniserde::Serialize for Cache {
            fn begin(&self) -> miniserde::ser::Fragment {
                struct Map<V: miniserde::Serialize> {
                    debug: V,
                    release: V,
                    pos: usize,
                }

                impl<V: miniserde::Serialize> miniserde::ser::Map for Map<V> {
                    fn next(&mut self) -> Option<(Cow<str>, &dyn miniserde::Serialize)> {
                        match self.pos {
                            0 => {
                                self.pos += 1;
                                Some((
                                    Cow::Borrowed("debug"),
                                    &self.debug as &dyn miniserde::Serialize,
                                ))
                            }
                            1 => {
                                self.pos += 1;
                                Some((
                                    Cow::Borrowed("release"),
                                    &self.release as &dyn miniserde::Serialize,
                                ))
                            }
                            _ => None,
                        }
                    }
                }

                miniserde::ser::Fragment::Map(Box::new(Map {
                    debug: crate::ser::miniser_package_id_package_id_set_map(&self.debug),
                    release: crate::ser::miniser_package_id_package_id_set_map(&self.release),
                    pos: 0,
                }))
            }
        }
    }
}
