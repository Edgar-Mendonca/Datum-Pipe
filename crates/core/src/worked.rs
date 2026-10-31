//! Hand-worked cases (FAMILY §11, trust in the results): one small model for each component and
//! support kind, with the expected values worked out by hand before running.
#![cfg(test)]

use crate::doc::{Doc, Item, Op};
use crate::elements::KAPPA;
use crate::examples::table;
use crate::model::{Comp, Load, Nozzle, Pipe, RigidKind, SupKind, Support, TeeKind, K0};
use crate::solve::{analyze, Results};

fn no_g(d: &mut Doc) {
    let mut s = d.m.settings.clone();
    s.g = 0.0;
    d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
}
fn sup(d: &mut Doc, node: u32, kind: SupKind) {
    d.apply(Op::Add { item: Item::Support(Support { node, kind, ..Default::default() }), after: None }).unwrap();
}
fn load(d: &mut Doc, node: u32, f: [f64; 3]) {
    d.apply(Op::Add { item: Item::Load(Load { node, f, m: [0.0; 3], ..Default::default() }), after: None }).unwrap();
}
fn comp(d: &mut Doc, to: u32, c: Comp) {
    let mut e = d.m.elements.iter().find(|e| e.to == to).unwrap().clone();
    e.comp = c;
    d.apply(Op::Set { item: Item::Element(e) }).unwrap();
}
fn case<'a>(r: &'a Results, n: &str) -> &'a crate::solve::CaseResult {
    r.cases.iter().find(|c| c.name == n).unwrap()
}
fn at(r: &Results, c: &str, node: u32) -> [f64; 6] {
    case(r, c).disp[r.geom.node_point[&node]]
}
fn react(r: &Results, c: &str, node: u32) -> [f64; 6] {
    let mut s = [0.0; 6];
    for x in case(r, c).reactions.iter().filter(|x| x.node == node) {
        for i in 0..6 {
            s[i] += x.f[i];
        }
    }
    s
}
/// 6" Sch 40 carbon steel: E at 21 °C, I, A, G.
fn props(d: &Doc) -> (f64, f64, f64, f64) {
    let p = &d.m.pipes[0];
    let e = crate::library::interp(&d.m.materials[0].e, d.m.settings.ambient);
    let id = p.od - 2.0 * p.wall;
    let pi = std::f64::consts::PI;
    (e, pi / 64.0 * (p.od.powi(4) - id.powi(4)), pi / 4.0 * (p.od.powi(2) - id.powi(2)), e / 2.6)
}
fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol * b.abs().max(1e-30)
}

#[test]
fn a_spring_under_a_cantilever_tip() {
    // 3 m cantilever, spring k = 1e6 N/m under the tip, 10 kN down: δ = P / (k + 1/c), with the
    // cantilever's compliance c = L³/3EI + L/κGA.
    let mut d = table("spring", &[(10, 20, 3.0, 0.0, 0.0)]);
    no_g(&mut d);
    sup(&mut d, 10, SupKind::Anchor);
    sup(&mut d, 20, SupKind::Spring { dir: [0.0, 1.0, 0.0], k: 1e6 });
    load(&mut d, 20, [0.0, -10e3, 0.0]);
    let (e, i, a, g) = props(&d);
    let c = 27.0 / (3.0 * e * i) + 3.0 / (KAPPA * g * a);
    let want = -10e3 / (1e6 + 1.0 / c);
    let r = analyze(&d.m).unwrap();
    assert!(close(at(&r, "OPE1", 20)[1], want, 1e-10), "{} {want}", at(&r, "OPE1", 20)[1]);
    // The spring carries k δ.
    assert!(close(react(&r, "OPE1", 20)[1], -1e6 * want, 1e-9));
}

#[test]
fn an_imposed_settlement_between_anchors() {
    // 4 m between anchors, the far one settles 10 mm: F = 12 EI δ / (L³ (1 + Φ)), Φ = 12 EI / (κ G A L²).
    let mut d = table("settle", &[(10, 20, 4.0, 0.0, 0.0)]);
    no_g(&mut d);
    sup(&mut d, 10, SupKind::Anchor);
    sup(&mut d, 20, SupKind::Anchor);
    let mut s = d.m.supports[1].clone();
    s.disp[0][1] = -0.010;
    d.apply(Op::Set { item: Item::Support(s) }).unwrap();
    let (e, i, a, g) = props(&d);
    let l = 4.0f64;
    let phi = 12.0 * e * i / (KAPPA * g * a * l * l);
    let f = 12.0 * e * i * 0.010 / (l.powi(3) * (1.0 + phi));
    let r = analyze(&d.m).unwrap();
    // The anchor pulls the pipe down with it.
    assert!(close(react(&r, "OPE1", 20)[1], -f, 1e-10), "{:?} {f}", react(&r, "OPE1", 20));
    assert!(close(at(&r, "OPE1", 20)[1], -0.010, 1e-12));
    // The sustained case does not see it; the expansion range does.
    assert!(react(&r, "SUS", 20)[1].abs() < 1e-6 && close(react(&r, "EXP1", 20)[1], -f, 1e-10));
}

#[test]
fn guides_line_stops_and_skewed_restraints_hold_only_their_direction() {
    // A cantilever along X with a force (1, 2, −1) kN at the tip, held there by one support.
    for (kind, free, held) in [
        (SupKind::Guide, [true, true, false], "Z"),
        (SupKind::Linestop, [false, true, true], "X"),
        (SupKind::Rest, [true, false, true], "Y"),
        (SupKind::Restraint { dir: [0.0, 1.0, 1.0] }, [true, true, true], "skew"),
    ] {
        let mut d = table("guide", &[(10, 20, 3.0, 0.0, 0.0)]);
        no_g(&mut d);
        sup(&mut d, 10, SupKind::Anchor);
        sup(&mut d, 20, kind.clone());
        load(&mut d, 20, [1e3, 2e3, -1e3]);
        let r = analyze(&d.m).unwrap();
        let u = at(&r, "OPE1", 20);
        for k in 0..3 {
            if free[k] {
                assert!(u[k].abs() > 1e-7, "{held}: direction {k} should move: {u:?}");
            } else {
                assert!(u[k].abs() < 1e-12, "{held}: direction {k} should be held: {u:?}");
            }
        }
        if held == "skew" {
            // Along (0, 1, 1)/√2 it hardly moves (a stiff spring), across it, it does.
            let along = (u[1] + u[2]) / 2f64.sqrt();
            assert!(along.abs() < 1e-6 * u[1].abs().max(1e-9) + 1e-9, "{u:?}");
        }
        assert!(case(&r, "OPE1").imbalance < 1e-9);
    }
}

