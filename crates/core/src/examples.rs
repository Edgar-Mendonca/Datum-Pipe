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
        Example { id: "relief", name: "Safety valve vent stack", what: "4\" steam safety valve on a header, discharging up a vent stack to the open air: the valve's thrust as an occasional load." },
        Example { id: "bellows", name: "Expansion joint between anchors", what: "8\" steam in a straight run with an untied bellows: guides each side, and main anchors that take the pressure thrust." },
        Example { id: "separator", name: "Vertical separator V-101", what: "A pressure vessel: 1.8 m two-phase separator with 2:1 heads at 16 bar and 200 °C; shell and head thickness, MAWP, test pressure and weights." },
        Example { id: "drum", name: "Horizontal drum D-201 on saddles", what: "A pressure vessel: 2.4 m horizontal drum on two 120° saddles; shell and heads, nozzles, and the saddle stresses by Zick's method." },
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
        "relief" => Some(relief()),
        "bellows" => Some(bellows()),
        "separator" => Some(separator()),
        "drum" => Some(drum()),
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

fn relief() -> Model {
    // The valve on the header nozzle (anchored), a short riser, 1.5 m east to a rest under the bend,
    // then 3 m up to the open end, guided halfway.
    let mut d = table("Safety valve vent stack", &[(10, 20, 0.0, 0.6, 0.0), (20, 25, 0.0, 0.3, 0.0), (25, 30, 1.5, 0.0, 0.0), (30, 35, 0.0, 1.5, 0.0), (35, 40, 0.0, 1.5, 0.0)]);
    pipe(&mut d, "4", "40", 0.0, 0.0);
    rigid(&mut d, 20, RigidKind::Valve, 120.0);
    bends(&mut d, &[25, 30]);
    conditions(&mut d, "Steam vent 200 °C", [200.0, 21.0, 21.0], [0.0, 0.0, 0.0], 0.0);
    support(&mut d, 10, SupKind::Anchor, "PSV-1 inlet");
    support(&mut d, 30, SupKind::Rest, "S-30");
    support(&mut d, 35, SupKind::Guide, "G-35");
    // Example values: 3 kg/s of steam (k 1.3, 18.015 kg/kmol) relieving at 200 °C, out of the
    // stack's open end; the bore of the 4" pipe is the exit; a dynamic load factor of 2.
    let rv = crate::relief::Relief { name: "PSV-1".into(), node: 40, flow: 3.0, k: 1.3, temp: K0 + 200.0, mw: 18.015, ..Default::default() };
    d.apply(Op::SetReliefs { list: vec![rv] }).expect("relief valve");
    meta(&mut d, "Safety valve vent stack", "4\"-SV-01", "A safety valve on a steam header, its 4\" Sch 40 outlet running east and then up a 3 m vent stack to the open air. When the valve lifts, the steam leaving the stack pushes it back down: 3 kg/s at 200 °C (example values) with a dynamic load factor of 2. OPE1+RV adds the thrust to the hot operating case; OCC RV checks the sustained stress plus the thrust's own stress against 1.33 Sh.");
    d.m
}

