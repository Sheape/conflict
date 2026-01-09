use std::collections::HashMap;

use guppy::{PackageId, Version};
use serde::Deserialize;

pub type GroupIndex = HashMap<String, Vec<Dependency>>;
pub type DependencyIndex = HashMap<String, DependencyProp>;

#[derive(Debug, Deserialize)]
pub struct Group {
    pub members: Vec<String>,
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
    pub groups: HashMap<String, Group>,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    ExactlyOneOf,
    NoneOrOneOf,
    AtleastOneOf,
}

#[derive(Debug, Clone)]
pub struct DependencyProp {
    pub ids: Vec<PackageId>,
    pub versions: Vec<Version>,
    pub groups: Vec<String>,
}

#[derive(Debug)]
pub struct Dependency {
    pub name: String,
    pub properties: DependencyProp,
}

impl Dependency {
    pub fn new(name: impl Into<String>, properties: DependencyProp) -> Self {
        Self {
            name: name.into(),
            properties,
        }
    }
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