#[test]
fn a_valve_and_a_constant_support_carry_their_weight() {
    // 6 m between anchors with a 200 kg valve in the middle (1 m long), empty pipe: the anchors carry
    // the pipe's weight over 5 m plus 200 kg, and the valve's 1 m of pipe metal is replaced by it.
    let mut d = table("valve", &[(10, 20, 2.5, 0.0, 0.0), (20, 30, 1.0, 0.0, 0.0), (30, 40, 2.5, 0.0, 0.0)]);
    comp(&mut d, 30, Comp::Rigid { rigid: RigidKind::Valve, mass: 200.0 });
    sup(&mut d, 10, SupKind::Anchor);
    sup(&mut d, 40, SupKind::Anchor);
    let w = crate::solve::weight(&d.m, d.m.pipes[0].id, d.m.materials[0].rho, false);
    let want = w * 5.0 + 200.0 * 9.806_65;
    let r = analyze(&d.m).unwrap();
    let fy = react(&r, "SUS", 10)[1] + react(&r, "SUS", 40)[1];
    assert!(close(fy, want, 1e-12), "{fy} {want}");
    // A constant-force support of 1 kN at the valve takes 1 kN off the anchors in every weight case.
    sup(&mut d, 20, SupKind::Constant { load: 1000.0 });
    let r = analyze(&d.m).unwrap();
    let fy2 = react(&r, "SUS", 10)[1] + react(&r, "SUS", 40)[1];
    assert!(close(fy2, want - 1000.0, 1e-12) && close(react(&r, "SUS", 20)[1], 1000.0, 1e-12));
    assert!(react(&r, "EXP1", 20)[1].abs() < 1e-9);
}

#[test]
fn a_reducer_tapers_between_its_two_sizes() {
    // A 1 m concentric reducer 6" Sch 40 to 4" Sch 40, anchored at the big end, pulled at the small
    // end: the stretch is ∫ P / (E A(s)) ds with A from the diameter and wall varying linearly
    // (here by a 2000-step midpoint sum, independent of the element's Gauss points).
    let mut d = table("reducer", &[(10, 20, 1.0, 0.0, 0.0)]);
    no_g(&mut d);
    let small = Pipe::std(0, "4", "40");
    let o = d.apply(Op::Add { item: Item::Pipe(small.clone()), after: None }).unwrap();
    comp(&mut d, 20, Comp::Reducer { to: o[0] });
    sup(&mut d, 10, SupKind::Anchor);
    load(&mut d, 20, [10e3, 0.0, 0.0]);
    let (e, ..) = props(&d);
    let big = &d.m.pipes[0];
    let n = 2000;
    let mut stretch = 0.0;
    for k in 0..n {
        let s = (k as f64 + 0.5) / n as f64;
        let (od, t) = (big.od + (small.od - big.od) * s, big.wall + (small.wall - big.wall) * s);
        let a = std::f64::consts::PI / 4.0 * (od * od - (od - 2.0 * t).powi(2));
        stretch += 10e3 / (e * a) / n as f64;
    }
    let r = analyze(&d.m).unwrap();
    assert!(close(at(&r, "OPE1", 20)[0], stretch, 1e-6), "{} {stretch}", at(&r, "OPE1", 20)[0]);
    // Its SIF: a 25 mm taper over 1 m is a 0.7° cone, so i = 0.5 + 0.01 α √(D2/t2) is under 1: 1.0.
    let s = &case(&r, "SUS").stresses[0];
    assert!((s.ii - 1.0).abs() < 1e-12);
}

#[test]
fn a_welding_tee_intensifies_the_stresses_at_its_node() {
    // A tee at node 20: run 10–20–30 along X, branch 20–40 up; anchored at 10, 30 and 40; heated.
    let mut d = table("tee", &[(10, 20, 3.0, 0.0, 0.0), (20, 30, 3.0, 0.0, 0.0), (20, 40, 0.0, 3.0, 0.0)]);
    comp(&mut d, 20, Comp::Tee { tee: TeeKind::Welding, pad: 0.0 });
    let mut c = d.m.conds[0].clone();
    c.t[0] = K0 + 200.0;
    d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
    for n in [10, 30, 40] {
        sup(&mut d, n, SupKind::Anchor);
    }
    let r = analyze(&d.m).unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let p = &d.m.pipes[0];
    let sif = crate::code::tee(TeeKind::Welding, p.wall, p.od, 0.0);
    let pt = r.geom.node_point[&20];
    let at_tee: Vec<_> = case(&r, "EXP1").stresses.iter().filter(|s| s.point == pt).collect();
    assert_eq!(at_tee.len(), 3);
    assert!(at_tee.iter().all(|s| (s.ii - sif.ii).abs() < 1e-12 && (s.io - sif.io).abs() < 1e-12));
    // Away from the tee the pipe is plain (i = 1).
    assert!(case(&r, "EXP1").stresses.iter().filter(|s| s.point != pt).all(|s| s.ii == 1.0));
}

#[test]
fn a_nozzle_reports_the_anchor_load_in_its_own_axes() {
    // A pump nozzle at node 10 pointing up (+Y) into the pipe; 2 m up then 3 m along X to an anchor;
    // a 5 kN force along X at the corner. The nozzle carries the pipe's push: axial = the vertical
    // load on it, shear = the horizontal one.
    let mut d = table("nozzle", &[(10, 20, 0.0, 2.0, 0.0), (20, 30, 3.0, 0.0, 0.0)]);
    no_g(&mut d);
    sup(&mut d, 10, SupKind::Anchor);
    sup(&mut d, 30, SupKind::Anchor);
    d.apply(Op::Add { item: Item::Nozzle(Nozzle { node: 10, name: "P-101 discharge".into(), axis: [0.0, 1.0, 0.0], allow: [1e9, 1e9, 1e9, 1e9], ..Default::default() }), after: None }).unwrap();
    load(&mut d, 20, [5e3, 0.0, 0.0]);
    let r = analyze(&d.m).unwrap();
    let c = case(&r, "OPE1");
    let z = &c.nozzles[0];
    let rx = react(&r, "OPE1", 10);
    assert!(close(z.local[0], -rx[1], 1e-12), "{:?} {rx:?}", z.local);
    assert!(close(z.shear, rx[0].hypot(rx[2]), 1e-12) && close(z.bending, rx[3].hypot(rx[5]), 1e-12));
    assert_eq!(z.pass, Some(true));
}

