use std::sync::Arc;

use guppy::graph::PackageGraph;
use toml_edit::Document;

use crate::{
    dependency::DependencyIndex,
    group::GroupIndex,
    ruleset::{AdjacencyMap, Ruleset},
};

pub struct EngineState {
    pub manifest: CargoManifest,
    pub graph: PackageGraph,
    pub ruleset: Ruleset,
    pub dependency_index: Arc<DependencyIndex>,
    pub group_index: Arc<GroupIndex>,
    pub adj_map: AdjacencyMap,
}

pub struct CargoManifest {
    pub filepath: String,
    pub document: Document<String>,
}
