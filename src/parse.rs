use cargo::core::manifest::TargetSourcePath;
use cargo::core::Edition;
use failure::Fallible;
use maplit::{btreeset, hashset};
use syn::visit::{self, Visit};
use syn::{Item, ItemMod, ItemUse, UseTree};

use std::collections::{BTreeSet, HashSet};
use std::path::Path;

pub(crate) fn find_uses_lossy<'a>(
    src: &TargetSourcePath,
    extern_crates: &HashSet<&'a str>,
    edition: Edition,
) -> Fallible<HashSet<&'a str>> {
    match edition {
        Edition::Edition2015 => find_uses_lossy_2015(src, extern_crates),
        Edition::Edition2018 => find_uses_lossy_2018(src, extern_crates),
    }
}

fn find_uses_lossy_2015<'a>(
    src: &TargetSourcePath,
    extern_crates: &HashSet<&'a str>,
) -> Fallible<HashSet<&'a str>> {
    let root_path = match src.path() {
        None => return Ok(hashset!()),
        Some(path) => path,
    };
    let file = crate::fs::read_src(root_path)?;
    if !file.attrs.is_empty() {
        return Ok(hashset!());
    }
    Ok(file
        .items
        .into_iter()
        .flat_map(|item| match item {
            Item::ExternCrate(item) => Some(item),
            _ => None,
        })
        .filter(|item| item.attrs.is_empty())
        .flat_map(|item| extern_crates.get(&*item.ident.to_string()).cloned())
        .collect())
}

fn find_uses_lossy_2018<'a>(
    src: &TargetSourcePath,
    extern_crates: &HashSet<&'a str>,
) -> Fallible<HashSet<&'a str>> {
    struct Visitor<'a, 'b> {
        extern_crates: &'b HashSet<&'a str>,
        used: HashSet<&'a str>,
        mods: BTreeSet<String>,
    }

    impl<'a, 'b, 'ast> Visit<'ast> for Visitor<'a, 'b> {
        fn visit_item_mod(&mut self, item: &'ast ItemMod) {
            if item.attrs.is_empty() {
                if let Some((_, items)) = &item.content {
                    let used = uses_of_extern_crates(items, self.extern_crates);
                    self.used.extend(used);
                } else {
                    self.mods.insert(item.ident.to_string());
                }
                visit::visit_item_mod(self, item);
            }
        }

        fn visit_item_use(&mut self, item: &'ast ItemUse) {
            let used = use_of_extern_crate(item, self.extern_crates);
            self.used.extend(used);
        }
    }

    let root_path = match src.path() {
        None => return Ok(hashset!()),
        Some(path) => path.to_owned(),
    };
    let (mut mods, mut used) = (btreeset!(vec![]), hashset!());

    while !mods.is_empty() {
        let mut next_mods = btreeset!();
        for mods in mods {
            let path = {
                let mut path = root_path.clone();
                let mut mods = mods.iter().peekable();
                if mods.peek().is_some() {
                    path.pop();
                }
                let mut another_path = None;
                while let Some(m) = mods.next() {
                    if mods.peek().is_some() {
                        path.push(m);
                    } else {
                        another_path = Some(path.join(m).join("mod.rs"));
                        path.push(Path::new(m).with_extension("rs"));
                    }
                }
                if path.exists() {
                    path
                } else if let Some(another_path) = another_path {
                    if another_path.exists() {
                        another_path
                    } else {
                        return Err(failure::err_msg(format!(
                            "No such file: {:?}",
                            btreeset!(path, another_path),
                        )));
                    }
                } else {
                    return Err(failure::err_msg(format!("No such file: {:?}", path)));
                }
            };
            let file = crate::fs::read_src(&path)?;
            let mut visitor = Visitor {
                extern_crates,
                used: uses_of_extern_crates(&file.items, extern_crates),
                mods: btreeset!(),
            };
            visitor.visit_file(&file);
            for m in visitor.mods {
                let mut mods = mods.clone();
                mods.push(m);
                next_mods.insert(mods);
            }
            used.extend(visitor.used);
        }
        mods = next_mods;
    }
    Ok(used)
}

fn use_of_extern_crate<'a>(item: &ItemUse, extern_crates: &HashSet<&'a str>) -> Option<&'a str> {
    if !item.attrs.is_empty() {
        return None;
    }
    let top = match &item.tree {
        UseTree::Path(path) => &path.ident,
        UseTree::Name(name) => &name.ident,
        UseTree::Rename(rename) => &rename.ident,
        _ => return None,
    };
    extern_crates.get(&*top.to_string()).cloned()
}

fn uses_of_extern_crates<'a>(items: &[Item], extern_crates: &HashSet<&'a str>) -> HashSet<&'a str> {
    items
        .iter()
        .flat_map(|item| match item {
            Item::Use(item) => Some(item),
            _ => None,
        })
        .flat_map(|item| use_of_extern_crate(item, extern_crates))
        .collect()
}

#[cfg(test)]
mod tests {
    use cargo::core::manifest::TargetSourcePath;
    use failure::Fallible;
    use maplit::hashset;
    use once_cell::sync::Lazy;

    use std::collections::HashSet;

    #[test]
    fn test_find_uses_lossy_2018() -> Fallible<()> {
        static PATH: Lazy<TargetSourcePath> = Lazy::new(|| TargetSourcePath::Path(file!().into()));
        static EXTERN_CRATES: Lazy<HashSet<&str>> =
            Lazy::new(|| hashset!("cargo", "failure", "maplit", "once_cell", "syn"));
        static EXPECTED: Lazy<HashSet<&str>> =
            Lazy::new(|| hashset!("cargo", "failure", "maplit", "syn"));

        let used = super::find_uses_lossy_2018(&PATH, &EXTERN_CRATES)?;
        assert_eq!(used, *EXPECTED);
        Ok(())
    }
}