#[test]
fn liberal_allowable_and_cycles() {
    // The Z-bend: the liberal allowable adds Sh − SL; 100 000 cycles lower f to 6 × 1e5^−0.2 = 0.6.
    let mut m = crate::examples::get("zbend").unwrap();
    let r = analyze(&m).unwrap();
    let basic = case(&r, "EXP1").stresses[0].allow;
    m.settings.liberal = true;
    let r = analyze(&m).unwrap();
    assert!(case(&r, "EXP1").stresses[0].allow > basic);
    m.settings.liberal = false;
    m.settings.cycles = 1e5;
    let r = analyze(&m).unwrap();
    assert!(close(case(&r, "EXP1").stresses[0].allow, basic * 6.0 * 1e5f64.powf(-0.2), 1e-12));
}

#[test]
fn the_hydrotest_fills_the_line_with_water() {
    // 6 m of 6" Sch 40 between anchors, empty in service; the hydrotest case carries the pipe and
    // its water: ρ_steel A_metal g + 1000 A_bore g per metre.
    let mut d = table("hydro", &[(10, 20, 3.0, 0.0, 0.0), (20, 30, 3.0, 0.0, 0.0)]);
    sup(&mut d, 10, SupKind::Anchor);
    sup(&mut d, 30, SupKind::Anchor);
    let mut s = d.m.settings.clone();
    s.hydro = true;
    d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
    assert!(d.m.cases.iter().any(|c| c.name == "HYD"));
    let p = &d.m.pipes[0];
    let pi = std::f64::consts::PI;
    let id = p.od - 2.0 * p.wall;
    let w = 9.806_65 * (d.m.materials[0].rho * pi / 4.0 * (p.od * p.od - id * id) + 1000.0 * pi / 4.0 * id * id);
    let r = analyze(&d.m).unwrap();
    let fy = react(&r, "HYD", 10)[1] + react(&r, "HYD", 30)[1];
    assert!(close(fy, 6.0 * w, 1e-12), "{fy} {}", 6.0 * w);
    // In service it is empty: less load.
    assert!(react(&r, "SUS", 10)[1] < react(&r, "HYD", 10)[1]);
}

#[test]
fn an_occasional_case_is_checked_against_a_third_more() {
    // A user's own OCC case (weight and pressure, no occasional loads yet in 0.1): its allowable is
    // 1.33 Sh and its stress the sustained stress.
    let mut m = crate::examples::get("lbend").unwrap();
    let mut cs = m.cases.clone();
    cs.push(crate::model::LoadCase { id: 0, name: "OCC".into(), kind: crate::model::CaseKind::Occ, recipe: crate::model::Recipe::Sum { w: true, water: false, t: 0, d: 0, p: 1, wind: false, seis: false, relief: false } });
    let mut d = Doc::new(m.clone());
    d.apply(Op::SetCases { cases: cs }).unwrap();
    m = d.m;
    let r = analyze(&m).unwrap();
    let (sus, occ) = (case(&r, "SUS"), case(&r, "OCC"));
    for (a, b) in sus.stresses.iter().zip(&occ.stresses) {
        assert!(close(b.stress, a.stress, 1e-12) && close(b.allow, 1.33 * a.allow, 1e-12));
    }
}

#[test]
fn a_range_takes_sh_at_the_hotter_case() {
    // Found by review: EXP1–2 took Sh at case 2 even when case 1 was hotter. T1 = 400 °C, T2 = 100 °C.
    let mut m = crate::examples::get("zbend").unwrap();
    m.conds[0].t = [K0 + 400.0, K0 + 100.0, K0 + 21.0];
    crate::cases::rebuild(&mut m);
    let r = analyze(&m).unwrap();
    let mat = &m.materials[0];
    let f = crate::code::range_factor(m.settings.cycles);
    let want = crate::code::allowable_range(f, crate::library::interp(&mat.allow, m.settings.ambient), crate::library::interp(&mat.allow, K0 + 400.0), None);
    assert!(close(case(&r, "EXP1–2").stresses[0].allow, want, 1e-12) && close(case(&r, "EXP1").stresses[0].allow, want, 1e-12));
}

#[test]
fn a_line_stop_at_a_tee_holds_the_run_only() {
    // Found by review: the axis at a tee averaged in the branch. Run 10–20–30 along X drawn towards
    // the tee from both ends, branch 20–40 along Z; a line stop at 20 must not take a Z load.
    let mut d = table("tee stop", &[(10, 20, 3.0, 0.0, 0.0), (30, 20, -3.0, 0.0, 0.0), (20, 40, 0.0, 0.0, 3.0)]);
    no_g(&mut d);
    sup(&mut d, 10, SupKind::Anchor);
    sup(&mut d, 40, SupKind::Anchor);
    sup(&mut d, 20, SupKind::Linestop);
    load(&mut d, 20, [0.0, 0.0, 1e3]);
    let r = analyze(&d.m).unwrap();
    let ls = case(&r, "OPE1").reactions.iter().find(|x| x.node == 20).unwrap().f;
    assert!(ls[2].abs() < 1e-9 && ls[1].abs() < 1e-9, "{ls:?}");
}

#[test]
fn a_tee_marked_on_its_branch_uses_the_run_size() {
    // Found by review: a 2" branch drawn into a 6" run took h from the 2" pipe.
    let mut d = table("tee branch", &[(10, 20, 3.0, 0.0, 0.0), (20, 30, 3.0, 0.0, 0.0)]);
    let o = d.apply(Op::Add { item: Item::Pipe(Pipe::std(0, "2", "40")), after: None }).unwrap();
    let e = crate::model::Element { from: 40, to: 20, d: [0.0, 0.0, -2.0], pipe: o[0], material: d.m.materials[0].id, cond: d.m.conds[0].id, comp: Comp::Tee { tee: TeeKind::Welding, pad: 0.0 }, ..Default::default() };
    d.apply(Op::Add { item: Item::Element(e), after: None }).unwrap();
    let g = crate::geom::Geom::build(&d.m).unwrap();
    assert_eq!(g.tees[0].pipe, d.m.pipes[0].id);
    assert!((crate::mat::dot(g.tees[0].normal, [0.0, 1.0, 0.0]).abs() - 1.0).abs() < 1e-12, "{:?}", g.tees[0].normal);
}

