use cargo::core::PackageId;
use miniserde::ser::Fragment;

use std::borrow::{Borrow, Cow};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::slice;

impl miniserde::Serialize for crate::Cache {
    fn begin(&self) -> Fragment {
        self.0.begin()
    }
}

impl miniserde::Serialize for crate::CacheValue {
    fn begin(&self) -> Fragment {
        struct Map<V1, V2> {
            key: V1,
            used_packages: V2,
            pos: usize,
        }

        impl<V1: miniserde::Serialize, V2: miniserde::Serialize> miniserde::ser::Map for Map<V1, V2> {
            fn next(&mut self) -> Option<(Cow<str>, &dyn miniserde::Serialize)> {
                match self.pos {
                    0 => {
                        self.pos = 1;
                        Some(("key".into(), &self.key))
                    }
                    1 => {
                        self.pos = 2;
                        Some(("used_packages".into(), &self.used_packages))
                    }
                    _ => None,
                }
            }
        }

        Fragment::Map(Box::new(Map {
            key: &self.key,
            used_packages: self
                .used_packages
                .iter()
                .map(|(k, v)| (unwrap_to_string_with_serde(k), v))
                .collect::<BTreeMap<_, _>>(),
            pos: 0,
        }))
    }
}

impl miniserde::Serialize for crate::CacheUsedPackages {
    fn begin(&self) -> Fragment {
        struct Map<V> {
            lib: V,
            bin: V,
            test: V,
            bench: V,
            example_lib: V,
            example_bin: V,
            custom_build: V,
            pos: usize,
        }

        impl<V: miniserde::Serialize> miniserde::ser::Map for Map<V> {
            fn next(&mut self) -> Option<(Cow<str>, &dyn miniserde::Serialize)> {
                match self.pos {
                    0 => {
                        self.pos = 1;
                        Some(("lib".into(), &self.lib))
                    }
                    1 => {
                        self.pos = 2;
                        Some(("bin".into(), &self.bin))
                    }
                    2 => {
                        self.pos = 3;
                        Some(("test".into(), &self.test))
                    }
                    3 => {
                        self.pos = 4;
                        Some(("bench".into(), &self.bench))
                    }
                    4 => {
                        self.pos = 5;
                        Some(("example_lib".into(), &self.example_lib))
                    }
                    5 => {
                        self.pos = 6;
                        Some(("example_bin".into(), &self.example_bin))
                    }
                    6 => {
                        self.pos = 7;
                        Some(("custom_build".into(), &self.custom_build))
                    }
                    _ => None,
                }
            }
        }

        Fragment::Map(Box::new(Map {
            lib: miniser_to_string_package_ids_map(&self.lib),
            bin: miniser_to_string_package_ids_map(&self.bin),
            test: miniser_to_string_package_ids_map(&self.test),
            bench: miniser_to_string_package_ids_map(&self.bench),
            example_lib: miniser_to_string_package_ids_map(&self.example_lib),
            example_bin: miniser_to_string_package_ids_map(&self.example_bin),
            custom_build: miniser_to_string_package_ids_map(&self.custom_build),
            pos: 0,
        }))
    }
}

impl miniserde::Serialize for crate::LinkedPackages {
    fn begin(&self) -> Fragment {
        struct Map<V1, V2> {
            used: V1,
            unused: V2,
            pos: usize,
        }

        impl<V1: miniserde::Serialize, V2: miniserde::Serialize> miniserde::ser::Map for Map<V1, V2> {
            fn next(&mut self) -> Option<(Cow<str>, &dyn miniserde::Serialize)> {
                match self.pos {
                    0 => {
                        self.pos = 1;
                        Some(("used".into(), &self.used))
                    }
                    1 => {
                        self.pos = 2;
                        Some(("unused".into(), &self.unused))
                    }
                    _ => None,
                }
            }
        }

        Fragment::Map(Box::new(Map {
            used: miniser_package_ids(&self.used),
            unused: &self.unused,
            pos: 0,
        }))
    }
}

impl miniserde::Serialize for crate::LinkedPackagesUnused {
    fn begin(&self) -> Fragment {
        struct Map<V> {
            trivial: V,
            maybe_obsolete: V,
            pos: usize,
        }

        impl<V: miniserde::Serialize> miniserde::ser::Map for Map<V> {
            fn next(&mut self) -> Option<(Cow<str>, &dyn miniserde::Serialize)> {
                match self.pos {
                    0 => {
                        self.pos = 1;
                        Some(("trivial".into(), &self.trivial))
                    }
                    1 => {
                        self.pos = 2;
                        Some(("maybe_obsolete".into(), &self.maybe_obsolete))
                    }
                    _ => None,
                }
            }
        }

        Fragment::Map(Box::new(Map {
            trivial: miniser_package_ids(&self.trivial),
            maybe_obsolete: miniser_package_ids(&self.maybe_obsolete),
            pos: 0,
        }))
    }
}

fn miniser_package_ids<P: Borrow<PackageId>, I: IntoIterator<Item = P>>(
    ids: I,
) -> impl miniserde::Serialize + 'static {
    struct Serializer(Vec<String>);

    impl miniserde::Serialize for Serializer {
        fn begin(&self) -> Fragment {
            Fragment::Seq(Box::new(Seq(self.0.iter())))
        }
    }

    struct Seq<'a>(slice::Iter<'a, String>);

    impl<'a> miniserde::ser::Seq for Seq<'a> {
        fn next(&mut self) -> Option<&dyn miniserde::Serialize> {
            self.0.next().map(|s| s as &dyn miniserde::Serialize)
        }
    }

    Serializer(
        ids.into_iter()
            .map(|id| unwrap_to_string_with_serde(id.borrow()))
            .collect(),
    )
}

fn miniser_to_string_package_ids_map<
    M: IntoIterator<Item = (K, V)>,
    K: ToString,
    V: IntoIterator<Item = P>,
    P: Borrow<PackageId>,
>(
    map: M,
) -> impl miniserde::Serialize {
    struct Serializer<K>(Vec<(K, Vec<String>)>);

    impl<K: ToString> miniserde::Serialize for Serializer<K> {
        fn begin(&self) -> Fragment {
            Fragment::Map(Box::new(Map(self.0.iter())))
        }
    }

    struct Map<'a, K>(slice::Iter<'a, (K, Vec<String>)>);

    impl<'a, K: ToString> miniserde::ser::Map for Map<'a, K> {
        fn next(&mut self) -> Option<(Cow<str>, &dyn miniserde::Serialize)> {
            self.0
                .next()
                .map(|(k, v)| (k.to_string().into(), v as &dyn miniserde::Serialize))
        }
    }

    Serializer(
        map.into_iter()
            .map(|(key, val)| {
                let val = val
                    .into_iter()
                    .map(|p| unwrap_to_string_with_serde(p.borrow()))
                    .collect();
                (key, val)
            })
            .collect(),
    )
}

/// # Panics
///
/// Panics if `item` is not converted into `serde_json::Value::String`.
fn unwrap_to_string_with_serde(item: impl Debug + serde::Serialize) -> String {
    serde_json::to_value(&item)
        .ok()
        .and_then(|value| match value {
            serde_json::Value::String(value) => Some(value),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "{:?} cannot be converted into `serde_json::Value::String`",
                item,
            )
        })
}
