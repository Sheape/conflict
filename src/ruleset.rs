use std::collections::{HashMap, HashSet};

use guppy::{PackageId, Version, graph::PackageMetadata};
use serde::Deserialize;

use crate::dependency::Package;
use crate::group::Group;

pub type AdjacencyMap = HashMap<Package, HashSet<Package>>;
type RuleId = String;

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