#[test]
fn a_reducer_wall_is_checked_at_both_ends() {
    // 6" Sch 40 to 4" Sch 40 at 50 bar, the small end with its own 3 mm corrosion allowance: one
    // check per end, each with its own pipe.
    let mut d = table("reducer wall", &[(10, 20, 1.0, 0.0, 0.0)]);
    let mut small = Pipe::std(0, "4", "40");
    small.corrosion = 0.003;
    let o = d.apply(Op::Add { item: Item::Pipe(small.clone()), after: None }).unwrap();
    comp(&mut d, 20, Comp::Reducer { to: o[0] });
    let mut c = d.m.conds[0].clone();
    c.p[0] = 50e5;
    d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
    sup(&mut d, 10, SupKind::Anchor);
    let r = analyze(&d.m).unwrap();
    let ends: Vec<_> = r.walls.iter().filter(|w| w.case == 1).collect();
    assert_eq!(ends.len(), 2);
    let s = ends.iter().find(|w| w.at == "small end").unwrap();
    let big = ends.iter().find(|w| w.at == "large end").unwrap();
    assert!((s.od - small.od).abs() < 1e-12 && (big.od - d.m.pipes[0].od).abs() < 1e-12);
    // The small end: tm = t + 3 mm.
    assert!((s.t_min - s.t_design - 0.003).abs() < 1e-12);
}

#[test]
fn two_rigid_supports_in_one_direction_share_the_load() {
    // A rest and a vertical restraint both at node 20 of a cantilever: 10 kN down is shared 5 + 5,
    // with a warning; the total is the one support's.
    let mut d = table("shared", &[(10, 20, 3.0, 0.0, 0.0)]);
    no_g(&mut d);
    sup(&mut d, 10, SupKind::Anchor);
    sup(&mut d, 20, SupKind::Rest);
    load(&mut d, 20, [0.0, -10e3, 0.0]);
    let one = react(&analyze(&d.m).unwrap(), "OPE1", 20)[1];
    sup(&mut d, 20, SupKind::Restraint { dir: [0.0, 1.0, 0.0] });
    let r = analyze(&d.m).unwrap();
    let each: Vec<f64> = case(&r, "OPE1").reactions.iter().filter(|x| x.node == 20).map(|x| x.f[1]).collect();
    assert_eq!(each.len(), 2);
    assert!(close(each[0], one / 2.0, 1e-12) && close(each[1], one / 2.0, 1e-12), "{each:?} {one}");
    assert!(r.warnings.iter().any(|w| w.contains("node 20")));
    assert!(case(&r, "OPE1").imbalance < 1e-9);
}

#[test]
fn the_branch_section_modulus_can_be_turned_off() {
    // The same out-of-plane moment on a 2" branch: the stress scales by Z / Ze between the two.
    let build = |ze: bool| {
        let mut d = table("ze", &[(10, 20, 3.0, 0.0, 0.0), (20, 30, 3.0, 0.0, 0.0)]);
        let mut s = d.m.settings.clone();
        s.g = 0.0;
        s.branch_ze = ze;
        d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
        let o = d.apply(Op::Add { item: Item::Pipe(Pipe::std(0, "2", "40")), after: None }).unwrap();
        let e = crate::model::Element { from: 20, to: 40, d: [0.0, 0.0, 2.0], pipe: o[0], material: d.m.materials[0].id, cond: d.m.conds[0].id, ..Default::default() };
        d.apply(Op::Add { item: Item::Element(e), after: None }).unwrap();
        comp(&mut d, 20, Comp::Tee { tee: TeeKind::Welding, pad: 0.0 });
        sup(&mut d, 10, SupKind::Anchor);
        sup(&mut d, 30, SupKind::Anchor);
        load(&mut d, 40, [0.0, 1e3, 0.0]);
        let r = analyze(&d.m).unwrap();
        let pt = r.geom.node_point[&20];
        let br = d.m.elements.iter().find(|e| e.to == 40).unwrap().id;
        let st = case(&r, "SUS").stresses.iter().find(|x| x.point == pt && r.geom.segs[x.seg].element == br).unwrap().stress;
        (st, d.m.pipes.clone())
    };
    let ((on, pipes), (off, _)) = (build(true), build(false));
    let (run, b) = (&pipes[0], &pipes[1]);
    let ii = crate::code::tee(TeeKind::Welding, run.wall, run.od, 0.0).ii;
    let ratio = crate::code::z(b.od, b.wall) / crate::code::branch_z(b.od, b.wall, run.wall, ii);
    assert!(close(on / off, ratio, 1e-9), "{} {ratio}", on / off);
}

#[test]
fn pressure_elongation_is_off_unless_chosen_and_stays_out_of_sustained() {
    let build = |on: bool| {
        let mut d = table("pe", &[(10, 20, 3.0, 0.0, 0.0), (20, 30, 3.0, 0.0, 0.0)]);
        let mut s = d.m.settings.clone();
        s.pressure_elongation = on;
        d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
        let mut c = d.m.conds[0].clone();
        c.p[0] = 50e5;
        d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
        sup(&mut d, 10, SupKind::Anchor);
        sup(&mut d, 30, SupKind::Anchor);
        analyze(&d.m).unwrap()
    };
    let (off, on) = (build(false), build(true));
    assert!(react(&off, "OPE1", 30)[0].abs() < 1e-6);
    assert!(react(&on, "OPE1", 30)[0] < -1e3);
    assert!(close(react(&on, "SUS", 30)[1], react(&off, "SUS", 30)[1], 1e-12));
    assert!(react(&on, "SUS", 30)[0].abs() < 1e-6);
}

#[test]
fn a_thick_wall_fails_with_a_warning() {
    // 2" Sch 40 at 1000 bar with 20 MPa allowed: t ≥ D/6.
    let mut d = table("thick", &[(10, 20, 1.0, 0.0, 0.0)]);
    let mut p = d.m.pipes[0].clone();
    let (od, wall) = crate::library::pipe("2", "40").unwrap();
    p.od = od;
    p.wall = wall;
    d.apply(Op::Set { item: Item::Pipe(p) }).unwrap();
    let mut c = d.m.conds[0].clone();
    c.p[0] = 1000e5;
    d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
    sup(&mut d, 10, SupKind::Anchor);
    let r = analyze(&d.m).unwrap();
    assert!(r.walls.iter().any(|w| w.thick && !w.pass));
    assert!(r.warnings.iter().any(|w| w.contains("D/6")));
}