fn bellows() -> Model {
    // 24 m between main anchors with an untied bellows in the middle, two guides each side.
    let mut d = table("Expansion joint between anchors", &[(10, 20, 6.0, 0.0, 0.0), (20, 30, 5.6, 0.0, 0.0), (30, 31, 0.4, 0.0, 0.0), (31, 40, 6.0, 0.0, 0.0), (40, 50, 6.0, 0.0, 0.0)]);
    pipe(&mut d, "8", "40", 0.075, 12.0);
    conditions(&mut d, "Steam 200 °C", [200.0, 21.0, 21.0], [10.0, 0.0, 0.0], 15.0);
    // Example values for an 8" single bellows: take yours from the maker.
    let b = crate::model::Bellows { axial: 3e5, lateral: 1.5e6, angular: 6e4, torsion: 0.0, area: 0.045, tied: false, mass: 45.0 };
    let mut e = d.m.elements.iter().find(|e| e.to == 31).expect("joint").clone();
    e.comp = Comp::Bellows(b);
    d.apply(Op::Set { item: Item::Element(e) }).expect("bellows");
    support(&mut d, 10, SupKind::Anchor, "MA-10");
    support(&mut d, 50, SupKind::Anchor, "MA-50");
    for n in [20, 30, 31, 40] {
        support(&mut d, n, SupKind::Rest, &format!("S-{n}"));
        support(&mut d, n, SupKind::Guide, &format!("G-{n}"));
    }
    meta(&mut d, "Expansion joint between anchors", "8\"-ST-201", "8\" Sch 40 insulated steam at 200 °C and 10 bar, 24 m straight between main anchors, with an untied bellows expansion joint in the middle (example rates and an effective area of 0.045 m²) and guides each side of it. The joint takes the thermal growth with a small spring force, but the pressure on its effective area (45 kN at 10 bar) pushes the pipe apart: the main anchors carry that thrust in every case with pressure.");
    d.m
}

/// A vertical two-phase separator: the vessel worked example (example values throughout: the
/// allowable stresses, joint efficiencies and yield strength are to be checked against the code
/// edition and the material certificate).
fn separator() -> Model {
    use crate::vessel::{Part, PartKind, VNozzle, Vessel};
    let mut d = Doc::new(Model::new());
    let noz = |name: &str, part: usize, pos: f64, angle: f64, nps: &str, sch: &str, leg: f64, pad: (f64, f64, f64)| {
        let (od, t) = crate::library::pipe(nps, sch).expect("nozzle size");
        VNozzle { name: name.into(), part, pos, angle, nps: nps.into(), od, t, pipe: true, leg, pad_od: pad.0, pad_t: pad.1, pad_leg: pad.2, proj: 0.25 }
    };
    let head = |name: &str| Part { name: name.into(), kind: PartKind::Ellipsoidal, d: 1.8, len: 0.0, t: 0.016, e: 1.0 };
    let v = Vessel {
        name: "V-101".into(),
        vertical: true,
        p: 16e5,
        temp: K0 + 200.0,
        s: 138e6,
        s_cold: 138e6,
        sy: 260e6,
        corrosion: 0.003,
        under: 0.0,
        rho: 7850.0,
        liquid: 850.0,
        level: 1.8,
        extra: 1500.0,
        parts: vec![head("Bottom head"), Part { name: "Shell".into(), kind: PartKind::Cylinder, d: 1.8, len: 6.0, t: 0.016, e: 0.85 }, head("Top head")],
        nozzles: vec![
            noz("N1 inlet", 1, 4.5, 0.0, "8", "80", 0.010, (0.38, 0.016, 0.010)),
            noz("N2 gas outlet", 2, 0.0, 0.0, "6", "80", 0.008, (0.3, 0.016, 0.008)),
            noz("N3 liquid outlet", 0, 0.0, 0.0, "4", "160", 0.008, (0.18, 0.016, 0.008)),
            // The manway's neck is rolled from 16 mm plate (no pipe under-tolerance).
            VNozzle { name: "M1 manway".into(), part: 1, pos: 1.2, angle: 180.0, nps: String::new(), od: 0.54, t: 0.016, pipe: false, leg: 0.012, pad_od: 0.84, pad_t: 0.016, pad_leg: 0.012, proj: 0.3 },
        ],
        saddles: None,
        // On a 2.5 m skirt with a base ring and twelve M30 anchor bolts (root diameter 26.2 mm),
        // in a 1 kPa wind and 0.15 g of static seismic (example values).
        skirt: Some(crate::vessel::Skirt { h: 2.5, t: 0.012, ring_od: 2.25, ring_id: 1.55, ring_t: 0.03, bolts: 12, bolt_d: 0.0262, bc: 2.05, bolt_s: 140e6, bearing: 7e6 }),
        seis: 0.15,
        // The manway's bolted cover (example dimensions and gasket factors: take yours from the
        // flange standard and the gasket maker).
        flanges: vec![crate::vessel::BodyFlange { name: "M1 cover".into(), ..Default::default() }],
        ..Default::default()
    };
    let mut st = d.m.settings.clone();
    st.wind_q = 1000.0;
    d.apply(Op::SetSettings { s: st, units: "SI".into() }).expect("wind");
    d.apply(Op::SetVessels { list: vec![v] }).expect("vessel");
    meta(&mut d, "Vertical separator V-101", "V-101", "A vertical two-phase separator: 1800 mm inside diameter, 6000 mm between tangent lines, 2:1 semi-ellipsoidal heads, an 8\" inlet, 6\" gas and 4\" liquid outlets and a 20\" manway with reinforcing pads; 16 bar and 200 °C, 3 mm corrosion allowance; carbon steel plate with an allowable stress of 138 MPa hot and cold and a yield of 260 MPa (example values); shell joint efficiency 0.85 (spot radiography), seamless heads 1.0. The liquid (850 kg/m³) stands 1.8 m above the bottom tangent line in operation; internals and platforms add 1500 kg. It stands on a 2.5 m skirt with a base ring and twelve M30 anchor bolts, in a 1 kPa wind (Analysis settings) and 0.15 g of static seismic. Open the Vessels view for the thicknesses, the MAWP, the test pressure and the weights.");
    d.m
}

