use core::fmt;
use std::{collections::HashMap, error::Error, fmt::Display, fs::read_to_string, path::Path};

use clap::Parser;
use guppy::{
    MetadataCommand, PackageId,
    graph::{DependencyDirection, DotWrite, PackageDotVisitor, PackageLink, PackageMetadata},
};

mod cli;
mod error;
mod ruleset;

use cli::Cli;
use owo_colors::{OwoColorize, Style};
use ruleset::{Dependency, DependencyProp, Ruleset};
use termtree::Tree;
use toml_edit::{Document, Item, Table};

use crate::ruleset::{DependencyIndex, GroupIndex, RuleType};

struct PackageNameVisitor;

impl PackageDotVisitor for PackageNameVisitor {
    fn visit_package(&self, package: PackageMetadata<'_>, f: &mut DotWrite<'_, '_>) -> fmt::Result {
        // Print out the name of the package. Other metadata can also be printed out.
        //
        // If you need to look at data for other packages, store a reference to the PackageGraph in
        // the visitor.
        write!(f, "{} ({})", package.name(), package.version())
    }

    fn visit_link(&self, link: PackageLink<'_>, f: &mut DotWrite<'_, '_>) -> fmt::Result {
        if link.dev_only() {
            write!(f, "dev-only")
        } else {
            // Don't print out anything if this isn't a dev-only link.
            Ok(())
        }
    }
}

fn manifest_contains_dep(manifest: &Document<String>, name: &str) -> bool {
    if let Item::Table(workspace_table) = &manifest["workspace"]["dependencies"] {
        if let Item::Table(table) = &manifest["dependencies"] {
            return table.contains_key(name) || workspace_table.contains_key(name);
        }
        return workspace_table.contains_key(name);
    }

    false
}

fn reverse_bfs<'g>(manifest: &Document<String>, link: PackageLink<'g>, root: &PackageMetadata<'g>) {
    let from_name = link.from().name();
    let manifest_style = Style::new().red();
    let root_style = Style::new().magenta().bold();

    if manifest_contains_dep(manifest, link.from().name()) {
        if link.to().name() == root.name() {
            println!(
                "{} -> {}",
                from_name.style(manifest_style),
                link.to().name().style(root_style)
            );
        } else {
            println!(
                "{} -> {}",
                from_name.style(manifest_style),
                link.to().name()
            );
        }
    } else {
        if link.to().name() == root.name() {
            println!("{} -> {}", from_name, link.to().name().style(root_style));
        } else {
            println!("{} -> {}", from_name, link.to().name());
        }
        for child_link in link.from().reverse_direct_links() {
            reverse_bfs(manifest, child_link, root);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut cmd = MetadataCommand::new();
    if let Some(dir) = &cli.workspace {
        cmd.current_dir(dir);
    }
    let graph = cmd.build_graph()?;

    let root_dir = graph.workspace().root();

    let ruleset: Ruleset = {
        let ruleset_file = &cli
            .ruleset_file
            .unwrap_or(format!("{root_dir}/conflict.toml"));
        let ruleset_path = Path::new(ruleset_file);
        let input = read_to_string(ruleset_path)?;
        toml::from_str(&input)?
    };

    let cargo_manifest: Document<_> = {
        let filepath = format!("{root_dir}/Cargo.toml");
        let path = Path::new(&filepath);
        let input = read_to_string(path)?;
        input.parse()?
    };

    println!("{ruleset:?}");

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
            dep.packages.push(pkg);
        });
    }

    deps_index.retain(|_, v| !v.packages.is_empty());

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
            RuleType::OneOf => {
                let mut is_valid = false;
                for group in rule.targets.iter() {
                    if groups_index.contains_key(group) {
                        groups_index.get(group).unwrap().iter().for_each(|dep| {
                            for pkg in dep.properties.packages.iter() {
                                for link in pkg.reverse_direct_links() {
                                    reverse_bfs(&cargo_manifest, link, pkg);
                                }
                            }
                            //let ids = dep
                            //    .properties
                            //    .packages
                            //    .iter()
                            //    .map(|pkg| pkg.id())
                            //    .collect::<Vec<&PackageId>>();
                            //let pkgs = graph.query_reverse(ids).unwrap().resolve();
                            //for link in pkgs.links(DependencyDirection::Reverse) {
                            //    println!("{} -> {}", link.from().name(), link.to().name());
                            //}
                            println!("--------------");
                        });

                        if !is_valid {
                            is_valid = true;
                        } else {
                            is_valid = false;
                            break;
                        }
                    }
                }

                if !is_valid {
                    println!("{}", rule.reason);
                }
            }
            RuleType::AtLeastOne => todo!(),
            RuleType::AtMostOne => todo!(),
            RuleType::Requires => todo!(),
            RuleType::Forbids => todo!(),
        }
    }

    Ok(())
}
