//! The worked examples (File › Examples), built in code so they always match the engine.

use crate::doc::{Doc, Item, Op};
use crate::model::{BendKind, Comp, Element, Model, Nozzle, Pipe, RigidKind, Support, SupKind, K0};
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
        Example { id: "pump", name: "Pump discharge with an expansion loop", what: "6\" hot water from a pump nozzle to a header, an expansion loop, rests and a guide; nozzle loads checked." },
        Example { id: "compressor", name: "Compressor suction to a header", what: "8\" suction line from a header down to a compressor nozzle, a spring hanger, nozzle loads in the nozzle's axes." },
        Example { id: "steam", name: "Steam line on a pipe rack", what: "8\" insulated steam at 250 °C along a rack: rests, guides and a line stop, then down to a turbine." },
        Example { id: "tank", name: "Tank outlet with settlement", what: "10\" tank outlet with a valve; the tank settles 12 mm (case 2) and the nozzle grows when hot." },
        Example { id: "lbend", name: "A simple L-bend", what: "6\" carbon steel at 200 °C between two anchors, with a long-radius bend. Check it by hand." },
        Example { id: "zbend", name: "A Z-bend", what: "6\" carbon steel at 250 °C: two bends between two anchors, the classic hand-check case." },
    ]
}

pub fn get(id: &str) -> Option<Model> {
    match id {
        "blank" => Some(Model::new()),
        "pump" => Some(pump()),
        "compressor" => Some(compressor()),
        "steam" => Some(steam()),
        "tank" => Some(tank()),
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
    // Rests are one-way, as new rests are in the window.
    let one_way = kind == SupKind::Rest;
    d.apply(Op::Add { item: Item::Support(Support { node, kind, tag: tag.into(), one_way, ..Default::default() }), after: None }).expect("support");
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

fn bends(d: &mut Doc, at: &[u32]) {
    for n in at {
        bend(d, *n);
    }
}
fn pipe(d: &mut Doc, nps: &str, sched: &str, ins: f64, fluid: f64) {
    let mut p = Pipe::std(d.m.pipes[0].id, nps, sched);
    p.ins_t = ins;
    p.ins_rho = if ins > 0.0 { 150.0 } else { 0.0 };
    p.clad_t = if ins > 0.0 { 0.0007 } else { 0.0 };
    p.clad_rho = if ins > 0.0 { 2700.0 } else { 0.0 };
    p.fluid_rho = fluid;
    p.corrosion = 0.0015;
    d.apply(Op::Set { item: Item::Pipe(p) }).expect("pipe");
}
fn rigid(d: &mut Doc, to: u32, kind: RigidKind, mass: f64) {
    let mut e = d.m.elements.iter().find(|e| e.to == to).expect("rigid element").clone();
    e.comp = Comp::Rigid { rigid: kind, mass };
    d.apply(Op::Set { item: Item::Element(e) }).expect("rigid");
}
fn nozzle(d: &mut Doc, node: u32, name: &str, axis: [f64; 3], allow: [f64; 4]) {
    d.apply(Op::Add { item: Item::Nozzle(Nozzle { node, name: name.into(), axis, allow, ..Default::default() }), after: None }).expect("nozzle");
}
/// The machine a nozzle belongs to, checked against the published criteria.
fn machine(d: &mut Doc, name: &str, kind: crate::equip::EquipKind, factor: f64, shaft: [f64; 3]) {
    let nozzles: Vec<u32> = d.m.nozzles.iter().map(|z| z.id).collect();
    let e = crate::equip::Equipment { name: name.into(), kind, nozzles: nozzles.clone(), reference: nozzles[0], shaft, factor, ..Default::default() };
    d.apply(Op::SetEquipment { list: vec![e] }).expect("equipment");
}
fn displace(d: &mut Doc, node: u32, case: usize, v: [f64; 6]) {
    let mut s = d.m.supports.iter().find(|s| s.node == node).expect("support").clone();
    s.disp[case] = v;
    d.apply(Op::Set { item: Item::Support(s) }).expect("displacement");
}

/// Example 1, pump discharge: up from the pump nozzle, a check valve and a gate valve, east along a sleeper
/// with an expansion loop to the header anchor.
fn pump() -> Model {
    let mut d = table("Pump discharge with an expansion loop", &[
        (10, 20, 0.0, 0.6, 0.0), (20, 30, 0.0, 0.5, 0.0), (30, 40, 0.0, 0.6, 0.0), (40, 50, 0.0, 1.0, 0.0),
        (50, 60, 5.0, 0.0, 0.0), (60, 70, 5.0, 0.0, 0.0), (70, 80, 0.0, 0.0, -5.0), (80, 85, 2.0, 0.0, 0.0), (85, 90, 2.0, 0.0, 0.0),
        (90, 100, 0.0, 0.0, 5.0), (100, 110, 5.0, 0.0, 0.0), (110, 120, 5.0, 0.0, 0.0),
    ]);
    pipe(&mut d, "6", "40", 0.0, 900.0);
    rigid(&mut d, 30, RigidKind::Valve, 120.0);
    bends(&mut d, &[50, 70, 80, 90, 100]);
    conditions(&mut d, "Hot water 150 °C", [150.0, 21.0, 21.0], [20.0, 0.0, 0.0], 30.0);
    support(&mut d, 10, SupKind::Anchor, "P-101");
    displace(&mut d, 10, 0, [0.0, 0.0012, 0.0, 0.0, 0.0, 0.0]);
    nozzle(&mut d, 10, "P-101 discharge", [0.0, 1.0, 0.0], [12000.0, 9000.0, 6000.0, 8000.0]);
    support(&mut d, 60, SupKind::Rest, "S-60");
    support(&mut d, 110, SupKind::Rest, "S-110");
    support(&mut d, 85, SupKind::Rest, "S-85");
    support(&mut d, 110, SupKind::Guide, "G-110");
    support(&mut d, 120, SupKind::Anchor, "A-120 header");
    let mut st = d.m.settings.clone();
    st.hydro = true;
    d.apply(Op::SetSettings { s: st, units: "SI".into() }).expect("hydrotest");
    meta(&mut d, "Pump discharge with an expansion loop", "6\"-HW-101", "6\" Sch 40 carbon steel, hot water at 150 °C and 20 bar, from the pump discharge nozzle (which grows 1.2 mm upwards when hot) up through a gate valve, then east with an expansion loop to an anchor at the header. The nozzle loads are checked against allowables entered for the pump; the hydrotest case (30 bar, full of water) gives the support loads during the test.");
    d.m
}

/// Example 2, compressor suction: from an anchor at the header, across and down to the compressor's top
/// nozzle, with a spring hanger on the vertical run.
fn compressor() -> Model {
    let mut d = table("Compressor suction to a header", &[
        (10, 20, 0.0, 2.5, 0.0), (20, 30, 4.0, 0.0, 0.0), (30, 35, 0.0, 0.0, -3.0), (35, 40, 0.0, 0.0, -3.0), (40, 50, 0.0, 3.0, 0.0), (50, 60, 3.0, 0.0, 0.0),
    ]);
    pipe(&mut d, "8", "40", 0.0, 8.0);
    bends(&mut d, &[20, 30, 40, 50]);
    conditions(&mut d, "Suction gas 95 °C", [95.0, -10.0, 21.0], [8.0, 8.0, 0.0], 12.0);
    support(&mut d, 10, SupKind::Anchor, "K-201");
    displace(&mut d, 10, 0, [0.0, 0.0008, 0.0, 0.0, 0.0, 0.0]);
    nozzle(&mut d, 10, "K-201 suction", [0.0, 1.0, 0.0], [6000.0, 4500.0, 3000.0, 4000.0]);
    machine(&mut d, "K-201", crate::equip::EquipKind::Compressor, 1.85, [1.0, 0.0, 0.0]);
    support(&mut d, 30, SupKind::Spring { dir: [0.0, 1.0, 0.0], k: 175e3 }, "SH-30");
    support(&mut d, 35, SupKind::Rest, "S-35");
    support(&mut d, 60, SupKind::Anchor, "A-60 header");
    meta(&mut d, "Compressor suction to a header", "8\"-PG-201", "8\" Sch 40 carbon steel suction gas: case 1 runs at 95 °C, case 2 at −10 °C (a cold start), both at 8 bar. The compressor nozzle grows 0.8 mm up when hot. A spring hanger at node 30 and a rest at 35 carry the weight; the nozzle loads are reported in the nozzle's axes.");
    d.m
}

/// Example 3, steam on a pipe rack: east along the rack on rests with guides, held axially by a line stop
/// at 20, then down and north to a turbine inlet; the drop and the north leg take the growth.
fn steam() -> Model {
    let mut d = table("Steam line on a pipe rack", &[
        (10, 20, 6.0, 0.0, 0.0), (20, 30, 6.0, 0.0, 0.0), (30, 40, 6.0, 0.0, 0.0), (40, 50, 6.0, 0.0, 0.0),
        (50, 60, 0.0, -6.0, 0.0), (60, 65, 0.0, 0.0, -3.0), (65, 70, 0.0, 0.0, -3.0),
    ]);
    pipe(&mut d, "8", "40", 0.075, 12.0);
    bends(&mut d, &[50, 60]);
    conditions(&mut d, "Steam 250 °C", [250.0, 21.0, 21.0], [15.0, 0.0, 0.0], 23.0);
    for n in [10, 20, 30, 40] {
        support(&mut d, n, SupKind::Rest, &format!("S-{n}"));
    }
    support(&mut d, 10, SupKind::Guide, "G-10");
    support(&mut d, 30, SupKind::Guide, "G-30");
    // The guides have 3 mm of clearance on each side, as rack guides usually do, and the rack's
    // rests and guides have steel-on-steel friction (μ = 0.3).
    for g in d.m.supports.iter().filter(|s| s.kind == SupKind::Guide).cloned().collect::<Vec<_>>() {
        d.apply(Op::Set { item: Item::Support(Support { gap: 0.003, mu: 0.3, ..g }) }).expect("guide gap");
    }
    for g in d.m.supports.iter().filter(|s| s.kind == SupKind::Rest).cloned().collect::<Vec<_>>() {
        d.apply(Op::Set { item: Item::Support(Support { mu: 0.3, ..g }) }).expect("rest friction");
    }
    support(&mut d, 65, SupKind::Rest, "S-65");
    support(&mut d, 20, SupKind::Linestop, "LS-20");
    support(&mut d, 70, SupKind::Anchor, "T-301 inlet");
    displace(&mut d, 70, 0, [0.0, 0.002, 0.0, 0.0, 0.0, 0.0]);
    // Outdoors on the rack: a design wind of 0.8 kPa across it (from the south) and 0.15 g of static
    // seismic sideways, each checked as an occasional case.
    let mut st = d.m.settings.clone();
    st.wind_q = 800.0;
    st.wind_cd = 0.7;
    st.wind_dir = [0.0, 0.0, -1.0];
    st.seis = [0.15, 0.0, 0.15];
    d.apply(Op::SetSettings { s: st, units: "SI".into() }).expect("occasional loads");
    nozzle(&mut d, 70, "T-301 inlet", [0.0, 0.0, -1.0], [12000.0, 9000.0, 12000.0, 12000.0]);
    machine(&mut d, "T-301", crate::equip::EquipKind::Turbine, 1.0, [1.0, 0.0, 0.0]);
    meta(&mut d, "Steam line on a pipe rack", "8\"-ST-301", "8\" Sch 40 carbon steel, steam at 250 °C and 15 bar, 75 mm of insulation with aluminium cladding. 24 m along a pipe rack on one-way rests with friction (μ 0.3), with guides (3 mm gaps) at 10 and 30 and a line stop at 20 that holds the line along its axis, then 6 m down and 6 m north (on a rest at 65) to a turbine inlet that grows 2 mm up when hot. The turbine's nozzle fails the turbine criteria (Results › Equipment): the line is too stiff near it, so try a loop or a spring near the turbine. Wind (0.8 kPa from the south) and 0.15 g of static seismic are checked as occasional cases.");
    d.m
}

/// Example 4, tank outlet: from the tank's nozzle, a valve, then to a pump suction header; the tank settles.
fn tank() -> Model {
    let mut d = table("Tank outlet with settlement", &[
        (10, 20, 1.0, 0.0, 0.0), (20, 30, 0.7, 0.0, 0.0), (30, 40, 3.0, 0.0, 0.0), (40, 50, 0.0, 0.0, -3.0), (50, 60, 0.0, 0.0, -3.0), (60, 70, 6.0, 0.0, 0.0),
    ]);
    pipe(&mut d, "10", "STD", 0.0, 870.0);
    rigid(&mut d, 30, RigidKind::Valve, 260.0);
    bends(&mut d, &[40, 60]);
    conditions(&mut d, "Crude 65 °C", [65.0, 21.0, 21.0], [3.0, 0.0, 0.0], 6.0);
    support(&mut d, 10, SupKind::Anchor, "TK-401");
    // Case 1: the shell nozzle moves out 1 mm and up 0.5 mm when warm. Case 2: the tank settles 12 mm.
    displace(&mut d, 10, 0, [-0.001, 0.0005, 0.0, 0.0, 0.0, 0.0]);
    displace(&mut d, 10, 1, [0.0, -0.012, 0.0, 0.0, 0.0, 0.0]);
    nozzle(&mut d, 10, "TK-401 outlet", [1.0, 0.0, 0.0], [20000.0, 15000.0, 20000.0, 25000.0]);
    // A spring near the tank so the settlement does not hang the line off the nozzle.
    support(&mut d, 40, SupKind::Hanger { k: 0.0, cold: 0.0 }, "SH-40");
    support(&mut d, 50, SupKind::Rest, "S-50");
    support(&mut d, 70, SupKind::Anchor, "A-70");
    meta(&mut d, "Tank outlet with settlement", "10\"-CR-401", "10\" Std carbon steel crude oil at 65 °C from a tank shell nozzle, through a gate valve, round two bends to an anchor. Case 2 is the tank settling 12 mm: a displacement range the piping must take without overloading the nozzle, which is why the first support is a spring hanger, sized by Analyze from the spring table.");
    d.m
}

/// A large model for speed checks (not in the File page's list): a rack line of `loops` bays, each
/// 36 m straight in 6 m elements on one-way rests (friction 0.3) with a guide every other rest,
/// then an expansion loop 4 m up, 3 m across and 4 m down with long-radius bends; a line stop in
/// each bay; anchors at both ends; 8" Sch 40 steam at 250 °C. About 10 elements per bay.
pub fn big(loops: usize) -> Model {
    let mut m = Model::new();
    m.name = format!("Speed check: {loops} bays");
    m.elements.clear();
    m.nodes.clear();
    m.supports.clear();
    m.pipes[0] = crate::model::Pipe { id: m.pipes[0].id, ..crate::model::Pipe::std(m.pipes[0].id, "8", "40") };
    m.conds[0].t[0] = K0 + 250.0;
    m.conds[0].p[0] = 15e5;
    let (pipe, mat, cond) = (m.pipes[0].id, m.materials[0].id, m.conds[0].id);
    let mut next = m.next.max(m.max_id() + 1);
    let mut id = || { next += 1; next - 1 };
    let mut node = 10u32;
    let add = |m: &mut Model, d: [f64; 3], comp: Comp, node: &mut u32| {
        let e = Element { id: 0, from: *node, to: *node + 10, d, pipe, material: mat, cond, comp, ..Default::default() };
        m.elements.push(e);
        *node += 10;
    };
    let mut sups: Vec<(u32, SupKind, f64, bool)> = vec![(10, SupKind::Anchor, 0.0, false)];
    for _ in 0..loops {
        for k in 0..6 {
            let at = node;
            sups.push((at, SupKind::Rest, 0.3, true));
            if k % 2 == 1 {
                sups.push((at, SupKind::Guide, 0.0, false));
            }
            if k == 3 {
                sups.push((at, SupKind::Linestop, 0.0, false));
            }
            let last = k == 5;
            add(&mut m, [6.0, 0.0, 0.0], if last { Comp::Bend { bend: BendKind::Long, r: 0.0 } } else { Comp::Pipe }, &mut node);
        }
        add(&mut m, [0.0, 4.0, 0.0], Comp::Bend { bend: BendKind::Long, r: 0.0 }, &mut node);
        add(&mut m, [3.0, 0.0, 0.0], Comp::Bend { bend: BendKind::Long, r: 0.0 }, &mut node);
        add(&mut m, [0.0, -4.0, 0.0], Comp::Bend { bend: BendKind::Long, r: 0.0 }, &mut node);
    }
    add(&mut m, [6.0, 0.0, 0.0], Comp::Pipe, &mut node);
    sups.push((node, SupKind::Anchor, 0.0, false));
    for e in &mut m.elements {
        e.id = id();
    }
    for (n, kind, mu, one_way) in sups {
        m.supports.push(Support { id: id(), node: n, kind, mu, one_way, ..Default::default() });
    }
    m.nodes = vec![crate::model::Node { num: 10, x: 0.0, y: 0.0, z: 0.0 }];
    m.next = next;
    crate::cases::rebuild(&mut m);
    m.fix_ids();
    m.layout().expect("big model layout");
    m.check().expect("big model");
    m
}