/// A horizontal drum on two saddles: the second vessel example (example values throughout).
fn drum() -> Model {
    use crate::vessel::{Part, PartKind, Saddles, VNozzle, Vessel};
    let mut d = Doc::new(Model::new());
    let head = |name: &str| Part { name: name.into(), kind: PartKind::Ellipsoidal, d: 2.4, len: 0.0, t: 0.014, e: 1.0 };
    let noz = |name: &str, part: usize, pos: f64, angle: f64, nps: &str, sch: &str, pad: (f64, f64)| {
        let (od, t) = crate::library::pipe(nps, sch).expect("nozzle size");
        VNozzle { name: name.into(), part, pos, angle, nps: nps.into(), od, t, pipe: true, leg: 0.008, pad_od: pad.0, pad_t: pad.1, pad_leg: if pad.0 > 0.0 { 0.008 } else { 0.0 }, proj: 0.25 }
    };
    let v = Vessel {
        name: "D-201".into(),
        vertical: false,
        p: 10e5,
        temp: K0 + 150.0,
        s: 138e6,
        s_cold: 138e6,
        sy: 260e6,
        corrosion: 0.003,
        under: 0.0,
        rho: 7850.0,
        liquid: 1000.0,
        level: 1.6,
        extra: 1000.0,
        parts: vec![head("Left head"), Part { name: "Shell".into(), kind: PartKind::Cylinder, d: 2.4, len: 9.0, t: 0.014, e: 0.85 }, head("Right head")],
        nozzles: vec![noz("N1 inlet", 1, 1.5, 0.0, "8", "80", (0.38, 0.014)), noz("N2 outlet", 1, 7.5, 180.0, "6", "80", (0.3, 0.014)), noz("N3 vent", 2, 0.0, 0.0, "2", "160", (0.0, 0.0))],
        saddles: Some(Saddles { a: 0.6, theta: 120.0, b: 0.3, welded: true, rings: false }),
        ..Default::default()
    };
    d.apply(Op::SetVessels { list: vec![v] }).expect("vessel");
    meta(&mut d, "Horizontal drum D-201 on saddles", "D-201", "A horizontal drum: 2400 mm inside diameter, 9000 mm between tangent lines, 2:1 semi-ellipsoidal heads, 14 mm plate; 10 bar and 150 °C, 3 mm corrosion allowance; allowable stress 138 MPa, yield 260 MPa (example values); shell joint efficiency 0.85, seamless heads. Two 120° saddles 300 mm wide, welded, 600 mm from the tangent lines (within R/2, so the heads stiffen the shell there). Liquid 1.6 m deep in operation; the saddles carry the heavier of the operating weight and the weight full of water. Open the Vessels view for the shell, heads, nozzles and the saddle stresses by Zick's method.");
    d.m
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
    // The valve's flanged ends checked for leakage (example values: a 180 mm gasket reaction
    // diameter and a 40 bar rating at temperature; take yours from your flange data).
    let valve = d.m.elements.iter().find(|e| matches!(e.comp, Comp::Rigid { .. })).expect("valve").id;
    d.apply(Op::SetFlanges { list: vec![crate::flange::Flange { element: valve, gasket: 0.18, rating: 40e5 }] }).expect("flange check");
    meta(&mut d, "Pump discharge with an expansion loop", "6\"-HW-101", "6\" Sch 40 carbon steel, hot water at 150 °C and 20 bar, from the pump discharge nozzle (which grows 1.2 mm upwards when hot) up through a gate valve, then east with an expansion loop to an anchor at the header. The nozzle loads are checked against allowables entered for the pump; the valve's flanges are checked for leakage (example gasket diameter and rating); the hydrotest case (30 bar, full of water) gives the support loads during the test.");
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
    // Both directions: with one-way rests, gapped guides and friction, the reversed loads differ.
    st.occ_both = true;
    d.apply(Op::SetSettings { s: st, units: "SI".into() }).expect("occasional loads");
    nozzle(&mut d, 70, "T-301 inlet", [0.0, 0.0, -1.0], [12000.0, 9000.0, 12000.0, 12000.0]);
    machine(&mut d, "T-301", crate::equip::EquipKind::Turbine, 1.0, [1.0, 0.0, 0.0]);
    meta(&mut d, "Steam line on a pipe rack", "8\"-ST-301", "8\" Sch 40 carbon steel, steam at 250 °C and 15 bar, 75 mm of insulation with aluminium cladding. 24 m along a pipe rack on one-way rests with friction (μ 0.3), with guides (3 mm gaps) at 10 and 30 and a line stop at 20 that holds the line along its axis, then 6 m down and 6 m north (on a rest at 65) to a turbine inlet that grows 2 mm up when hot. The turbine's nozzle fails the turbine criteria (Results › Equipment): the line is too stiff near it, so try a loop or a spring near the turbine. Wind (0.8 kPa from the south) and 0.15 g of static seismic are checked as occasional cases, in both directions.");
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
    // The tank shell gives at the nozzle (example values; take yours from the tank's maker or your
    // own calculation): 35 kN/mm radially, 60 MN·m/rad in longitudinal and 120 MN·m/rad in
    // circumferential bending; rigid in shear and torsion. The tank's axis is vertical.
    let mut z = d.m.nozzles[0].clone();
    z.stiff = [3.5e7, 0.0, 0.0, 6e7, 1.2e8];
    d.apply(Op::Set { item: Item::Nozzle(z) }).expect("nozzle stiffness");
    // A spring near the tank so the settlement does not hang the line off the nozzle.
    support(&mut d, 40, SupKind::Hanger { k: 0.0, cold: 0.0 }, "SH-40");
    support(&mut d, 50, SupKind::Rest, "S-50");
    support(&mut d, 70, SupKind::Anchor, "A-70");
    meta(&mut d, "Tank outlet with settlement", "10\"-CR-401", "10\" Std carbon steel crude oil at 65 °C from a tank shell nozzle, through a gate valve, round two bends to an anchor. Case 2 is the tank settling 12 mm: a displacement range the piping must take without overloading the nozzle, which is why the first support is a spring hanger, sized by Analyze from the spring table. The tank shell's flexibility at the nozzle is included (example stiffnesses).");
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
