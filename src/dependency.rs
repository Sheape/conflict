use std::collections::HashMap;

use dashmap::DashMap;
use guppy::PackageId;
use rayon::prelude::*;

use crate::{engine::EngineState, error::Result};

pub type DependencyIndex = DashMap<String, DependencyProperty>;

#[derive(Debug, Clone)]
pub struct DependencyProperty {
    pub package_ids: Vec<PackageId>,
    pub groups: Vec<String>,
}

#[derive(Debug)]
pub struct Dependency {
    pub name: String,
    pub properties: DependencyProperty,
}

impl Dependency {
    pub fn new(name: impl Into<String>, properties: DependencyProperty) -> Self {
        Self {
            name: name.into(),
            properties,
        }
    }
}

impl DependencyProperty {
    pub fn new<T: Into<String>>(group: T) -> Self {
        Self {
            package_ids: vec![],
            groups: vec![group.into()],
        }
    }

    pub fn insert_group(&mut self, group: impl Into<String>) {
        self.groups.push(group.into());
    }
}

// Converts the groups in the ruleset to a proper dependency index where each dependency (by name)
// is mapped to its versions (via `PackageId`) and which groups it belongs to.
pub fn eval_ruleset(engine_state: &EngineState) -> Result<DependencyIndex> {
    let mut deps_index = DependencyIndex::new();

    engine_state
        .ruleset
        .groups
        .par_iter()
        .for_each(|(group_name, group)| {
            group.members.iter().for_each(|pkg| {
                deps_index
                    .entry(pkg.clone())
                    .and_modify(|dep| dep.insert_group(group_name))
                    .or_insert_with(|| DependencyProperty::new(group_name));
            });
        });

    for pkg in engine_state.graph.packages() {
        deps_index.entry(pkg.name().to_string()).and_modify(|dep| {
            dep.package_ids.push(pkg.id().clone());
        });
    }

    deps_index.retain(|_, v| !v.package_ids.is_empty());

    Ok(deps_index)
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Package {
    pub id: PackageId,
    pub span_in_manifest: Option<(usize, usize)>,
}

impl Package {
    pub fn new(id: PackageId, span_in_manifest: Option<(usize, usize)>) -> Self {
        Self {
            id,
            span_in_manifest,
        }
    }
}
