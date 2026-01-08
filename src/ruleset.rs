use std::collections::HashMap;

use guppy::{PackageId, Version};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Label {
    pub members: Vec<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: RuleType,
    pub targets: Vec<String>,
    pub fix_hint: String,
}

#[derive(Debug, Deserialize)]
pub struct Ruleset {
    pub labels: HashMap<String, Label>,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    ExactlyOneOf,
    NoneOrOneOf,
    AtleastOneOf,
}

#[derive(Debug)]
pub struct DependencyIndex {
    pub deps: HashMap<String, DependencyProp>,
}

impl DependencyIndex {
    pub fn new() -> Self {
        Self {
            deps: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct DependencyProp {
    pub ids: Vec<PackageId>,
    pub versions: Vec<Version>,
    pub groups: Vec<String>,
}

impl DependencyProp {
    pub fn new<T: Into<String>>(group: T) -> Self {
        Self {
            ids: vec![],
            versions: vec![],
            groups: vec![group.into()],
        }
    }

    pub fn insert_group(&mut self, group: impl Into<String>) {
        self.groups.push(group.into());
    }
}
