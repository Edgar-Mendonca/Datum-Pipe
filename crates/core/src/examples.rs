//! The worked examples (File › Examples), built in code so they always match the engine.

use crate::doc::{Doc, Item, Op};
use crate::model::{BendKind, Comp, Element, Model, Support, SupKind, K0};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Example {
    pub id: &'static str,
    pub name: &'static str,
    pub what: &'static str,
}

pub fn list() -> Vec<Example> {
    vec![
        Example { id: "blank", name: "New piping model", what: "An empty model: type the first element in the table." },
        Example { id: "lbend", name: "A simple L-bend", what: "6\" carbon steel at 200 °C between two anchors, with a long-radius bend. Check it by hand." },
        Example { id: "zbend", name: "A Z-bend", what: "6\" carbon steel at 250 °C: two bends between two anchors, the classic hand-check case." },
    ]
}

pub fn get(id: &str) -> Option<Model> {
    match id {
        "blank" => Some(Model::new()),
        "lbend" => Some(lbend()),
        "zbend" => Some(zbend()),
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

fn bend(d: &mut Doc, at: u32) {
    let mut e = d.m.elements.iter().find(|e| e.to == at).expect("bend element").clone();
    e.comp = Comp::Bend { bend: BendKind::Long, r: 0.0 };
    d.apply(Op::Set { item: Item::Element(e) }).expect("bend");
}
pub(crate) fn support(d: &mut Doc, node: u32, kind: SupKind, tag: &str) {
    d.apply(Op::Add { item: Item::Support(Support { node, kind, tag: tag.into(), ..Default::default() }), after: None }).expect("support");
}
fn conditions(d: &mut Doc, name: &str, t: [f64; 3], p: [f64; 3], hydro: f64) {
    let mut c = d.m.conds[0].clone();
    c.name = name.into();
    c.t = t.map(|x| K0 + x);
    c.p = p.map(|x| x * 1e5);
    c.hydro = hydro * 1e5;
    d.apply(Op::Set { item: Item::Cond(c) }).expect("conditions");
}
fn meta(d: &mut Doc, name: &str, line: &str, notes: &str) {
    let mut pr = d.m.project.clone();
    pr.project = "Datum Pipe worked examples".into();
    pr.line = line.into();
    pr.calc = format!("EX-{line}");
    pr.rev = "A".into();
    d.apply(Op::SetMeta { name: name.into(), notes: notes.into(), project: pr }).expect("meta");
}

fn lbend() -> Model {
    let mut d = table("A simple L-bend", &[(10, 20, 6.0, 0.0, 0.0), (20, 30, 0.0, 0.0, -4.0)]);
    bend(&mut d, 20);
    conditions(&mut d, "Hot gas 200 °C", [200.0, 21.0, 21.0], [10.0, 0.0, 0.0], 15.0);
    support(&mut d, 10, SupKind::Anchor, "A-10");
    support(&mut d, 30, SupKind::Anchor, "A-30");
    meta(&mut d, "A simple L-bend", "L-BEND", "6\" Sch 40 carbon steel, 6 m east then 4 m north with a long-radius bend, anchored at both ends; 200 °C and 10 bar. The thermal end forces can be checked by hand with the guided-cantilever or elastic centre method.");
    d.m
}

fn zbend() -> Model {
    let mut d = table("A Z-bend", &[(10, 20, 4.0, 0.0, 0.0), (20, 30, 0.0, 0.0, -3.0), (30, 40, 4.0, 0.0, 0.0)]);
    bend(&mut d, 20);
    bend(&mut d, 30);
    conditions(&mut d, "Hot oil 250 °C", [250.0, 21.0, 21.0], [12.0, 0.0, 0.0], 18.0);
    support(&mut d, 10, SupKind::Anchor, "A-10");
    support(&mut d, 40, SupKind::Anchor, "A-40");
    meta(&mut d, "A Z-bend", "Z-BEND", "6\" Sch 40 carbon steel: 4 m east, 3 m north, 4 m east, with two long-radius bends, anchored at both ends; 250 °C and 12 bar.");
    d.m
}