#[test]
fn a_bend_wall_is_checked_at_its_intrados() {
    // An L with a long-radius bend at 20 at 50 bar: the bend has its own check with I > 1, and
    // needs more wall than the straight pipe.
    let mut d = table("bend wall", &[(10, 20, 3.0, 0.0, 0.0), (20, 30, 0.0, 3.0, 0.0)]);
    comp(&mut d, 20, Comp::Bend { bend: crate::model::BendKind::Long, r: 0.0 });
    let mut c = d.m.conds[0].clone();
    c.p[0] = 50e5;
    d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
    sup(&mut d, 10, SupKind::Anchor);
    let r = analyze(&d.m).unwrap();
    let e = d.m.elements[0].id;
    let pipe = r.walls.iter().find(|w| w.element == e && w.at == "pipe").unwrap();
    let bend = r.walls.iter().find(|w| w.element == e && w.at == "bend").unwrap();
    let rb = crate::geom::bend_radius(&d.m, crate::model::BendKind::Long, 0.0, d.m.pipes[0].id);
    assert!(close(bend.i, crate::code::bend_factor(rb, d.m.pipes[0].od).0, 1e-12));
    assert!(bend.i > 1.0 && bend.t_design > pipe.t_design);
}

fn sup_with(d: &mut Doc, node: u32, kind: SupKind, one_way: bool, gap: f64) {
    d.apply(Op::Add { item: Item::Support(Support { node, kind, one_way, gap, ..Default::default() }), after: None }).unwrap();
}

#[test]
fn a_hot_riser_lifts_the_pipe_off_its_rest() {
    // A 5 m riser anchored at the bottom (10), then 6 m along X to an anchor (30), with a one-way
    // rest in the middle (25). Cold, the rest carries weight; at 300 °C the riser grows about 18 mm
    // and lifts the pipe off the rest, which then carries nothing.
    let mut d = table("riser", &[(10, 20, 0.0, 5.0, 0.0), (20, 25, 3.0, 0.0, 0.0), (25, 30, 3.0, 0.0, 0.0)]);
    let mut c = d.m.conds[0].clone();
    c.t[0] = K0 + 300.0;
    d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
    sup(&mut d, 10, SupKind::Anchor);
    sup(&mut d, 30, SupKind::Anchor);
    sup_with(&mut d, 25, SupKind::Rest, true, 0.0);
    let r = analyze(&d.m).unwrap();
    let state = |c: &str| case(&r, c).contacts.iter().find(|x| x.node == 25).unwrap().clone();
    assert_eq!(state("SUS").state, "touching");
    assert!(react(&r, "SUS", 25)[1] > 0.0);
    let ope = state("OPE1");
    assert_eq!(ope.state, "lifted off");
    assert!(ope.movement > 0.005 && react(&r, "OPE1", 25)[1] == 0.0, "{ope:?}");
    assert!(case(&r, "OPE1").iterations >= 2);
    for c in &r.cases {
        assert!(c.imbalance < 1e-9, "{} {:e}", c.name, c.imbalance);
    }
    // The range is still OPE − SUS.
    let (o, s, e) = (react(&r, "OPE1", 25)[1], react(&r, "SUS", 25)[1], react(&r, "EXP1", 25)[1]);
    assert!(close(e, o - s, 1e-12));
}

#[test]
fn a_guide_gap_stays_open_under_a_small_push() {
    // 3 m cantilever, a Z restraint with a 50 mm gap at the tip, 1 kN along Z: the tip moves about
    // 3.8 mm, well inside the gap, so the restraint carries nothing.
    let mut d = table("gap open", &[(10, 20, 3.0, 0.0, 0.0)]);
    no_g(&mut d);
    sup(&mut d, 10, SupKind::Anchor);
    sup_with(&mut d, 20, SupKind::Restraint { dir: [0.0, 0.0, 1.0] }, false, 0.05);
    load(&mut d, 20, [0.0, 0.0, 1e3]);
    let (e, i, a, g) = props(&d);
    let c = 27.0 / (3.0 * e * i) + 3.0 / (KAPPA * g * a);
    let r = analyze(&d.m).unwrap();
    assert!(close(at(&r, "OPE1", 20)[2], 1e3 * c, 1e-10));
    assert_eq!(case(&r, "OPE1").contacts[0].state, "open");
    assert_eq!(react(&r, "OPE1", 20)[2], 0.0);
}

#[test]
fn a_rest_from_an_older_file_still_holds_both_ways() {
    // Without one_way (files from before 0.1.7) a rest pulls the pipe down too.
    let mut d = table("two-way", &[(10, 20, 3.0, 0.0, 0.0)]);
    no_g(&mut d);
    sup(&mut d, 10, SupKind::Anchor);
    sup(&mut d, 20, SupKind::Rest);
    load(&mut d, 20, [0.0, 10e3, 0.0]);
    let r = analyze(&d.m).unwrap();
    assert!(close(react(&r, "OPE1", 20)[1], -10e3, 1e-9));
    assert!(case(&r, "OPE1").contacts.is_empty());
}

#[test]
fn gapped_guides_alone_still_solve() {
    // An L on rests: 10–20 along X, 20–30 along Z. A line stop at 10 (along X) and one at 30 (along Z)
    // leave the L free to turn about node 20 in plan, which only the gapped guides at 10 and 30
    // resist: while their gaps are open they keep a light stiffness so the case can be solved.
    let mut d = table("guides", &[(10, 20, 6.0, 0.0, 0.0), (20, 30, 0.0, 0.0, 6.0)]);
    for n in [10, 20, 30] {
        sup_with(&mut d, n, SupKind::Rest, true, 0.0);
    }
    sup(&mut d, 10, SupKind::Linestop);
    sup(&mut d, 30, SupKind::Linestop);
    sup_with(&mut d, 10, SupKind::Guide, false, 0.003);
    sup_with(&mut d, 30, SupKind::Guide, false, 0.003);
    let mut c = d.m.conds[0].clone();
    c.t[0] = K0 + 150.0;
    d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
    let r = analyze(&d.m).unwrap();
    assert!(r.warnings.iter().any(|w| w.contains("light stiffness")), "{:?}", r.warnings);
    for c in &r.cases {
        assert!(c.imbalance < 1e-9, "{} {:e}", c.name, c.imbalance);
    }
}

