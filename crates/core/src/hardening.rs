//! FAMILY §11, the hardening standard: a failed edit leaves the model exactly as it was; random
//! models and random edits never panic and keep every rule; damaged files open as far as possible
//! or give a clear error; a speed gate on a large model.
#![cfg(test)]

use crate::doc::{Doc, Item, Op};
use crate::model::{self, Comp, Element, Model, SupKind, Support};

/// A small random generator (xorshift), so the tests need no extra crate and repeat exactly.
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    pub fn int(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    pub fn f(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// A random piping model: a run of elements along the axes with branches, supports and loads.
pub fn random_model(r: &mut Rng, n: usize) -> Model {
    let mut d = Doc::new(Model::new());
    let (p, m, c) = (d.m.pipes[0].id, d.m.materials[0].id, d.m.conds[0].id);
    let mut nums = vec![10u32];
    let mut next = 20u32;
    for _ in 0..n {
        let from = nums[r.int(nums.len() as u64) as usize];
        let axis = r.int(3) as usize;
        let mut dv = [0.0; 3];
        dv[axis] = (0.5 + 4.0 * r.f()) * if r.int(2) == 0 { 1.0 } else { -1.0 };
        let el = Element { from, to: next, d: dv, pipe: p, material: m, cond: c, comp: Comp::Pipe, ..Default::default() };
        if d.apply(Op::Add { item: Item::Element(el), after: None }).is_ok() {
            nums.push(next);
            next += 10;
        }
    }
    for _ in 0..(n / 3).max(1) {
        let node = nums[r.int(nums.len() as u64) as usize];
        let kind = match r.int(4) {
            0 => SupKind::Anchor,
            1 => SupKind::Rest,
            2 => SupKind::Guide,
            _ => SupKind::Restraint { dir: [r.f() - 0.5, r.f() - 0.5, r.f() - 0.5 + 1e-3] },
        };
        // Some rests one-way and some supports with a gap, so the iteration is tried on every shape.
        let (one_way, gap, mu) = (r.int(2) == 0, if r.int(4) == 0 { r.f() * 0.01 } else { 0.0 }, if r.int(3) == 0 { 0.1 + 0.3 * r.f() } else { 0.0 });
        let _ = d.apply(Op::Add { item: Item::Support(Support { node, kind, one_way, gap, mu, ..Default::default() }), after: None });
    }
    d.m
}

#[test]
fn failed_edits_change_nothing() {
    let mut r = Rng::new(7);
    let mut d = Doc::new(random_model(&mut r, 12));
    let before = d.m.clone();
    let bad = [
        Op::Add { item: Item::Element(Element { from: 10, to: 10, ..Default::default() }), after: None },
        Op::Add { item: Item::Element(Element { from: 10, to: 999, d: [f64::NAN, 0.0, 0.0], pipe: before.pipes[0].id, material: before.materials[0].id, cond: before.conds[0].id, ..Default::default() }), after: None },
        Op::Add { item: Item::Element(Element { from: 10, to: 998, d: [1.0, 0.0, 0.0], pipe: 4242, ..Default::default() }), after: None },
        Op::Delete { ids: vec![before.pipes[0].id] },
        Op::Add { item: Item::Support(Support { node: 4242, ..Default::default() }), after: None },
        Op::Renumber { start: 0, step: 10 },
        Op::Split { id: before.elements[0].id, at: 1.5 },
        Op::Batch(vec![Op::Renumber { start: 1, step: 1 }, Op::Delete { ids: vec![before.materials[0].id] }]),
    ];
    for op in bad {
        assert!(d.apply(op.clone()).is_err(), "{op:?} should fail");
        assert_eq!(d.m, before, "a failed {op:?} changed the model");
    }
}

#[test]
fn random_models_keep_their_rules_and_round_trip() {
    // Pipe lying on pipe (two elements leaving a node the same way) is not real piping: the file
    // reader may refuse it. Every other random model must read back.
    let overlaps = |m: &Model| {
        let out = |e: &Element, n: u32| { let l = e.length(); let s = if e.from == n { 1.0 } else { -1.0 }; e.d.map(|v| s * v / l) };
        m.nodes.iter().any(|n| {
            let at: Vec<&Element> = m.elements.iter().filter(|e| e.from == n.num || e.to == n.num).collect();
            at.iter().enumerate().any(|(i, a)| at[i + 1..].iter().any(|b| { let (u, v) = (out(a, n.num), out(b, n.num)); u[0] * v[0] + u[1] * v[1] + u[2] * v[2] > 0.999 }))
        })
    };
    let (mut read_back, mut real) = (0, 0);
    for seed in 0..60 {
        let mut r = Rng::new(seed);
        let m = random_model(&mut r, 3 + seed as usize % 25);
        m.check().unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        let (back, warn) = model::from_json(&model::to_json(&m)).unwrap();
        assert!(warn.is_empty(), "seed {seed}: {warn:?}");
        assert_eq!(back, m, "seed {seed}: the file does not round-trip");
        // A piping component file goes out and reads back (or explains), the same length of piping.
        let text = crate::export::pcf(&m).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        let back = crate::import::pcf(&text);
        if !overlaps(&m) {
            real += 1;
            assert!(back.is_ok(), "seed {seed}: {:?}", back.err());
        }
        if let Ok(r) = back {
            read_back += 1;
            let total = |m: &Model| m.elements.iter().map(|e| e.length()).sum::<f64>();
            assert!((total(&m) - total(&r.model)).abs() < 1e-3 * m.elements.len() as f64, "seed {seed}: {} vs {}", total(&m), total(&r.model));
        }
    }
    assert!(real >= 3 && read_back >= real, "{real} random models without overlaps, {read_back} read back");
}

#[test]
fn damaged_files_open_or_explain() {
    let mut r = Rng::new(99);
    let m = random_model(&mut r, 20);
    let text = model::to_json(&m);
    let bytes = text.as_bytes();
    for i in 0..400 {
        // Cut short, garbled bytes, swapped values.
        let mut b = bytes.to_vec();
        match i % 3 {
            0 => b.truncate((r.int(b.len() as u64)) as usize),
            1 => {
                for _ in 0..1 + r.int(6) {
                    let k = r.int(b.len() as u64) as usize;
                    b[k] = b"0-9.e,:[]{}\"x "[r.int(14) as usize];
                }
            }
            _ => {
                let s = String::from_utf8_lossy(&b).replace("\"from\": 10", "\"from\": 20").replace("\"pipe\": 1", "\"pipe\": -1").replace("\"next\":", "\"next\": -5, \"x\":");
                b = s.into_bytes();
            }
        }
        let s = String::from_utf8_lossy(&b);
        if let Ok((m2, _)) = model::from_json(&s) {
            m2.check().unwrap_or_else(|e| panic!("case {i}: a repaired file breaks a rule: {e}"));
        }
    }
    assert!(model::from_json("[1,2]").is_err() && model::from_json("{\"format\": 99}").is_err());
}

#[test]
fn speed_gate_on_a_large_model() {
    let t = std::time::Instant::now();
    let mut r = Rng::new(3);
    let m = random_model(&mut r, 600);
    let mut d = Doc::new(m);
    let id = d.m.elements[0].id;
    let e = d.m.elements[0].clone();
    for i in 0..50 {
        d.apply(Op::Set { item: Item::Element(Element { d: [1.0 + i as f64 * 0.01, e.d[1], e.d[2]], ..e.clone() }) }).unwrap();
    }
    let _ = id;
    let limit = if cfg!(debug_assertions) { 20.0 } else { 4.0 };
    assert!(t.elapsed().as_secs_f64() < limit, "{:?}", t.elapsed());
}

#[test]
fn random_models_solve_or_explain_and_always_balance() {
    let mut dynamic = 0;
    let mut buried = 0;
    for seed in 0..80 {
        let mut r = Rng::new(1000 + seed);
        let mut m = random_model(&mut r, 2 + seed as usize % 30);
        // Heat some of them and give a few bends.
        if seed % 2 == 0 {
            m.conds[0].t[0] = model::K0 + 150.0 + seed as f64;
            m.conds[0].p[0] = 5e5;
        }
        if seed % 5 == 0 {
            m.settings.wind_q = 500.0 + seed as f64;
            m.settings.wind_dir = [1.0, 0.0, 0.3];
            m.settings.seis = [0.2, -0.1, 0.15];
        }
        if seed % 3 == 0 {
            for e in m.elements.iter_mut().step_by(3) {
                e.comp = Comp::Bend { bend: model::BendKind::Short, r: 0.0 };
            }
        }
        // Every piping code, and a cold spring on some straight pipe.
        m.settings.code = [crate::code::PipingCode::Process, crate::code::PipingCode::Power, crate::code::PipingCode::European][seed as usize % 3];
        if seed % 4 == 2 {
            if let Some(e) = m.elements.iter_mut().find(|e| matches!(e.comp, Comp::Pipe) && e.length() > 0.1) {
                e.cold = 0.02 * e.length();
            }
        }
        // Buried pipe on some: a soil under every other element.
        if seed % 7 == 3 {
            m.soils = vec![crate::soil::Soil { id: 1, phi: 25.0 + seed as f64 % 10.0, c: if seed % 2 == 0 { 0.0 } else { 20e3 }, ..Default::default() }];
            for e in m.elements.iter_mut().step_by(2) {
                e.soil = 1;
            }
        }
        // And dynamics on some: modes, a spectrum and a harmonic load at a random node.
        if seed % 4 == 1 {
            m.settings.modes = 1 + seed as u32 % 8;
            m.settings.spectrum = vec![[1.0, 0.2], [10.0, 0.4], [33.0, 0.15]];
            m.settings.rs_dir = [1.0, 0.5, 1.0];
            let node = m.nodes[seed as usize % m.nodes.len()].num;
            m.settings.harmonic = Some(model::Harmonic { node, f: [100.0, 200.0, 0.0], m: [0.0, 0.0, 50.0], freq: 0.5 + seed as f64 / 4.0 });
        }
        crate::cases::rebuild(&mut m);
        match crate::solve::analyze(&m) {
            Ok(res) => {
                assert!(res.modes.iter().all(|x| x.freq.is_finite() && x.freq >= 0.0), "seed {seed}");
                dynamic += res.cases.iter().filter(|c| c.name == "HARM" || c.name == "OCC SPEC").count();
                buried += res.cases.iter().filter(|c| !c.soil.is_empty()).count();
                for c in &res.cases {
                    assert!(c.imbalance < 1e-6, "seed {seed}, case {}: imbalance {:e} {:?}", c.name, c.imbalance, res.warnings);
                    assert!(c.disp.iter().flatten().all(|v| v.is_finite()), "seed {seed}");
                    assert!(c.stresses.iter().all(|s| s.stress.is_finite() && s.stress >= 0.0), "seed {seed}");
                }
            }
            Err(e) => assert!(!e.is_empty()),
        }
    }
    assert!(dynamic >= 10, "only {dynamic} dynamic cases came out of the random models");
    assert!(buried >= 10, "only {buried} solved cases with soil came out of the random models");
}

#[test]
fn a_free_model_is_a_mechanism_not_a_crash() {
    let mut r = Rng::new(5);
    let mut m = random_model(&mut r, 6);
    m.supports.clear();
    let e = crate::solve::analyze(&m).unwrap_err();
    assert!(e.contains("without resistance"), "{e}");
    assert!(crate::solve::analyze(&Model::new()).is_err());
}

#[test]
fn speed_gate_on_a_large_analysis() {
    let mut r = Rng::new(11);
    let mut m = random_model(&mut r, 1500);
    m.conds[0].t[0] = model::K0 + 200.0;
    crate::cases::rebuild(&mut m);
    let n0 = m.elements[0].from;
    m.supports.push(Support { id: m.next, node: n0, ..Default::default() });
    m.next += 1;
    // A 3 km random chain held at one end is too floppy for sliding friction to be meaningful (the
    // solver's mechanism check trips on it): this gate times the solve, friction has its own below.
    m.supports.iter_mut().for_each(|s| s.mu = 0.0);
    let t = std::time::Instant::now();
    let res = crate::solve::analyze(&m).unwrap();
    let limit = if cfg!(debug_assertions) { 20.0 } else { 3.0 };
    assert!(t.elapsed().as_secs_f64() < limit, "{:?} for {} points", t.elapsed(), res.geom.points.len());
    let _ = SupKind::Anchor;
}

#[test]
fn speed_gate_with_friction() {
    // 300 random elements with one-way rests, gaps and friction, anchored at the start and heated:
    // the iteration must finish quickly and balance.
    let mut r = Rng::new(23);
    let mut m = random_model(&mut r, 300);
    m.conds[0].t[0] = model::K0 + 200.0;
    crate::cases::rebuild(&mut m);
    let n0 = m.elements[0].from;
    m.supports.push(Support { id: m.next, node: n0, ..Default::default() });
    m.next += 1;
    assert!(m.supports.iter().any(|s| s.mu > 0.0));
    let t = std::time::Instant::now();
    let res = crate::solve::analyze(&m).unwrap();
    assert!(res.cases.iter().all(|c| c.imbalance < 1e-6));
    let limit = if cfg!(debug_assertions) { 30.0 } else { 5.0 };
    assert!(t.elapsed().as_secs_f64() < limit, "{:?}", t.elapsed());
}

#[test]
fn every_example_solves_balances_and_has_no_warnings() {
    for e in crate::examples::list() {
        let m = crate::examples::get(e.id).unwrap();
        if m.elements.is_empty() {
            continue;
        }
        let r = crate::solve::analyze(&m).unwrap_or_else(|x| panic!("{}: {x}", e.id));
        assert!(r.warnings.is_empty(), "{}: {:?}", e.id, r.warnings);
        for c in &r.cases {
            assert!(c.imbalance < 1e-9, "{} {}: {:e}", e.id, c.name, c.imbalance);
        }
        let (back, w) = model::from_json(&model::to_json(&m)).unwrap();
        assert!(w.is_empty() && back == m, "{}", e.id);
    }
}



#[test]
fn speed_gate_on_a_large_rack_with_friction() {
    // 160 bays (about 1440 elements and 1600 supports, one-way rests with friction, guides, line
    // stops): the whole analysis, every case settled. Found in 0.2.2: it took 16 s before the
    // estimated flexibility.
    let m = crate::examples::big(160);
    let t = std::time::Instant::now();
    let r = crate::solve::analyze(&m).unwrap();
    let limit = if cfg!(debug_assertions) { 15.0 } else { 1.0 };
    assert!(t.elapsed().as_secs_f64() < limit, "{:?} for {} elements", t.elapsed(), m.elements.len());
    assert!(r.warnings.is_empty() && r.cases.iter().all(|c| c.imbalance < 1e-9));
}

#[test]
fn random_vessels_check_or_explain_and_round_trip() {
    use crate::vessel::{check, BodyFlange, Legs, Lifting, Part, PartKind, Saddles, Skirt, VNozzle, Vessel};
    let mut ok = 0;
    for seed in 0..200 {
        let mut r = Rng::new(5000 + seed);
        let d = 0.5 + 3.0 * r.f();
        let vertical = r.int(2) == 0;
        let head = |r: &mut Rng| match r.int(4) {
            0 => PartKind::Ellipsoidal,
            1 => PartKind::Hemispherical,
            2 => PartKind::Torispherical { crown: 0.8 + 0.2 * r.f(), knuckle: 0.06 + 0.1 * r.f() },
            _ => PartKind::Ellipsoidal,
        };
        let t = 0.004 + 0.04 * r.f();
        let mut parts = vec![Part { name: "H1".into(), kind: head(&mut r), d, len: 0.0, t, e: 0.7 + 0.3 * r.f() }];
        let mut dd = d;
        for k in 0..1 + r.int(3) {
            if r.int(4) == 0 {
                let d_end = dd * (0.6 + 0.3 * r.f());
                parts.push(Part { name: format!("C{k}"), kind: PartKind::Cone { d_end }, d: dd, len: (dd - d_end) / 2.0 / 0.4, t, e: 1.0 });
                dd = d_end;
            } else {
                parts.push(Part { name: format!("S{k}"), kind: PartKind::Cylinder, d: dd, len: 0.5 + 8.0 * r.f(), t, e: 0.85 });
            }
        }
        parts.push(Part { name: "H2".into(), kind: head(&mut r), d: dd, len: 0.0, t, e: 1.0 });
        let n = parts.len();
        let (od, wall) = crate::library::pipe("4", "80").unwrap();
        let nozzles = (0..r.int(3)).map(|k| VNozzle { name: format!("N{k}"), part: r.int(n as u64) as usize, pos: r.f(), angle: 90.0 * r.int(4) as f64, nps: "4".into(), od, t: wall, pipe: true, leg: 0.006, pad_od: if r.int(2) == 0 { 0.25 } else { 0.0 }, pad_t: 0.01, pad_leg: 0.006, proj: 0.2, link: 0 }).collect();
        let v = Vessel {
            name: format!("R{seed}"),
            vertical,
            p: 2e6 * r.f(),
            liquid: 1000.0 * r.f(),
            level: d * r.f(),
            parts,
            nozzles,
            saddles: (!vertical && r.int(2) == 0).then(|| Saddles { a: 0.2 + 0.5 * r.f(), theta: 120.0 + 60.0 * r.f(), b: 0.2, welded: r.int(2) == 0, rings: r.int(4) == 0 }),
            skirt: (vertical && r.int(3) == 0).then(Skirt::default),
            legs: None,
            seis: 0.3 * r.f(),
            flanges: if r.int(3) == 0 { vec![BodyFlange::default()] } else { vec![] },
            lifting: (r.int(2) == 0).then(Lifting::default),
            pe: if r.int(3) == 0 { 1e5 } else { 0.0 },
            ring_spacing: if r.int(2) == 0 { 2.0 } else { 0.0 },
            ring_i: 1e-6,
            ..Default::default()
        };
        let v = if vertical && v.skirt.is_none() && r.int(3) == 0 { Vessel { legs: Some(Legs::default()), ..v } } else { v };
        // Half of them by the European rules.
        let v = if seed % 2 == 0 { Vessel { rules: crate::vessel::Rules::European, ..v } } else { v };
        match check(&v) {
            Ok(res) => {
                ok += 1;
                let nums = res.parts.iter().flat_map(|p| [p.t_req, p.mawp, p.map, p.test_stress, p.volume, p.metal]).chain([res.mawp, res.map, res.test, res.volume, res.empty, res.operating])
                    .chain(res.nozzles.iter().flat_map(|z| [z.a_req, z.a_avail, z.t_neck_req]))
                    .chain(res.external.iter().flat_map(|x| [x.allow, x.elastic]))
                    .chain(res.saddles.iter().flat_map(|s| s.rows.iter().map(|x| x.stress)))
                    .chain(res.support.iter().flat_map(|s| s.sections.iter().flat_map(|x| [x.tension, x.compression])))
                    .chain(res.lifting.iter().chain(&res.legs).map(|x| x.value));
                for x in nums {
                    assert!(x.is_finite(), "seed {seed}: a result is not a number: {x}");
                }
            }
            Err(e) => assert!(!e.is_empty(), "seed {seed}"),
        }
        // In a model, it round-trips through the file.
        let mut m = Model::new();
        if v.check_data().is_ok() {
            m.vessels.push(v.clone());
            let (back, warn) = model::from_json(&model::to_json(&m)).unwrap();
            assert!(warn.is_empty(), "seed {seed}: {warn:?}");
            assert_eq!(back.vessels, m.vessels, "seed {seed}");
        }
    }
    assert!(ok >= 100, "only {ok} of 200 random vessels could be checked");
}

#[test]
fn speed_gate_for_natural_modes() {
    // The 160-bay rack (about 3 200 points): 20 modes in a few seconds. Its lowest modes are the
    // loops rocking about the line's axis, held only by the pipe's torsion between the end anchors
    // (rests and guides do not hold a pipe's rotation): about 2.4 Hz·bay / bays.
    let mut m = crate::examples::big(160);
    m.settings.modes = 20;
    let t = std::time::Instant::now();
    let r = crate::solve::analyze(&m).unwrap();
    let s = t.elapsed().as_secs_f64();
    assert_eq!(r.modes.len(), 20);
    assert!(r.modes[0].freq > 0.0 && r.modes.iter().all(|x| x.freq.is_finite()));
    eprintln!("modes: {s:.2} s, f1 {:.3} Hz", r.modes[0].freq);
    assert!(s < 30.0, "{s} s");
}
