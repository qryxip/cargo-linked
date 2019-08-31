use cargo_metadata::PackageId;
use indexmap::{IndexMap, IndexSet};

use std::borrow::Cow;
use std::ops::Deref as _;

pub(crate) fn miniser_package_id_set<'a>(
    set: &'a IndexSet<PackageId>,
) -> impl miniserde::Serialize + 'a {
    struct Serializer<'a>(indexmap::set::Iter<'a, PackageId>);

    impl<'a> miniserde::Serialize for Serializer<'a> {
        fn begin(&self) -> miniserde::ser::Fragment {
            miniserde::ser::Fragment::Seq(Box::new(Seq(self.0.clone())))
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

pub(crate) fn miniser_package_id_package_id_set_map<'a>(
    map: &'a IndexMap<PackageId, IndexSet<PackageId>>,
) -> impl miniserde::Serialize + 'a {
    struct Serializer<'a>(&'a IndexMap<PackageId, IndexSet<PackageId>>);

    impl<'a> miniserde::Serialize for Serializer<'a> {
        fn begin(&self) -> miniserde::ser::Fragment {
            miniserde::ser::Fragment::Map(Box::new(Map {
                pairs: self
                    .0
                    .iter()
                    .map(|(k, v)| (k.repr.deref(), miniser_package_id_set(v)))
                    .collect(),
                pos: 0,
            }))
        }
    }

    struct Map<'a, V: miniserde::Serialize + 'a> {
        pairs: Vec<(&'a str, V)>,
        pos: usize,
    }

    impl<'a, V: miniserde::Serialize + 'a> miniserde::ser::Map for Map<'a, V> {
        fn next(&mut self) -> Option<(Cow<str>, &dyn miniserde::Serialize)> {
            let (key, values) = self.pairs.get(self.pos)?;
            self.pos += 1;
            Some(((*key).into(), values))
        }
    }

    Serializer(map)
}