fn sup_mu(d: &mut Doc, node: u32, kind: SupKind, one_way: bool, mu: f64) {
    d.apply(Op::Add { item: Item::Support(Support { node, kind, one_way, mu, ..Default::default() }), after: None }).unwrap();
}

#[test]
fn friction_opposes_the_movement_in_the_plane_of_a_rest() {
    // 6 m along X, anchored at 10, a rest with μ = 0.3 at 20 carrying 10 kN, heated (the pipe grows
    // along X) and pushed 1 kN along Z: it slides along both, and the friction is μ N straight
    // against the movement across the rest.
    let mut d = table("2D friction", &[(10, 20, 6.0, 0.0, 0.0)]);
    no_g(&mut d);
    let mut c = d.m.conds[0].clone();
    c.t[0] = K0 + 200.0;
    d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
    sup(&mut d, 10, SupKind::Anchor);
    sup_mu(&mut d, 20, SupKind::Rest, true, 0.3);
    load(&mut d, 20, [0.0, -10e3, 1e3]);
    let r = analyze(&d.m).unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let fr = &case(&r, "OPE1").friction[0];
    assert_eq!(fr.state, "sliding");
    assert!(close(fr.normal, 10e3, 1e-9) && close(crate::mat::norm(fr.force), 3e3, 1e-6), "{fr:?}");
    let u = at(&r, "OPE1", 20);
    let mv = crate::mat::unit([u[0], 0.0, u[2]]);
    assert!(crate::mat::dot(crate::mat::unit(fr.force), mv) < -1.0 + 1e-6, "{fr:?} {u:?}");
    assert!(case(&r, "OPE1").imbalance < 1e-9);
    // Cold, with the 1 kN within μ N, the rest holds it: sticking, with the load as friction.
    let sus = &case(&r, "SUS").friction[0];
    assert_eq!(sus.state, "sticking");
    assert!(close(sus.force[2], -1e3, 1e-9), "{sus:?}");
}

#[test]
fn a_lifted_rest_has_no_friction() {
    // Pushed up off a one-way rest with μ = 0.3 and pulled along X: no load on the rest, so no friction.
    let mut d = table("lifted", &[(10, 20, 3.0, 0.0, 0.0)]);
    no_g(&mut d);
    sup(&mut d, 10, SupKind::Anchor);
    sup_mu(&mut d, 20, SupKind::Rest, true, 0.3);
    load(&mut d, 20, [1e3, 1e3, 0.0]);
    let r = analyze(&d.m).unwrap();
    let fr = &case(&r, "OPE1").friction[0];
    assert!(crate::mat::norm(fr.force) == 0.0 && fr.normal == 0.0, "{fr:?}");
    assert_eq!(react(&r, "OPE1", 20), [0.0; 6]);
}

#[test]
fn a_guide_pressed_sideways_resists_the_pipe_growing_along_it() {
    // A guide with μ = 0.4 at 20 on a pipe along X, pressed by 5 kN along Z, the rest at 20 taking
    // the vertical: the pipe grows along X and the guide's friction is 0.4 × 5 kN against it.
    let mut d = table("guide friction", &[(10, 20, 6.0, 0.0, 0.0)]);
    no_g(&mut d);
    let mut c = d.m.conds[0].clone();
    c.t[0] = K0 + 200.0;
    d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
    sup(&mut d, 10, SupKind::Anchor);
    sup(&mut d, 20, SupKind::Rest);
    sup_mu(&mut d, 20, SupKind::Guide, false, 0.4);
    load(&mut d, 20, [0.0, 0.0, 5e3]);
    let r = analyze(&d.m).unwrap();
    let fr = &case(&r, "OPE1").friction[0];
    assert_eq!(fr.state, "sliding");
    assert!(close(fr.normal, 5e3, 1e-9) && close(fr.force[0], -2e3, 1e-9) && fr.force[1].abs() < 1e-9, "{fr:?}");
    for c in &r.cases {
        assert!(c.imbalance < 1e-9, "{} {:e}", c.name, c.imbalance);
    }
}

#[test]
fn wind_acts_across_the_pipe_only_and_on_its_insulation() {
    // A 3 m cantilever along X: wind along X (down its axis) puts no load on it; wind along Z puts
    // q Cd D per metre, with D over 50 mm of insulation.
    let build = |dir: [f64; 3]| {
        let mut d = table("wind", &[(10, 20, 3.0, 0.0, 0.0)]);
        let mut p = d.m.pipes[0].clone();
        p.ins_t = 0.05;
        d.apply(Op::Set { item: Item::Pipe(p) }).unwrap();
        let mut s = d.m.settings.clone();
        s.g = 0.0;
        s.wind_q = 800.0;
        s.wind_cd = 0.6;
        s.wind_dir = dir;
        d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
        sup(&mut d, 10, SupKind::Anchor);
        (analyze(&d.m).unwrap(), d.m.pipes[0].od)
    };
    let (r, _) = build([1.0, 0.0, 0.0]);
    assert!(react(&r, "OCC WIN", 10).iter().all(|v| v.abs() < 1e-9));
    let (r, od) = build([0.0, 0.0, -1.0]);
    assert!(close(react(&r, "OCC WIN", 10)[2], 800.0 * 0.6 * (od + 0.1) * 3.0, 1e-12));
    assert!(case(&r, "OCC WIN").stresses.iter().all(|s| (s.allow / case(&r, "SUS").stresses[0].allow - 1.33).abs() < 1e-12));
}

#[test]
fn occasional_cases_come_with_wind_or_seismic_only() {
    let mut m = crate::examples::get("lbend").unwrap();
    assert!(!m.cases.iter().any(|c| c.kind == crate::model::CaseKind::Occ));
    m.settings.wind_q = 500.0;
    m.settings.seis = [0.2, 0.0, 0.2];
    crate::cases::rebuild(&mut m);
    let names: Vec<&str> = m.cases.iter().map(|c| c.name.as_str()).collect();
    for n in ["OPE1+WIN", "OCC WIN", "OPE1+SEIS", "OCC SEIS"] {
        assert!(names.contains(&n), "{names:?}");
    }
    let r = analyze(&m).unwrap();
    for c in &r.cases {
        assert!(c.imbalance < 1e-9, "{} {:e}", c.name, c.imbalance);
    }
    // The occasional stress is never below the sustained one at the same point.
    for (o, s) in case(&r, "OCC SEIS").stresses.iter().zip(&case(&r, "SUS").stresses) {
        assert!(o.stress >= s.stress - 1e-6);
    }
}

