use dashmap::{DashMap, mapref::multiple::RefMulti};
use rayon::prelude::*;
use serde::Deserialize;

use crate::{dependency::Dependency, engine::EngineState, error::Result};

pub type GroupIndex = DashMap<String, Vec<Dependency>>;

#[derive(Debug, Deserialize)]
pub struct Group {
    pub members: Vec<String>,
}

pub fn eval_dependencies(engine_state: &EngineState) -> Result<GroupIndex> {
    let mut groups_index = GroupIndex::new();

    engine_state
        .dependency_index
        .par_iter()
        .for_each(|ref_multi| {
            let (name, props) = ref_multi.pair();
            props.groups.clone().into_iter().for_each(|group| {
                groups_index
                    .entry(group)
                    .and_modify(|grp| grp.push(Dependency::new(name.clone(), props.clone())))
                    .or_insert_with(|| vec![Dependency::new(name.clone(), props.clone())]);
            });
        });

    Ok(groups_index)
}
