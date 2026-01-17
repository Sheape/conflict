use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    path::Path,
};

use clap::Parser;
use guppy::{
    MetadataCommand, PackageId,
    graph::{PackageLink, PackageMetadata},
};

mod cli;
mod error;
mod ruleset;

use cli::Cli;
use miette::{NamedSource, SourceOffset, SourceSpan};
use owo_colors::{OwoColorize, Style};
use ruleset::{Dependency, DependencyProp, Package, Ruleset};
use termtree::Tree;
use toml_edit::{Document, Item};

use crate::{
    error::{Error, SslProviderConflict},
    ruleset::{AdjacencyMap, DependencyIndex, GroupIndex, RuleState, RuleType},
};

fn manifest_contains_dep(manifest: &Document<String>, name: &str) -> Option<(usize, usize)> {
    if let Item::Table(table) = &manifest["dependencies"] {
        if let Item::Table(workspace_table) = &manifest["workspace"]["dependencies"] {
            let key_opt = match (table.key(name), workspace_table.key(name)) {
                (None, None) => None,
                (None, Some(key)) => Some(key),
                (Some(key), None) => Some(key),
                (Some(_), Some(key)) => Some(key),
            };

            return key_opt.map(|key| {
                let span = key.span().unwrap();
                (span.start, span.end - span.start)
            });
        }
        return table.key(name).map(|key| {
            let span = key.span().unwrap();
            (span.start, span.end - span.start)
        });
    }

    None
}

fn reverse_bfs<'g>(
    manifest: &Document<String>,
    link: PackageLink<'g>,
    adj_map: &'g mut AdjacencyMap,
) {
    let from_name = link.from().name();
    let manifest_style = Style::new().red();
    let root_style = Style::new().magenta().bold();
    adj_map
        .entry(Package::new(
            link.to().id().clone(),
            manifest_contains_dep(manifest, link.to().name()),
        ))
        .and_modify(|parents| {
            parents.insert(Package::new(
                link.from().id().clone(),
                manifest_contains_dep(manifest, link.from().name()),
            ));
        })
        .or_insert(HashSet::from([Package::new(
            link.from().id().clone(),
            manifest_contains_dep(manifest, link.from().name()),
        )]));

    if manifest_contains_dep(manifest, link.from().name()).is_none() {
        for child_link in link.from().reverse_direct_links() {
            reverse_bfs(manifest, child_link, adj_map);
        }
    }
}

fn dfs<'g>(
    current: Package,
    adj_map: &'g AdjacencyMap,
    visited: &'g mut HashSet<Package>,
    path: &'g mut Vec<Package>,
    results: &'g mut Vec<Vec<Package>>,
) {
    if visited.contains(&current) {
        return;
    }

    visited.insert(current.clone());
    path.push(current.clone());

    if current.span_in_manifest.is_some() {
        results.push(path.clone());
    } else if let Some(parents) = adj_map.get(&current) {
        for parent in parents {
            dfs(parent.clone(), adj_map, visited, path, results);
        }
    }

    path.pop();
    visited.remove(&current);
}

struct CargoManifest {
    filepath: String,
    document: Document<String>,
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    let mut cmd = MetadataCommand::new();
    if let Some(dir) = &cli.workspace {
        cmd.current_dir(dir);
    }
    let graph = cmd.build_graph().unwrap();

    let root_dir = graph.workspace().root();

    let ruleset: Ruleset = {
        let ruleset_file = &cli
            .ruleset_file
            .unwrap_or(format!("{root_dir}/conflict.toml"));
        let ruleset_path = Path::new(ruleset_file);
        let input = read_to_string(ruleset_path).unwrap();
        toml::from_str(&input).unwrap()
    };

    let cargo_manifest: CargoManifest = {
        let filepath = format!("{root_dir}/Cargo.toml");
        let path = Path::new(&filepath);
        let input = read_to_string(path).unwrap();
        CargoManifest {
            filepath,
            document: input.parse().unwrap(),
        }
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
                let mut state = RuleState::NoGroupFound;
                let mut spans = HashMap::<String, SourceSpan>::new();
                for group in rule.targets.iter() {
                    if groups_index.contains_key(group) {
                        match state {
                            RuleState::NoConflict => state = RuleState::Conflict,
                            RuleState::Conflict => break,
                            RuleState::NoGroupFound => {
                                state = RuleState::NoConflict;
                            }
                        }

                        for dep in groups_index.get(group).unwrap().iter() {
                            let mut adj_map = AdjacencyMap::new();
                            for pkg in dep.properties.packages.iter() {
                                for link in pkg.reverse_direct_links() {
                                    reverse_bfs(&cargo_manifest.document, link, &mut adj_map);
                                }
                            }

                            let mut visited: HashSet<Package> = HashSet::new();
                            let mut path: Vec<Package> = vec![];
                            let mut results: Vec<Vec<Package>> = vec![];

                            let target = adj_map
                                .keys()
                                .find(|pkg| {
                                    graph
                                        .metadata(&pkg.id)
                                        .map(|meta| meta.name() == dep.name)
                                        .expect("Expected that the package exists.")
                                })
                                .unwrap();

                            dfs(
                                target.clone(),
                                &adj_map,
                                &mut visited,
                                &mut path,
                                &mut results,
                            );

                            let min_len = results
                                .iter()
                                .map(|path| path.len())
                                .min()
                                .expect("There must be expected min length of path");
                            let shortest_path = results
                                .iter()
                                .find(|path| path.iter().len() == min_len)
                                .unwrap();

                            let dep_in_manifest = shortest_path.last().unwrap();
                            spans.insert(
                                group.clone(),
                                dep_in_manifest
                                    .span_in_manifest
                                    .expect("There should be span here")
                                    .into(),
                            );

                            let tree = shortest_path
                                .iter()
                                .map(|pkg| {
                                    let meta = graph.metadata(&pkg.id).unwrap();
                                    let display_str = if pkg == target {
                                        format!(
                                            "{} (v{})",
                                            meta.name().magenta().bold(),
                                            meta.version()
                                        )
                                    } else {
                                        format!("{} (v{})", meta.name(), meta.version())
                                    };
                                    Tree::new(display_str)
                                })
                                .reduce(|prev, final_tree| final_tree.with_leaves([prev]))
                                .expect("there should be a final tree");
                            println!("{tree}");
                        }
                    }
                }

                if state.is_conflict() {
                    println!("{}", rule.reason);
                    Err(Error::SSLConflict(SslProviderConflict {
                        manifest: NamedSource::new(
                            cargo_manifest.filepath.clone(),
                            cargo_manifest.document.raw().to_string(),
                        ),
                        openssl_span: *spans.get("openssl").unwrap(),
                        boringssl_span: *spans.get("boring").unwrap(),
                    }))?;
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
