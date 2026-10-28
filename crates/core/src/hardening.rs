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
        let _ = d.apply(Op::Add { item: Item::Support(Support { node, kind, ..Default::default() }), after: None });
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
    for seed in 0..60 {
        let mut r = Rng::new(seed);
        let m = random_model(&mut r, 3 + seed as usize % 25);
        m.check().unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        let (back, warn) = model::from_json(&model::to_json(&m)).unwrap();
        assert!(warn.is_empty(), "seed {seed}: {warn:?}");
        assert_eq!(back, m, "seed {seed}: the file does not round-trip");
    }
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
