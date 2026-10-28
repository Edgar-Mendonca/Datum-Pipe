//! The worked examples (File › Examples), built in code so they always match the engine.

use crate::doc::{Doc, Item, Op};
use crate::model::{Element, Model};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Example {
    pub id: &'static str,
    pub name: &'static str,
    pub what: &'static str,
}

pub fn list() -> Vec<Example> {
    vec![Example { id: "blank", name: "New piping model", what: "An empty model: type the first element in the table." }]
}

pub fn get(id: &str) -> Option<Model> {
    match id {
        "blank" => Some(Model::new()),
        _ => None,
    }
}

/// Builds a model from rows of (from, to, dx, dy, dz) in metres, all with the first pipe,
/// material and line conditions.
pub fn table(name: &str, rows: &[(u32, u32, f64, f64, f64)]) -> Doc {
    let mut m = Model::new();
    m.name = name.into();
    let (p, mat, c) = (m.pipes[0].id, m.materials[0].id, m.conds[0].id);
    let mut d = Doc::new(m);
    for &(from, to, dx, dy, dz) in rows {
        d.apply(Op::Add { item: Item::Element(Element { from, to, d: [dx, dy, dz], pipe: p, material: mat, cond: c, ..Default::default() }), after: None }).expect("example row");
    }
    d
}
