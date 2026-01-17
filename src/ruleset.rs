use std::collections::{HashMap, HashSet};

use guppy::{PackageId, Version, graph::PackageMetadata};
use serde::Deserialize;

pub type GroupIndex<'g> = HashMap<String, Vec<Dependency<'g>>>;
pub type DependencyIndex<'g> = HashMap<String, DependencyProp<'g>>;
pub type AdjacencyMap = HashMap<Package, HashSet<Package>>;
type RuleId = String;

#[derive(Debug, Deserialize)]
pub struct Group {
    pub members: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub id: RuleId,
    pub name: String,
    #[serde(rename = "type")]
    pub _type: RuleType,
    pub targets: Vec<String>,
    pub severity: RuleSeverity,
    pub reason: String,
    pub scope: RuleScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Ruleset {
    pub groups: HashMap<String, Group>,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    OneOf,
    AtLeastOne,
    AtMostOne,
    Requires,
    Forbids,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleScope {
    All,
    Direct,
    Transitive,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleSeverity {
    Fatal,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct DependencyProp<'g> {
    pub packages: Vec<PackageMetadata<'g>>,
    pub groups: Vec<String>,
}

#[derive(Debug)]
pub struct Dependency<'g> {
    pub name: String,
    pub properties: DependencyProp<'g>,
}

impl<'g> Dependency<'g> {
    pub fn new(name: impl Into<String>, properties: DependencyProp<'g>) -> Self {
        Self {
            name: name.into(),
            properties,
        }
    }
}

impl DependencyProp<'_> {
    pub fn new<T: Into<String>>(group: T) -> Self {
        Self {
            packages: vec![],
            groups: vec![group.into()],
        }
    }

    pub fn insert_group(&mut self, group: impl Into<String>) {
        self.groups.push(group.into());
    }
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

#[derive(PartialEq)]
pub enum RuleState {
    NoConflict,
    Conflict,
    NoGroupFound,
}

impl RuleState {
    pub fn is_conflict(self) -> bool {
        self == Self::Conflict
    }
}