#[test]
fn files_without_occasional_loads_still_open() {
    // A file written before 0.1.9: no wind or seismic settings, no wind/seis in the case recipes.
    let m = crate::examples::get("lbend").unwrap();
    let mut j: serde_json::Value = serde_json::from_str(&crate::model::to_json(&m)).unwrap();
    for k in ["wind_q", "wind_cd", "wind_dir", "seis"] {
        j["settings"].as_object_mut().unwrap().remove(k);
    }
    for c in j["cases"].as_array_mut().unwrap() {
        if let Some(r) = c["recipe"].as_object_mut() {
            r.remove("wind");
            r.remove("seis");
        }
    }
    let (back, _) = crate::model::from_json(&j.to_string()).unwrap();
    assert_eq!(back.settings.wind_q, 0.0);
    assert_eq!(back.cases, m.cases);
}

#[test]
fn choosing_a_spring_from_the_table() {
    use crate::model::SpringSize;
    let t = vec![
        SpringSize { name: "stiff".into(), k: 1e6, min: 5e3, max: 15e3 },
        SpringSize { name: "soft".into(), k: 1e5, min: 5e3, max: 15e3 },
    ];
    // 10 kN going up 5 mm: the stiff spring's variation is 50 %, the soft one's 5 %: C = 10.5 kN.
    let (i, cold) = crate::solve::choose_spring(&t, 10e3, 0.005, 0.25).unwrap();
    assert_eq!(t[i].name, "soft");
    assert!(close(cold, 10.5e3, 1e-12));
    // Going down 5 mm the cold load is less than the hot: 9.5 kN.
    assert!(close(crate::solve::choose_spring(&t, 10e3, -0.005, 0.25).unwrap().1, 9.5e3, 1e-12));
    // Out of every range, or no weight: none.
    assert!(crate::solve::choose_spring(&t, 20e3, 0.0, 0.25).is_none());
    assert!(crate::solve::choose_spring(&t, 0.0, 0.0, 0.25).is_none());
}

#[test]
fn a_hanger_that_no_spring_fits_becomes_a_constant_support() {
    // 3 m cantilever with a hanger at its tip carrying 2 MN: beyond the table, so a constant-force
    // support of the hot load, with a warning; in operation it carries exactly that load.
    let mut d = table("big hanger", &[(10, 20, 3.0, 0.0, 0.0)]);
    no_g(&mut d);
    sup(&mut d, 10, SupKind::Anchor);
    sup(&mut d, 20, SupKind::Hanger { k: 0.0, cold: 0.0 });
    load(&mut d, 20, [0.0, -2e6, 0.0]);
    let r = analyze(&d.m).unwrap();
    assert_eq!(r.hangers[0].size, "constant");
    assert!(r.warnings.iter().any(|w| w.contains("No spring in the table fits")), "{:?}", r.warnings);
    assert!(close(react(&r, "OPE1", 20)[1], 2e6, 1e-9));
}

#[test]
fn files_without_a_spring_table_get_the_generic_one() {
    let m = crate::examples::get("lbend").unwrap();
    let mut j: serde_json::Value = serde_json::from_str(&crate::model::to_json(&m)).unwrap();
    j["settings"].as_object_mut().unwrap().remove("springs");
    j["settings"].as_object_mut().unwrap().remove("variability");
    let (back, _) = crate::model::from_json(&j.to_string()).unwrap();
    assert_eq!(back.settings.springs.len(), 60);
    assert_eq!(back.settings.variability, 0.25);
}

#[test]
fn two_one_way_rests_at_a_node_share_the_load() {
    // Found by the 0.2.0 review: two touching one-way rests on one node each reported the whole
    // load (the balance was out by half). They share it, with a warning, as linear supports do.
    let mut d = table("two rests", &[(10, 20, 3.0, 0.0, 0.0)]);
    no_g(&mut d);
    sup(&mut d, 10, SupKind::Anchor);
    sup_with(&mut d, 20, SupKind::Rest, true, 0.0);
    sup_with(&mut d, 20, SupKind::Rest, true, 0.0);
    load(&mut d, 20, [0.0, -10e3, 0.0]);
    let r = analyze(&d.m).unwrap();
    let each: Vec<f64> = case(&r, "OPE1").reactions.iter().filter(|x| x.node == 20).map(|x| x.f[1]).collect();
    assert_eq!(each.len(), 2);
    assert!(each.iter().all(|f| close(*f, 5e3, 1e-9)), "{each:?}");
    assert!(case(&r, "OPE1").imbalance < 1e-9);
    assert!(r.warnings.iter().any(|w| w.contains("node 20")), "{:?}", r.warnings);
}

#[test]
fn a_bend_too_tight_for_its_pipe_is_refused() {
    // Found by the 0.2.0 review: a custom radius of D/2 or less gave a negative bend factor and a
    // passing wall check. Such a bend is refused.
    let mut d = table("tight", &[(10, 20, 3.0, 0.0, 0.0), (20, 30, 0.0, 3.0, 0.0)]);
    let mut e = d.m.elements[0].clone();
    e.comp = Comp::Bend { bend: crate::model::BendKind::Custom, r: 0.06 };
    assert!(d.apply(Op::Set { item: Item::Element(e) }).is_err());
    // And the factor itself never gives a thinner wall than straight pipe needs.
    let (i, _) = crate::code::bend_factor(0.06, 0.168);
    assert!(!(i.is_finite() && i > 0.0) || i >= 1.0);
}

#[test]
fn a_hanger_on_a_moving_attachment_carries_its_hot_load() {
    // Found by the 0.2.0 review: a hanger whose attachment moves 10 mm up in case 1 was sized on
    // the pipe's absolute travel, so in operation it did not carry H.
    let mut d = table("moving hanger", &[(10, 20, 6.0, 0.0, 0.0)]);
    no_g(&mut d);
    sup(&mut d, 10, SupKind::Anchor);
    d.apply(Op::Add { item: Item::Support(Support { node: 20, kind: SupKind::Hanger { k: 0.0, cold: 0.0 }, disp: [[0.0, 0.01, 0.0, 0.0, 0.0, 0.0], [0.0; 6], [0.0; 6]], ..Default::default() }), after: None }).unwrap();
    load(&mut d, 20, [0.0, -5e3, 0.0]);
    let r = analyze(&d.m).unwrap();
    let h = &r.hangers[0];
    assert!(h.k > 0.0, "{h:?}");
    assert!(close(react(&r, "OPE1", 20)[1], h.hot, 1e-9), "{} {}", react(&r, "OPE1", 20)[1], h.hot);
    assert!(close(h.cold, h.hot + h.k * h.travel, 1e-12));
}

