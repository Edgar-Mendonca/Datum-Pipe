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
    cs.push(crate::model::LoadCase { id: 0, name: "OCC".into(), kind: crate::model::CaseKind::Occ, recipe: crate::model::Recipe::Sum { w: true, water: false, t: 0, d: 0, p: 1 } });
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
