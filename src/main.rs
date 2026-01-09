use std::{collections::HashMap, error::Error, fs::read_to_string, path::Path};

use guppy::{MetadataCommand, PackageId};

mod cli;
mod ruleset;

use ruleset::{Dependency, DependencyProp, Ruleset};

use crate::ruleset::{DependencyIndex, GroupIndex, RuleType};

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

    //println!("{:?}", graph.package_count());

    let mut deps_index = DependencyIndex::new();

    for (group_name, group) in ruleset.groups {
        group.members.into_iter().for_each(|pkg| {
            deps_index
                .entry(pkg)
                .and_modify(|dep| dep.insert_group(&group_name))
                .or_insert_with(|| DependencyProp::new(&group_name));
        });
    }

    for pkg in graph.packages() {
        deps_index.entry(pkg.name().to_string()).and_modify(|dep| {
            dep.ids.push(pkg.id().clone());
            dep.versions.push(pkg.version().clone());
        });
    }

    deps_index.retain(|_, v| !v.ids.is_empty());

    let mut groups_index = GroupIndex::new();

    for (name, props) in deps_index.iter() {
        props.groups.clone().into_iter().for_each(|group| {
            groups_index
                .entry(group)
                .and_modify(|grp| grp.push(Dependency::new(name.clone(), props.clone())))
                .or_insert_with(|| vec![Dependency::new(name.clone(), props.clone())]);
        });
    }

    //println!("{groups_index:?}");

    // Parse the rules
    for rule in ruleset.rules {
        match rule._type {
            RuleType::ExactlyOneOf => {
                let mut is_valid = false;
                for group in rule.targets.iter() {
                    if groups_index.contains_key(group) {
                        if !is_valid {
                            is_valid = true;
                        } else {
                            is_valid = false;
                            break;
                        }
                    }
                }

                if !is_valid {
                    println!("{}", rule.fix_hint);
                }
            }
            RuleType::NoneOrOneOf => todo!(),
            RuleType::AtleastOneOf => todo!(),
        }
    }

    Ok(())
}
