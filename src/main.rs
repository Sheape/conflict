use std::{collections::HashMap, error::Error, fs::read_to_string, path::Path};

use guppy::{MetadataCommand, PackageId};

mod cli;
mod ruleset;

use ruleset::{DependencyProp, Ruleset};

use crate::ruleset::DependencyIndex;

const RULESET_FILE: &str = "./conflict.toml";
const CARGO_MANIFEST_PATH: &str = "/home/sheape/code/surfql/Cargo.toml";

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(RULESET_FILE);
    let input = read_to_string(input_path)?;
    let ruleset: Ruleset = toml::from_str(&input)?;
    println!("{ruleset:?}");

    let mut cmd = MetadataCommand::new();
    cmd.manifest_path(CARGO_MANIFEST_PATH);
    let graph = cmd.build_graph()?;

    println!("{:?}", graph.package_count());

    let mut deps_index = DependencyIndex::new();

    for (group_name, label) in ruleset.labels {
        label.members.into_iter().for_each(|pkg| {
            deps_index
                .deps
                .entry(pkg)
                .and_modify(|dep| dep.insert_group(&group_name))
                .or_insert_with(|| DependencyProp::new(&group_name));
        });
    }

    for pkg in graph.packages() {
        deps_index
            .deps
            .entry(pkg.name().to_string())
            .and_modify(|dep| {
                dep.ids.push(pkg.id().clone());
                dep.versions.push(pkg.version().clone());
            });
    }

    deps_index.deps.retain(|_, v| !v.ids.is_empty());

    println!("{deps_index:?}");

    Ok(())
}
