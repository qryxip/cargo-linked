use cargo_metadata::PackageId;
use indexmap::{IndexMap, IndexSet};
use miniserde::ser::Fragment;

use std::borrow::Cow;
use std::hash::Hash;

impl miniserde::Serialize for crate::Outcome {
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

        Fragment::Map(Box::new(Map {
            used: miniser_package_id_set(&self.used),
            unused: miniser_indexmap(&self.unused),
            pos: 0,
        }))
    }
}

impl miniserde::Serialize for crate::OutcomeUnused {
    fn begin(&self) -> Fragment {
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

        Fragment::Map(Box::new(Map {
            by: miniser_indexmap(&self.by),
            pos: 0,
        }))
    }
}

impl miniserde::Serialize for crate::CacheByMode {
    fn begin(&self) -> Fragment {
        struct Map<V1, V2> {
            targets: V1,
            dependencies: V2,
            pos: usize,
        }

        impl<V1: miniserde::Serialize, V2: miniserde::Serialize> miniserde::ser::Map for Map<V1, V2> {
            fn next(&mut self) -> Option<(Cow<str>, &dyn miniserde::Serialize)> {
                match self.pos {
                    0 => {
                        self.pos = 1;
                        Some(("targets".into(), &self.targets))
                    }
                    1 => {
                        self.pos = 2;
                        Some(("dependencies".into(), &self.dependencies))
                    }
                    _ => None,
                }
            }
        }

        Fragment::Map(Box::new(Map {
            targets: miniser_to_string_package_id_indexset_map(&self.targets),
            dependencies: miniser_to_string_package_id_indexset_map(&self.dependencies),
            pos: 0,
        }))
    }
}

fn miniser_package_id_set<'a>(set: &'a IndexSet<PackageId>) -> impl miniserde::Serialize + 'a {
    struct Serializer<'a>(indexmap::set::Iter<'a, PackageId>);

    impl<'a> miniserde::Serialize for Serializer<'a> {
        fn begin(&self) -> Fragment {
            Fragment::Seq(Box::new(Seq(self.0.clone())))
        }
    }

    struct Seq<'a>(indexmap::set::Iter<'a, PackageId>);

    impl<'a> miniserde::ser::Seq for Seq<'a> {
        fn next(&mut self) -> Option<&dyn miniserde::Serialize> {
            self.0
                .next()
                .map(|id| &id.repr as &dyn miniserde::Serialize)
        }
    }

    Serializer(set.iter())
}

fn miniser_indexmap<'a>(
    map: &'a IndexMap<impl Eq + Hash + ToString, impl miniserde::Serialize>,
) -> impl miniserde::Serialize + 'a {
    struct Serializer<'a, K, V>(&'a IndexMap<K, V>);

    impl<'a, K: Eq + Hash + ToString, V: miniserde::Serialize> miniserde::Serialize
        for Serializer<'a, K, V>
    {
        fn begin(&self) -> Fragment {
            Fragment::Map(Box::new(Map(self.0.iter())))
        }
    }

    struct Map<'a, K, V>(indexmap::map::Iter<'a, K, V>);

    impl<'a, K: ToString, V: miniserde::Serialize> miniserde::ser::Map for Map<'a, K, V> {
        fn next(&mut self) -> Option<(Cow<str>, &dyn miniserde::Serialize)> {
            self.0
                .next()
                .map(|(k, v)| (k.to_string().into(), v as &dyn miniserde::Serialize))
        }
    }

    Serializer(map)
}

fn miniser_to_string_package_id_indexset_map<
    'a,
    K: ToString + 'a,
    I: IntoIterator<Item = (&'a K, &'a IndexSet<PackageId>)>,
>(
    map: I,
) -> impl miniserde::Serialize + 'a {
    struct Serializer<V>(Vec<(String, V)>);

    impl<V: miniserde::Serialize> miniserde::Serialize for Serializer<V> {
        fn begin(&self) -> Fragment {
            Fragment::Map(Box::new(Map {
                slice: &self.0,
                pos: 0,
            }))
        }
    }

    struct Map<'a, V> {
        slice: &'a [(String, V)],
        pos: usize,
    }

    impl<'a, V: miniserde::Serialize> miniserde::ser::Map for Map<'a, V> {
        fn next(&mut self) -> Option<(Cow<str>, &dyn miniserde::Serialize)> {
            if let Some((key, val)) = self.slice.get(self.pos) {
                self.pos += 1;
                Some((key.into(), val as &dyn miniserde::Serialize))
            } else {
                None
            }
        }
    }

    Serializer(
        map.into_iter()
            .map(|(k, v)| (k.to_string(), miniser_package_id_set(v)))
            .collect(),
    )
}