#[test]
fn a_free_model_says_where_it_is_free() {
    // A pipe along X anchored at 10 only along Y and Z by restraints: nothing holds it along X, and
    // it can turn. The message names a node and a direction to go to.
    let mut d = table("free", &[(10, 20, 3.0, 0.0, 0.0), (20, 30, 3.0, 0.0, 0.0)]);
    sup(&mut d, 10, SupKind::Restraint { dir: [0.0, 1.0, 0.0] });
    sup(&mut d, 30, SupKind::Restraint { dir: [0.0, 1.0, 0.0] });
    sup(&mut d, 10, SupKind::Restraint { dir: [0.0, 0.0, 1.0] });
    sup(&mut d, 30, SupKind::Restraint { dir: [0.0, 0.0, 1.0] });
    let e = analyze(&d.m).unwrap_err();
    assert!(e.contains("without resistance at node ") && (e.contains("along X") || e.contains("turning about X")), "{e}");
    // With a line stop along X, it is the twist about the pipe that is free.
    sup(&mut d, 20, SupKind::Linestop);
    let e = analyze(&d.m).unwrap_err();
    assert!(e.contains("turning about X"), "{e}");
}

#[test]
fn friction_on_a_long_rack_settles_to_coulomb_s_law() {
    // 30 bays of rack (over 64 supports with friction, so the fast flexibility estimate is used):
    // every sliding support pushes exactly μ N against its movement, every sticking one stays within
    // μ N, and every case balances.
    let m = crate::examples::big(30);
    let r = analyze(&m).unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    for c in r.cases.iter().filter(|c| !c.friction.is_empty()) {
        assert!(c.imbalance < 1e-9, "{} {:e}", c.name, c.imbalance);
        for f in &c.friction {
            let size = crate::mat::norm(f.force);
            match f.state {
                "sliding" => {
                    assert!(close(size, f.mu * f.normal, 1e-6), "{} {f:?}", c.name);
                    let pt = r.geom.node_point[&f.node];
                    let u = c.disp[pt];
                    let across = [u[0], 0.0, u[2]];
                    if crate::mat::norm(across) > 1e-9 {
                        assert!(crate::mat::dot(crate::mat::unit(f.force), crate::mat::unit(across)) < -1.0 + 1e-6, "{} {f:?} {u:?}", c.name);
                    }
                }
                "sticking" => assert!(size <= f.mu * f.normal * (1.0 + 1e-9) + 1e-9, "{} {f:?}", c.name),
                _ => {}
            }
        }
    }
}

#[test]
fn a_relief_valve_pushes_back_against_its_discharge() {
    // 2 m along X from an anchor, discharging along +Z (given) at the end: the thrust pushes the pipe
    // along −Z, so the anchor carries +Z and a moment about Y of F × 2 m. Only the cases with relief
    // carry it; it is an occasional range OCC RV.
    let mut d = table("rv", &[(10, 20, 2.0, 0.0, 0.0)]);
    no_g(&mut d);
    sup(&mut d, 10, SupKind::Anchor);
    let rv = crate::relief::Relief { node: 20, flow: 10.0, dir: [0.0, 0.0, 3.0], ..Default::default() };
    d.apply(Op::SetReliefs { list: vec![rv] }).unwrap();
    let names: Vec<&str> = d.m.cases.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"OPE1+RV") && names.contains(&"OCC RV"), "{names:?}");
    let r = analyze(&d.m).unwrap();
    let f = r.reliefs[0].thrust;
    assert!(f > 0.0 && r.reliefs[0].choked);
    assert!(close(r.reliefs[0].force[2], -f, 1e-12));
    let a = react(&r, "OPE1+RV", 10);
    assert!(close(a[2], f, 1e-9), "{a:?}");
    assert!(close(a[4].abs(), 2.0 * f, 1e-9), "{a:?}");
    assert!(react(&r, "OPE1", 10)[2].abs() < 1e-6 * f);
    assert!(close(react(&r, "OCC RV", 10)[2], f, 1e-9));
    assert!(r.cases.iter().all(|c| c.imbalance < 1e-9), "balance");
    // The default direction at the open end is along the pipe, out of it: the thrust is along −X.
    let rv = crate::relief::Relief { node: 20, flow: 10.0, ..Default::default() };
    d.apply(Op::SetReliefs { list: vec![rv] }).unwrap();
    let r = analyze(&d.m).unwrap();
    assert!(close(r.reliefs[0].force[0], -r.reliefs[0].thrust, 1e-12));
    assert!(close(react(&r, "OCC RV", 10)[0], r.reliefs[0].thrust, 1e-9));
}

#[test]
fn a_relief_valve_away_from_an_open_end_needs_a_direction_and_goes_with_its_node() {
    let mut d = table("rv", &[(10, 20, 2.0, 0.0, 0.0), (20, 30, 2.0, 0.0, 0.0)]);
    sup(&mut d, 10, SupKind::Anchor);
    let rv = crate::relief::Relief { node: 20, ..Default::default() };
    d.apply(Op::SetReliefs { list: vec![rv] }).unwrap();
    let e = analyze(&d.m).unwrap_err();
    assert!(e.contains("not an open end"), "{e}");
    // Damaged values are refused by the model check.
    let mut bad = d.m.clone();
    bad.reliefs[0].k = 0.5;
    assert!(bad.check().is_err());
    // A file with a damaged relief valve loads without it.
    let mut v: serde_json::Value = serde_json::from_str(&crate::model::to_json(&d.m)).unwrap();
    v["reliefs"][0]["flow"] = serde_json::json!(-1.0);
    let (m, warn) = crate::model::from_json(&v.to_string()).unwrap();
    assert!(m.reliefs.is_empty() && warn.iter().any(|w| w.contains("Relief")), "{warn:?}");
    // Removing the node's elements removes the relief valve, and its cases.
    let last = d.m.elements.iter().find(|e| e.to == 30).unwrap().id;
    d.apply(Op::Delete { ids: vec![last] }).unwrap();
    let mid = d.m.elements[0].id;
    d.apply(Op::Delete { ids: vec![mid] }).unwrap();
    assert!(d.m.reliefs.is_empty());
    assert!(!d.m.cases.iter().any(|c| c.name.contains("RV")));
}
