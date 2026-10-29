//! The analysis: the element stiffnesses assembled (sparse), the supports, every load case solved
//! with one factorisation, the reactions, the forces in the pipe, the code stresses, the support and
//! nozzle loads, and the equilibrium check of every case.
//!
//! Supports along the global axes that are rigid hold their degree of freedom exactly (with its
//! imposed displacement as a prescribed value); skewed or flexible ones are springs (a skewed rigid
//! restraint is a spring of 1e12 N/m). Load cases are linear sums of basic loads (weight, thermal,
//! imposed displacements), and ranges are differences of two cases, so everything is exact for
//! linear supports.
//!
//! One-way rests and supports with a gap are solved by iteration, case by case: each is either
//! touching (held at its gap, like a linear support) or open (no load), and the case is solved
//! again until no support changes state: a touching one must push the pipe, not pull it, and an
//! open one must not be passed by the pipe. Ranges stay differences of the two cases' results.

use crate::code::{self, Loads, Sif};
use crate::elements::{Elem, Line, Props, Section};
use crate::geom::{Geom, Shape};
use crate::library::interp;
use crate::mat::*;
use crate::model::{CaseKind, Comp, LoadCase, Model, Recipe, RigidKind, SupKind, RIGID};
use crate::sparse::{Stats, System};
use serde::Serialize;
use std::collections::HashMap;

/// The reference temperature of the mean expansion tables (21 °C).
pub const T_REF: f64 = 294.15;
pub const WATER: f64 = 1000.0;

/// A support's load on the pipe in one case (global axes: FX FY FZ MX MY MZ).
#[derive(Clone, Debug, Serialize)]
pub struct Reaction {
    pub support: u32,
    pub node: u32,
    pub f: [f64; 6],
}

/// The loads a nozzle carries from the pipe, in the nozzle's axes: axial (along the nozzle) and
/// the two directions across it, and the resultants with their checks.
#[derive(Clone, Debug, Serialize)]
pub struct NozzleLoad {
    pub nozzle: u32,
    pub node: u32,
    /// Fa, Fb, Fc, Ma (torsion), Mb, Mc.
    pub local: [f64; 6],
    pub shear: f64,
    pub bending: f64,
    /// Ratios to the allowable axial force, shear, torsion and bending (0 where none is given).
    pub ratios: [f64; 4],
    pub pass: Option<bool>,
}

/// The state of a one-way or gapped support in one case.
#[derive(Clone, Debug, Serialize)]
pub struct Contact {
    pub support: u32,
    pub node: u32,
    pub dir: V,
    /// "touching", "open", or "lifted off" (a one-way rest that is open).
    pub state: &'static str,
    /// The pipe's movement along the support's direction, relative to the support (m).
    pub movement: f64,
    pub gap: f64,
}

/// The code stress at one point of one case.
#[derive(Clone, Debug, Serialize)]
pub struct Stress {
    pub seg: usize,
    pub end: u8,
    pub point: usize,
    pub stress: f64,
    pub allow: f64,
    pub ratio: f64,
    pub pass: bool,
    pub ii: f64,
    pub io: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CaseResult {
    pub id: u32,
    pub name: String,
    pub kind: CaseKind,
    /// Displacements and rotations of every point (m, rad).
    pub disp: Vec<[f64; 6]>,
    /// The forces and moments in the pipe at each segment's start and end, in the segment's local
    /// axes (x along the pipe, y up, z across): Fx (axial), Fy, Fz, Mx (torsion), My, Mz.
    pub forces: Vec<[[f64; 6]; 2]>,
    pub reactions: Vec<Reaction>,
    pub nozzles: Vec<NozzleLoad>,
    /// The code stress at every segment end (sustained, expansion range, occasional cases).
    pub stresses: Vec<Stress>,
    /// Sum of the applied loads and of the reactions (forces, and moments about the origin).
    pub applied: [f64; 6],
    pub reacted: [f64; 6],
    pub imbalance: f64,
    /// The worst stress ratio and where.
    pub worst: Option<usize>,
    /// One-way and gapped supports: their state, and the solves it took to settle them.
    pub contacts: Vec<Contact>,
    pub iterations: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Results {
    pub geom: Geom,
    pub cases: Vec<CaseResult>,
    pub walls: Vec<code::Wall>,
    /// Stress intensification at each segment end (for the tables).
    pub sifs: Vec<[Sif; 2]>,
    pub stats: Stats,
    pub warnings: Vec<String>,
    pub version: &'static str,
}

/// Everything about one segment the solver needs.
struct Part {
    el: Elem,
    sec: (Section, Section),
    /// Weight per length with the contents and with water (N/m, down).
    w: f64,
    w_water: f64,
    /// Thermal strain in cases 1–3.
    strain: [f64; 3],
    e_cold: f64,
    mat: u32,
    /// Corrosion allowance at the start and the end (a reducer's ends may differ).
    corrosion: [f64; 2],
    /// Pressures in cases 1–3 and the hydrotest.
    p: [f64; 4],
    t: [f64; 3],
    sif: [Sif; 2],
    /// The plane normal for in-plane moments (bends and tees); none on straight pipe.
    plane: [Option<V>; 2],
    /// At a tee's branch: the effective section modulus on the nominal and the corroded walls.
    ze: [Option<(f64, f64)>; 2],
}

/// The section at fraction s of a segment (a reducer's changes along it).
fn sec_at(m: &Model, pa: u32, pb: u32, s: f64) -> Section {
    let (a, b) = (m.pipe(pa), m.pipe(pb));
    let (oa, ta) = a.map_or((0.1683, 0.0071), |p| (p.od, p.wall));
    let (ob, tb) = b.map_or((oa, ta), |p| (p.od, p.wall));
    Section::new(oa + (ob - oa) * s, ta + (tb - ta) * s)
}

/// Weight per length of a pipe with its contents (or full of water), insulation and cladding.
pub fn weight(m: &Model, pipe: u32, rho_steel: f64, water: bool) -> f64 {
    let Some(p) = m.pipe(pipe) else { return 0.0 };
    let pi4 = std::f64::consts::PI / 4.0;
    let id = p.od - 2.0 * p.wall;
    let metal = pi4 * (p.od * p.od - id * id) * rho_steel;
    let fluid = pi4 * id * id * if water { WATER } else { p.fluid_rho };
    let d1 = p.od + 2.0 * p.ins_t;
    let d2 = d1 + 2.0 * p.clad_t;
    let ins = pi4 * (d1 * d1 - p.od * p.od) * p.ins_rho + pi4 * (d2 * d2 - d1 * d1) * p.clad_rho;
    (metal + fluid + ins) * m.settings.g
}

/// The thermal strain of a material from the installed temperature to t: mean coefficients from
/// 21 °C, so ΔL/L = α(t)(t − 21 °C) − α(t₀)(t₀ − 21 °C).
pub fn strain(alpha: &[[f64; 2]], t: f64, t0: f64) -> f64 {
    interp(alpha, t) * (t - T_REF) - interp(alpha, t0) * (t0 - T_REF)
}

/// The axis of the pipe at a point, for guides and line stops: the run through it (two directions
/// that are opposite; at a tee the branch is left out), or the one segment at an end. At a corner
/// there is no single axis: the first segment's is used and the caller warns (found by review:
/// averaging every segment skewed a line stop at a tee).
fn pipe_dir(g: &Geom, pt: usize) -> (V, bool) {
    let dirs: Vec<V> = g.segs.iter().filter_map(|s| if s.a == pt { Some(g.at(s, 0.0).1) } else if s.b == pt { Some(scale(g.at(s, 1.0).1, -1.0)) } else { None }).collect();
    for (i, a) in dirs.iter().enumerate() {
        for b in &dirs[i + 1..] {
            if dot(*a, *b) < -0.999 {
                return (unit(*a), true);
            }
        }
    }
    match dirs.len() {
        0 => ([1.0, 0.0, 0.0], false),
        1 => (unit(dirs[0]), true),
        _ => (unit(dirs[0]), false),
    }
}

/// The restraint directions of a support: (direction, rotational, stiffness (0 = rigid)).
fn restraints(kind: &SupKind, k: f64, t: V) -> Vec<(V, bool, f64)> {
    let ax = |i: usize| { let mut v = [0.0; 3]; v[i] = 1.0; v };
    match kind {
        // A flexible anchor's stiffness is for its three movements; it holds the rotations rigidly.
        SupKind::Anchor => (0..6).map(|i| (ax(i % 3), i >= 3, if i >= 3 { 0.0 } else { k })).collect(),
        SupKind::Rest => vec![([0.0, 1.0, 0.0], false, k)],
        SupKind::Guide => {
            let lat = cross([0.0, 1.0, 0.0], t);
            if norm(lat) < 0.1 { vec![(ax(0), false, k), (ax(2), false, k)] } else { vec![(unit(lat), false, k)] }
        }
        SupKind::Linestop => vec![(unit(t), false, k)],
        SupKind::Restraint { dir } => vec![(unit(*dir), false, k)],
        SupKind::Spring { dir, k } => vec![(unit(*dir), false, *k)],
        SupKind::Rotspring { dir, k } => vec![(unit(*dir), true, *k)],
        SupKind::Constant { .. } => vec![],
    }
}

/// Which global axis a direction is along, if any.
fn axis_of(d: V) -> Option<usize> {
    (0..3).find(|&i| (d[i].abs() - 1.0).abs() < 1e-12)
}

/// A one-way or gapped restraint direction.
struct Cond {
    support: u32,
    node: u32,
    pt: usize,
    dir: V,
    /// 0 = rigid.
    k: f64,
    gap: f64,
    one_way: bool,
    u0: [f64; 3],
}

/// The most solves of one case while one-way and gapped supports settle.
const MAX_ITER: u32 = 100;

/// The light stiffness an open support keeps when nothing else holds the piping in its direction
/// (1 N/mm): only used when the case could not be solved without it.
const K_OPEN: f64 = 1e3;

/// One case solved with its one-way and gapped supports.
struct CondSolve {
    u: Vec<f64>,
    ku: Vec<f64>,
    f: Vec<f64>,
    /// −1 touching on the minus side (pushing the pipe along +dir), +1 on the plus side, 0 open.
    state: Vec<i8>,
    /// The stiffness each support was given as a spring (0: held exactly, or open).
    spring_k: Vec<f64>,
    iterations: u32,
    stats: Stats,
    settled: bool,
    soft: bool,
}

/// Solve one case with its one-way and gapped supports: start touching (a one-way rest without a
/// gap) or open (a gap), solve, and change every support whose state is wrong, until none changes.
/// If the open supports leave the piping free to move, they keep a light stiffness (K_OPEN).
fn solve_conditional(sys: &System, held: &[bool], f: &[f64], fixed: &[f64], conds: &[Cond], dcase: u8) -> Result<CondSolve, String> {
    let mut state: Vec<i8> = conds.iter().map(|c| if c.gap == 0.0 { -1 } else { 0 }).collect();
    let u0 = |c: &Cond| if dcase > 0 { c.u0[dcase as usize - 1] } else { 0.0 };
    let fscale = f.iter().map(|v| v.abs()).fold(0.0, f64::max).max(1.0);
    let (mut iter, mut soft) = (0, false);
    loop {
        iter += 1;
        let (mut s2, mut h2, mut x2, mut f2) = (System { n: sys.n, k: sys.k.clone(), f: vec![] }, held.to_vec(), fixed.to_vec(), f.to_vec());
        let mut spring_k = vec![0.0; conds.len()];
        for (i, c) in conds.iter().enumerate() {
            if state[i] == 0 && !soft {
                continue;
            }
            let target = state[i] as f64 * c.gap + u0(c);
            match (axis_of(c.dir), c.k == 0.0, state[i]) {
                (Some(ax), true, s) if s != 0 && !held[6 * c.pt + ax] => {
                    let dof = 6 * c.pt + ax;
                    h2[dof] = true;
                    x2[dof] = target * c.dir[ax];
                }
                (_, _, s) => {
                    let kk = if s == 0 { K_OPEN } else if c.k == 0.0 { RIGID } else { c.k };
                    spring_k[i] = kk;
                    for a in 0..3 {
                        for b in 0..3 {
                            s2.add(6 * c.pt + a, 6 * c.pt + b, kk * c.dir[a] * c.dir[b]);
                        }
                        f2[6 * c.pt + a] += kk * target * c.dir[a];
                    }
                }
            }
        }
        let (mut sol, stats) = match s2.solve_many(&h2, &[(f2.clone(), x2)]) {
            Ok(x) => x,
            Err(e) if !soft && e.contains("without resistance") && state.contains(&0) => {
                soft = true;
                iter -= 1;
                continue;
            }
            Err(e) => return Err(e),
        };
        let u = sol.remove(0);
        let ku = s2.mul(&u);
        let mut changed = false;
        for (i, c) in conds.iter().enumerate() {
            let v = dot(c.dir, [u[6 * c.pt], u[6 * c.pt + 1], u[6 * c.pt + 2]]) - u0(c);
            let r = conds_reaction(c, state[i], spring_k[i], &u, &ku, &f2, u0(c));
            let next = match state[i] {
                0 if v < -c.gap - 1e-9 => -1,
                0 if v > c.gap + 1e-9 && !c.one_way => 1,
                -1 if r < -1e-9 * fscale => 0,
                1 if r > 1e-9 * fscale => 0,
                s => s,
            };
            if next != state[i] {
                state[i] = next;
                changed = true;
            }
        }
        if !changed || iter >= MAX_ITER {
            return Ok(CondSolve { u, ku, f: f2, state, spring_k, iterations: iter, stats, settled: !changed, soft });
        }
    }
}

/// The force a support puts on the pipe along its direction: held exactly (K u − f at its unknown),
/// as a spring (−k (u − target)), or nothing when open.
fn conds_reaction(c: &Cond, state: i8, spring_k: f64, u: &[f64], ku: &[f64], f: &[f64], u0: f64) -> f64 {
    if spring_k > 0.0 {
        let v = dot(c.dir, [u[6 * c.pt], u[6 * c.pt + 1], u[6 * c.pt + 2]]);
        return -spring_k * (v - (state as f64 * c.gap + u0));
    }
    if state == 0 {
        return 0.0;
    }
    let ax = axis_of(c.dir).unwrap_or(0);
    let dof = 6 * c.pt + ax;
    (ku[dof] - f[dof]) * c.dir[ax]
}

struct Spring {
    support: u32,
    pt: usize,
    dir: V,
    rot: bool,
    k: f64,
    /// Imposed displacement along the direction, cases 1–3.
    u0: [f64; 3],
}

/// Analyse the model: every load case, with stresses, support and nozzle loads.
pub fn analyze(m: &Model) -> Result<Results, String> {
    let t0 = std::time::Instant::now();
    if m.elements.is_empty() {
        return Err("There is nothing to analyze yet: type the piping in the element table.".into());
    }
    let g = Geom::build(m)?;
    let mut warnings = g.warnings.clone();
    let amb = m.settings.ambient;
    // ---------------------------------------------------------------- the parts
    let mut parts: Vec<Part> = vec![];
    for (si, s) in g.segs.iter().enumerate() {
        let mat = m.material(s.material).ok_or("A material is missing.")?;
        let cond = m.cond(s.cond).ok_or("Line conditions are missing.")?;
        let pipe = m.pipe(s.pipe).ok_or("A pipe is missing.")?;
        let e_cold = interp(&mat.e, amb);
        let gmod = e_cold / (2.0 * (1.0 + mat.nu));
        let (pa, pb) = (s.pipe, s.pipe_b);
        let pmax = cond.p.iter().cloned().fold(0.0, f64::max);
        // The flexibility uses the highest pressure (one stiffness for every case); the SIFs the lowest,
        // which is on the safe side for every case.
        let pmin = cond.p.iter().cloned().fold(f64::INFINITY, f64::min).max(0.0);
        let mut sif = [Sif::ONE; 2];
        let mut plane = [None; 2];
        let mut kflex = 1.0;
        let line = match s.shape {
            Shape::Straight => Line::Straight { a: g.points[s.a].p, b: g.points[s.b].p },
            Shape::Arc { centre, normal, r, angle } => {
                let ra = unit(sub(g.points[s.a].p, centre));
                let b = code::bend(pipe.wall, pipe.od, r, if m.settings.bend_pressure { pmax } else { 0.0 }, e_cold);
                let bi = code::bend(pipe.wall, pipe.od, r, if m.settings.bend_pressure { pmin } else { 0.0 }, e_cold);
                kflex = b.k;
                let b = Sif { ii: bi.ii, io: bi.io, ..b };
                sif = [b, b];
                plane = [Some(normal), Some(normal)];
                Line::Arc { centre, ra, tan0: cross(normal, ra), r, angle }
            }
        };
        let rigid = s.rigid.is_some();
        let secf = |x: f64| sec_at(m, pa, pb, x);
        let el = Elem::new(line, &Props { sec: &secf, e: e_cold, g: gmod, k: kflex, rigid })?;
        // Weight: the pipe's own, or a rigid part's weight spread along it, plus the contents.
        let (mut w, mut ww) = (weight(m, s.pipe, mat.rho, false), weight(m, s.pipe, mat.rho, true));
        if let Some((kind, mass)) = s.rigid {
            let len = el.line.length().max(1e-9);
            let metal = weight(m, s.pipe, mat.rho, false) - weight(m, s.pipe, 0.0, false);
            let add = mass * m.settings.g / len - metal;
            w += add;
            ww += add;
            // Flanged joints carry their own SIF (weld neck 1.0, slip-on 1.2).
            if kind == RigidKind::Slipon {
                sif = [Sif { ii: 1.2, io: 1.2, k: 1.0, h: 0.0 }; 2];
            }
        }
        if let Some(Comp::Reducer { to }) = m.elements.iter().find(|e| e.id == s.element).map(|e| e.comp.clone()) {
            // The average weight of the two ends; the SIF from the cone angle and the small end.
            let (w2, ww2) = (weight(m, to, mat.rho, false), weight(m, to, mat.rho, true));
            w = (w + w2) / 2.0;
            ww = (ww + ww2) / 2.0;
            let (big, small) = (secf(0.0), secf(1.0));
            let (d_big, d_small) = (big.od.max(small.od), big.od.min(small.od));
            let len = el.line.length().max(1e-9);
            let alpha = ((d_big - d_small) / 2.0 / len).atan().to_degrees();
            let st = if small.od < big.od { small } else { big };
            sif = [code::reducer(alpha, st.od, st.wall); 2];
        }
        w += s.extra;
        ww += s.extra;
        let mut strain3 = [0, 1, 2].map(|i| strain(&mat.alpha, cond.t[i], amb));
        // Pressure elongation of straight pipe (not bends, rigid parts or reducers), when chosen.
        let is_reducer = s.pipe_b != s.pipe;
        if m.settings.pressure_elongation && matches!(s.shape, Shape::Straight) && !rigid && !is_reducer {
            for i in 0..3 {
                strain3[i] += code::pressure_strain(cond.p[i], pipe.od, pipe.wall, e_cold, mat.nu);
            }
        }
        let _ = si;
        parts.push(Part { el, sec: (secf(0.0), secf(1.0)), w, w_water: ww, strain: strain3, e_cold, mat: mat.id, corrosion: [pipe.corrosion, m.pipe(s.pipe_b).map_or(pipe.corrosion, |q| q.corrosion)], p: [cond.p[0], cond.p[1], cond.p[2], cond.hydro], t: cond.t, sif, plane, ze: [None; 2] });
    }
    // Tees: their SIF at every segment end on the tee's point.
    for t in &g.tees {
        let Some(p) = m.pipe(t.pipe) else { continue };
        let s = code::tee(t.kind, p.wall, p.od, t.pad);
        let branch = t.branch.filter(|_| m.settings.branch_ze).and_then(|(el, bp)| Some((el, m.pipe(bp)?)));
        for (i, seg) in g.segs.iter().enumerate() {
            for (end, pt) in [(0, seg.a), (1, seg.b)] {
                if pt == t.point {
                    let cur = parts[i].sif[end];
                    parts[i].sif[end] = Sif { ii: cur.ii.max(s.ii), io: cur.io.max(s.io), k: cur.k, h: s.h };
                    parts[i].plane[end] = Some(t.normal);
                    if let Some((_, b)) = branch.filter(|(el, _)| *el == seg.element) {
                        let nominal = code::branch_z(b.od, b.wall, p.wall, s.ii);
                        let corroded = code::branch_z(b.od, b.wall - b.corrosion, p.wall - p.corrosion, s.ii);
                        parts[i].ze[end] = Some((nominal, corroded));
                    }
                }
            }
        }
    }
    // ---------------------------------------------------------------- assembly
    let np = g.points.len();
    let n = 6 * np;
    let mut sys = System::new(n);
    for (s, p) in g.segs.iter().zip(&parts) {
        let k = p.el.stiffness();
        let dofs: Vec<usize> = (0..6).map(|i| 6 * s.a + i).chain((0..6).map(|i| 6 * s.b + i)).collect();
        for i in 0..12 {
            for j in 0..12 {
                sys.add(dofs[i], dofs[j], k[i][j]);
            }
        }
    }
    // Supports: held degrees of freedom, and springs.
    let mut held = vec![false; n];
    // Every support that holds a degree of freedom rigidly; two or more share its reaction equally
    // (the split between rigid supports in the same direction cannot be calculated).
    let mut hold_owner: HashMap<usize, Vec<u32>> = HashMap::new();
    let mut shared: Vec<u32> = vec![];
    let mut prescribed: [Vec<f64>; 3] = [vec![0.0; n], vec![0.0; n], vec![0.0; n]];
    let mut springs: Vec<Spring> = vec![];
    let mut conds: Vec<Cond> = vec![];
    for sp in &m.supports {
        let Some(&pt) = g.node_point.get(&sp.node) else { continue };
        let (t, clear) = pipe_dir(&g, pt);
        if !clear && matches!(sp.kind, SupKind::Guide | SupKind::Linestop) {
            warnings.push(format!("The guide or line stop at node {} is at a corner, where the pipe has no single axis: it uses the first element's direction. Put it a little along the run, or use a restraint with its direction.", sp.node));
        }
        for (dir, rot, k) in restraints(&sp.kind, sp.k, t) {
            let u0 = [0, 1, 2].map(|c| { let d = &sp.disp[c]; dot(dir, if rot { [d[3], d[4], d[5]] } else { [d[0], d[1], d[2]] }) });
            if sp.conditional() && !rot {
                conds.push(Cond { support: sp.id, node: sp.node, pt, dir, k, gap: sp.gap, one_way: sp.one_way && sp.kind == SupKind::Rest, u0 });
                continue;
            }
            match (axis_of(dir), k == 0.0) {
                (Some(ax), true) => {
                    let dof = 6 * pt + ax + if rot { 3 } else { 0 };
                    held[dof] = true;
                    let owners = hold_owner.entry(dof).or_default();
                    if !owners.contains(&sp.id) {
                        owners.push(sp.id);
                    }
                    if owners.len() > 1 && !shared.contains(&sp.node) {
                        shared.push(sp.node);
                    }
                    for c in 0..3 {
                        prescribed[c][dof] = u0[c] * dir[ax];
                    }
                }
                _ => {
                    let kk = if k == 0.0 { RIGID } else { k };
                    let off = if rot { 3 } else { 0 };
                    for i in 0..3 {
                        for j in 0..3 {
                            sys.add(6 * pt + off + i, 6 * pt + off + j, kk * dir[i] * dir[j]);
                        }
                    }
                    springs.push(Spring { support: sp.id, pt, dir, rot, k: kk, u0 });
                }
            }
        }
    }
    // ---------------------------------------------------------------- basic loads
    // Weight (with the contents, or with water), the thermal loads of each case, and the spring
    // loads of the imposed displacements of each case.
    let mut w_rhs = [vec![0.0; n], vec![0.0; n]];
    let mut rb_w: [Vec<[f64; 6]>; 2] = [vec![[0.0; 6]; parts.len()], vec![[0.0; 6]; parts.len()]];
    let props_of = |i: usize| (g.segs[i].pipe, g.segs[i].pipe_b, parts[i].sif[0].k, g.segs[i].rigid.is_some());
    for (i, s) in g.segs.iter().enumerate() {
        let (pa, pb, k, rigid) = props_of(i);
        let secf = |x: f64| sec_at(m, pa, pb, x);
        let mat = m.material(s.material).unwrap();
        let pr = Props { sec: &secf, e: parts[i].e_cold, g: parts[i].e_cold / (2.0 * (1.0 + mat.nu)), k: if matches!(s.shape, Shape::Arc { .. }) { k } else { 1.0 }, rigid };
        for (wi, w) in [parts[i].w, parts[i].w_water].into_iter().enumerate() {
            let (la, lb, rb) = parts[i].el.loads([0.0, -w, 0.0], 0.0, &pr);
            for d in 0..6 {
                w_rhs[wi][6 * s.a + d] += la[d];
                w_rhs[wi][6 * s.b + d] += lb[d];
            }
            rb_w[wi][i] = rb;
        }
    }
    let mut point_loads = vec![0.0; n];
    for l in &m.loads {
        if let Some(&pt) = g.node_point.get(&l.node) {
            for d in 0..3 {
                point_loads[6 * pt + d] += l.f[d];
                point_loads[6 * pt + 3 + d] += l.m[d];
            }
        }
    }
    // Constant-force supports push up with their load whenever the weight acts.
    let mut const_loads = vec![0.0; n];
    for sp in &m.supports {
        if let (SupKind::Constant { load }, Some(&pt)) = (&sp.kind, g.node_point.get(&sp.node)) {
            const_loads[6 * pt + 1] += load;
        }
    }
    let mut t_rhs = [vec![0.0; n], vec![0.0; n], vec![0.0; n]];
    for (i, s) in g.segs.iter().enumerate() {
        for c in 0..3 {
            let st = parts[i].strain[c];
            if st == 0.0 {
                continue;
            }
            // K [0; δ] with δ the free elongation: start −Hᵀ K δ, end K δ.
            let chord = sub(parts[i].el.line.end(), parts[i].el.line.start());
            let d = [chord[0] * st, chord[1] * st, chord[2] * st, 0.0, 0.0, 0.0];
            let kd = mv6(&parts[i].el.k, &d);
            let hkd = mv6(&parts[i].el.ht, &kd);
            for k in 0..6 {
                t_rhs[c][6 * s.a + k] -= hkd[k];
                t_rhs[c][6 * s.b + k] += kd[k];
            }
        }
    }
    let mut d_rhs = [vec![0.0; n], vec![0.0; n], vec![0.0; n]];
    for sp in &springs {
        let off = if sp.rot { 3 } else { 0 };
        for c in 0..3 {
            for i in 0..3 {
                d_rhs[c][6 * sp.pt + off + i] += sp.k * sp.u0[c] * sp.dir[i];
            }
        }
    }
    // ---------------------------------------------------------------- the load cases
    let sums: Vec<&LoadCase> = m.cases.iter().filter(|c| matches!(c.recipe, Recipe::Sum { .. })).collect();
    let mut rhs = vec![];
    for c in &sums {
        let Recipe::Sum { w, water, t, d, .. } = c.recipe else { unreachable!() };
        let mut f = vec![0.0; n];
        if w {
            for i in 0..n {
                f[i] += w_rhs[water as usize][i] + point_loads[i] + const_loads[i];
            }
        }
        if t > 0 {
            for i in 0..n {
                f[i] += t_rhs[t as usize - 1][i];
            }
        }
        let fixed = if d > 0 {
            for i in 0..n {
                f[i] += d_rhs[d as usize - 1][i];
            }
            prescribed[d as usize - 1].clone()
        } else {
            vec![0.0; n]
        };
        rhs.push((f, fixed));
    }
    // Linear supports only: every case with one factorisation. With one-way or gapped supports,
    // each case is solved by iteration with its own support states.
    let dcases: Vec<u8> = sums.iter().map(|c| match c.recipe { Recipe::Sum { d, .. } => d, _ => 0 }).collect();
    let (sols, ku, rhs_f, states, springs_k, iters, stats) = if conds.is_empty() {
        let (sols, stats) = sys.solve_many(&held, &rhs)?;
        let ku: Vec<Vec<f64>> = sols.iter().map(|u| sys.mul(u)).collect();
        let n_c = sols.len();
        (sols, ku, rhs.iter().map(|r| r.0.clone()).collect::<Vec<_>>(), vec![vec![]; n_c], vec![vec![]; n_c], vec![1u32; n_c], stats)
    } else {
        let (mut sols, mut kus, mut fs, mut sts, mut sks, mut its) = (vec![], vec![], vec![], vec![], vec![], vec![]);
        let mut stats = Stats::default();
        for (k, (f, fixed)) in rhs.iter().enumerate() {
            let cs = solve_conditional(&sys, &held, f, fixed, &conds, dcases[k]).map_err(|e| {
                if e.contains("without resistance") {
                    format!("In case {}, with the one-way rests that lift off and the gaps that stay open, the piping can move without resistance. Add a support that holds it both ways, such as a guide or an anchor.", sums[k].name)
                } else {
                    e
                }
            })?;
            if cs.soft {
                warnings.push(format!("In case {}, nothing but open gaps holds the piping in some direction: the open supports were given a light stiffness of 1 N/mm so the case could be solved. Their small loads are in the support loads.", sums[k].name));
            }
            if !cs.settled {
                warnings.push(format!("In case {} the one-way and gapped supports did not settle after {MAX_ITER} solves: its results are not reliable. Check for supports that fight each other.", sums[k].name));
            }
            stats.equations = cs.stats.equations;
            stats.nonzeros = cs.stats.nonzeros;
            stats.residual = stats.residual.max(cs.stats.residual);
            sols.push(cs.u);
            kus.push(cs.ku);
            fs.push(cs.f);
            sts.push(cs.state);
            sks.push(cs.spring_k);
            its.push(cs.iterations);
        }
        (sols, kus, fs, sts, sks, its, stats)
    };
    // ---------------------------------------------------------------- results per case
    let mut by_id: HashMap<u32, CaseResult> = HashMap::new();
    let mut raw: HashMap<u32, Vec<[[f64; 6]; 2]>> = HashMap::new();
    let nozzle_axes: Vec<(u32, usize, V)> = m.nozzles.iter().filter_map(|z| {
        let pt = *g.node_point.get(&z.node)?;
        let a = if norm(z.axis) > 1e-9 { unit(z.axis) } else {
            // Along the pipe, pointing from the nozzle into it.
            let s = g.segs.iter().find(|s| s.a == pt || s.b == pt)?;
            let (p, t) = g.at(s, if s.a == pt { 0.0 } else { 1.0 });
            let _ = p;
            if s.a == pt { t } else { scale(t, -1.0) }
        };
        Some((z.id, pt, a))
    }).collect();
    let mut results: Vec<CaseResult> = vec![];
    for c in &m.cases {
        let r = match c.recipe {
            Recipe::Sum { w, water, t, .. } => {
                let k = sums.iter().position(|x| x.id == c.id).unwrap();
                let (u, f) = (&sols[k], &rhs_f[k]);
                let disp: Vec<[f64; 6]> = (0..np).map(|p| std::array::from_fn(|d| u[6 * p + d])).collect();
                // Forces in the pipe: global resultants at each segment's ends.
                let mut glob = vec![[[0.0; 6]; 2]; parts.len()];
                for (i, s) in g.segs.iter().enumerate() {
                    let q = if w { [0.0, -(if water { parts[i].w_water } else { parts[i].w }), 0.0] } else { [0.0; 3] };
                    let rbq = if w { rb_w[water as usize][i] } else { [0.0; 6] };
                    let st = if t > 0 { parts[i].strain[t as usize - 1] } else { 0.0 };
                    let pe = parts[i].el.end_force(&disp[s.a], &disp[s.b], st, &rbq);
                    let l = parts[i].el.line.length();
                    for (end, at) in [(0usize, 0.0), (1, l)] {
                        let (ff, mm, _) = parts[i].el.section(at, &pe, q);
                        glob[i][end] = [ff[0], ff[1], ff[2], mm[0], mm[1], mm[2]];
                    }
                }
                // Reactions: held degrees of freedom (K u − f), and springs (−k (u − u0)).
                let mut reac: HashMap<u32, [f64; 6]> = HashMap::new();
                for (&dof, owners) in &hold_owner {
                    let r = (ku[k][dof] - f[dof]) / owners.len() as f64;
                    for &sid in owners {
                        reac.entry(sid).or_insert([0.0; 6])[dof % 6] += r;
                    }
                }
                let dcase = match c.recipe { Recipe::Sum { d, .. } => d, _ => 0 };
                for sp in &springs {
                    let off = if sp.rot { 3 } else { 0 };
                    let ud = dot(sp.dir, [u[6 * sp.pt + off], u[6 * sp.pt + off + 1], u[6 * sp.pt + off + 2]]);
                    let u0 = if dcase > 0 { sp.u0[dcase as usize - 1] } else { 0.0 };
                    let force = -sp.k * (ud - u0);
                    let e = reac.entry(sp.support).or_insert([0.0; 6]);
                    for i in 0..3 {
                        e[off + i] += force * sp.dir[i];
                    }
                }
                // One-way and gapped supports: the force of each touching one, and every state.
                let mut contacts = vec![];
                for (i, cd) in conds.iter().enumerate() {
                    let s = states[k][i];
                    let u0 = if dcase > 0 { cd.u0[dcase as usize - 1] } else { 0.0 };
                    let r = conds_reaction(cd, s, springs_k[k][i], u, &ku[k], f, u0);
                    let e = reac.entry(cd.support).or_insert([0.0; 6]);
                    for d in 0..3 {
                        e[d] += r * cd.dir[d];
                    }
                    let mv = dot(cd.dir, [u[6 * cd.pt], u[6 * cd.pt + 1], u[6 * cd.pt + 2]]) - u0;
                    let state = match (s, cd.one_way) { (0, true) => "lifted off", (0, false) => "open", _ => "touching" };
                    contacts.push(Contact { support: cd.support, node: cd.node, dir: cd.dir, state, movement: mv, gap: cd.gap });
                }
                // Constant-force supports carry their load in every case with weight.
                for sp in &m.supports {
                    if let SupKind::Constant { load } = sp.kind {
                        reac.entry(sp.id).or_insert([0.0; 6])[1] += if w { load } else { 0.0 };
                    }
                }
                let reactions: Vec<Reaction> = m.supports.iter().filter_map(|sp| reac.get(&sp.id).map(|f| Reaction { support: sp.id, node: sp.node, f: *f })).collect();
                // The applied loads, summed independently of the solver: weight along every
                // segment, point loads and constant supports.
                let mut applied = [0.0; 6];
                if w {
                    for (i, p) in parts.iter().enumerate() {
                        let q = [0.0, -(if water { p.w_water } else { p.w }), 0.0];
                        let l = p.el.line.length();
                        let mq = cross(p.el.line.pint(0.0, l), q);
                        for d in 0..3 {
                            applied[d] += q[d] * l;
                            applied[3 + d] += mq[d];
                        }
                        let _ = i;
                    }
                    for pt in 0..np {
                        let f3 = [point_loads[6 * pt], point_loads[6 * pt + 1], point_loads[6 * pt + 2]];
                        let mo = cross(g.points[pt].p, f3);
                        for d in 0..3 {
                            applied[d] += f3[d];
                            applied[3 + d] += point_loads[6 * pt + 3 + d] + mo[d];
                        }
                    }
                }
                let reacted = sum_reactions(&g, &reactions);
                let (mut raw_glob, cr) = (glob.clone(), CaseResult { id: c.id, name: c.name.clone(), kind: c.kind, disp, forces: vec![], reactions, nozzles: vec![], stresses: vec![], applied, reacted, imbalance: 0.0, worst: None, contacts, iterations: iters[k] });
                raw.insert(c.id, std::mem::take(&mut raw_glob));
                cr
            }
            Recipe::Range { a, b } => {
                let (ra, rb) = (by_id.get(&a).ok_or("A range uses a case that comes after it.")?, by_id.get(&b).ok_or("A range uses a case that comes after it.")?);
                let disp = ra.disp.iter().zip(&rb.disp).map(|(x, y)| std::array::from_fn(|d| x[d] - y[d])).collect();
                let (ga, gb) = (&raw[&a], &raw[&b]);
                let glob: Vec<[[f64; 6]; 2]> = ga.iter().zip(gb).map(|(x, y)| [0, 1].map(|e| std::array::from_fn(|d| x[e][d] - y[e][d]))).collect();
                let reactions: Vec<Reaction> = ra.reactions.iter().map(|x| {
                    let y = rb.reactions.iter().find(|y| y.support == x.support).map(|y| y.f).unwrap_or([0.0; 6]);
                    Reaction { support: x.support, node: x.node, f: std::array::from_fn(|d| x.f[d] - y[d]) }
                }).collect();
                let applied: [f64; 6] = std::array::from_fn(|d| ra.applied[d] - rb.applied[d]);
                let reacted = sum_reactions(&g, &reactions);
                raw.insert(c.id, glob);
                CaseResult { id: c.id, name: c.name.clone(), kind: c.kind, disp, forces: vec![], reactions, nozzles: vec![], stresses: vec![], applied, reacted, imbalance: 0.0, worst: None, contacts: vec![], iterations: 0 }
            }
        };
        by_id.insert(c.id, r.clone());
        results.push(r);
    }
    // ---------------------------------------------------------------- forces, stresses, nozzles
    let f_range = code::range_factor(m.settings.cycles);
    let temp_of = |c: &LoadCase| -> f64 {
        match c.recipe {
            Recipe::Sum { t, p, .. } => {
                let i = if t > 0 { t } else if (1..=3).contains(&p) { p } else { 0 };
                if i > 0 { i as f64 } else { 0.0 }
            }
            Recipe::Range { .. } => 0.0,
        }
    };
    // The largest sustained stress at each segment end (for the liberal allowable range).
    let mut sl_max: Vec<[f64; 2]> = vec![[0.0; 2]; parts.len()];
    for pass in 0..2 {
        for (ci, c) in m.cases.iter().enumerate() {
            let glob = &raw[&c.id];
            let res = &mut results[ci];
            if pass == 0 {
                res.forces = glob.iter().enumerate().map(|(i, e)| [0, 1].map(|end| {
                    let (_, t) = parts[i].el.line.at(if end == 0 { 0.0 } else { parts[i].el.line.length() });
                    let (x, y, z) = axes(t);
                    let f = [e[end][0], e[end][1], e[end][2]];
                    let mm = [e[end][3], e[end][4], e[end][5]];
                    [dot(f, x), dot(f, y), dot(f, z), dot(mm, x), dot(mm, y), dot(mm, z)]
                })).collect();
            }
            let (want, is_sus) = match c.kind {
                CaseKind::Sus | CaseKind::Occ => (pass == 0, true),
                CaseKind::Exp => (pass == 1, false),
                _ => (false, false),
            };
            if !want {
                continue;
            }
            // Pressure and temperature index of the case: SUS uses its pressure case's temperature;
            // a range uses the hotter of its two cases.
            // A range takes Sh at the hotter of its two cases' metal temperatures (found by review: it took
            // the higher-numbered case).
            let (pidx, tids) = match c.recipe {
                Recipe::Sum { p, .. } => (p, vec![temp_of(c) as usize]),
                Recipe::Range { a, b } => {
                    let ca = m.cases.iter().find(|x| x.id == a);
                    let cb = m.cases.iter().find(|x| x.id == b);
                    (0, vec![ca.map_or(0, |x| temp_of(x) as usize), cb.map_or(0, |x| temp_of(x) as usize)])
                }
            };
            let mut stresses = vec![];
            for (i, s) in g.segs.iter().enumerate() {
                let p = &parts[i];
                let mat = m.material(p.mat).unwrap();
                if mat.allow.is_empty() {
                    continue;
                }
                let sc = interp(&mat.allow, m.settings.ambient);
                let hot = if tids.iter().any(|&t| t > 0) { tids.iter().map(|&t| if t > 0 { p.t[t - 1] } else { m.settings.ambient }).fold(m.settings.ambient, f64::max) } else { p.t.iter().cloned().fold(m.settings.ambient, f64::max) };
                let sh = interp(&mat.allow, hot);
                let pressure = if pidx >= 1 { p.p[pidx as usize - 1] } else { 0.0 };
                for end in 0..2 {
                    let e = glob[i][end];
                    let (_, t) = p.el.line.at(if end == 0 { 0.0 } else { p.el.line.length() });
                    let f = [e[0], e[1], e[2]];
                    let mm = [e[3], e[4], e[5]];
                    let mt = dot(mm, t);
                    let mb = sub(mm, scale(t, mt));
                    let (mi, mo) = match p.plane[end] {
                        Some(nrm) => { let mi = dot(mb, nrm); (mi, norm(sub(mb, scale(nrm, mi)))) }
                        None => (norm(mb), 0.0),
                    };
                    let loads = Loads { fa: dot(f, t), mt, mi, mo };
                    let sec = if end == 0 { p.sec.0 } else { p.sec.1 };
                    let sif = p.sif[end];
                    let (stress, allow) = if is_sus {
                        let sl = code::sustained(loads, sif, pressure, sec.od, sec.wall - p.corrosion[end], p.ze[end].map(|z| z.1));
                        if c.kind == CaseKind::Sus {
                            sl_max[i][end] = sl_max[i][end].max(sl);
                        }
                        (sl, if c.kind == CaseKind::Occ { 1.33 * sh } else { sh })
                    } else {
                        let se = code::expansion(loads, sif, sec.od, sec.wall, p.ze[end].map(|z| z.0));
                        (se, code::allowable_range(f_range, sc, sh, m.settings.liberal.then_some(sl_max[i][end])))
                    };
                    let ratio = if allow > 0.0 { stress / allow } else { f64::INFINITY };
                    stresses.push(Stress { seg: i, end: end as u8, point: if end == 0 { s.a } else { s.b }, stress, allow, ratio, pass: ratio <= 1.0, ii: sif.ii, io: sif.io });
                }
            }
            res.worst = stresses.iter().enumerate().max_by(|a, b| a.1.ratio.total_cmp(&b.1.ratio)).map(|x| x.0);
            res.stresses = stresses;
        }
    }
    // The size of the model's loads, so a case with almost no reactions (a free thermal growth) is
    // judged against the loads of the model and not against its own rounding noise.
    let wscale = parts.iter().map(|p| p.w.abs() * p.el.line.length()).sum::<f64>() + point_loads.iter().map(|v| v.abs()).sum::<f64>();
    for res in &mut results {
        // Nozzle loads: what the pipe puts on the nozzle (the support reaction reversed).
        for &(zid, pt, a) in &nozzle_axes {
            let z = m.nozzles.iter().find(|z| z.id == zid).unwrap();
            let mut r = [0.0; 6];
            for x in res.reactions.iter().filter(|x| g.node_point.get(&x.node) == Some(&pt)) {
                for d in 0..6 {
                    r[d] -= x.f[d];
                }
            }
            let (ax, b, cc) = axes(a);
            let f = [r[0], r[1], r[2]];
            let mm = [r[3], r[4], r[5]];
            let local = [dot(f, ax), dot(f, b), dot(f, cc), dot(mm, ax), dot(mm, b), dot(mm, cc)];
            let shear = local[1].hypot(local[2]);
            let bending = local[4].hypot(local[5]);
            let vals = [local[0].abs(), shear, local[3].abs(), bending];
            let ratios: [f64; 4] = std::array::from_fn(|i| if z.allow[i] > 0.0 { vals[i] / z.allow[i] } else { 0.0 });
            let checked = z.allow.iter().any(|v| *v > 0.0);
            res.nozzles.push(NozzleLoad { nozzle: zid, node: z.node, local, shear, bending, ratios, pass: checked.then(|| ratios.iter().all(|r| *r <= 1.0)) });
        }
        res.imbalance = imbalance(&g, res, wscale);
    }
    for z in &m.nozzles {
        if !m.supports.iter().any(|s| s.node == z.node) {
            warnings.push(format!("The nozzle {} at node {} has no support, so it carries no load: put an anchor there.", z.name, z.node));
        }
    }
    // ---------------------------------------------------------------- minimum wall
    let mut walls = vec![];
    // Straight pipe; a bend at its intrados (the bend factor I); a reducer at both ends, each with
    // its own pipe.
    for e in &m.elements {
        let (Some(p), Some(mat), Some(cond)) = (m.pipe(e.pipe), m.material(e.material), m.cond(e.cond)) else { continue };
        if mat.allow.is_empty() {
            continue;
        }
        let mut checks: Vec<(&crate::model::Pipe, f64, &'static str)> = vec![];
        match &e.comp {
            Comp::Reducer { to } => {
                let q = m.pipe(*to).unwrap_or(p);
                let (big, small) = if q.od < p.od { ("large end", "small end") } else { ("small end", "large end") };
                checks.push((p, 1.0, big));
                checks.push((q, 1.0, small));
            }
            Comp::Bend { bend, r } => {
                checks.push((p, 1.0, "pipe"));
                let rb = crate::geom::bend_radius(m, *bend, *r, e.pipe);
                checks.push((p, code::bend_factor(rb, p.od).0, "bend"));
            }
            _ => checks.push((p, 1.0, "pipe")),
        }
        for c in 0..3 {
            if cond.p[c] > 0.0 {
                let s = interp(&mat.allow, cond.t[c].max(m.settings.ambient));
                for &(q, i, at) in &checks {
                    let w = code::wall_at(e.id, c as u8 + 1, cond.p[c], cond.t[c], s, q.od, q.wall, q.corrosion, q.mill, mat.quality, mat.weld, mat.y, i, at);
                    if w.thick && !warnings.iter().any(|x| x.contains(&format!("element {}–{} ", e.from, e.to))) {
                        warnings.push(format!("The pressure design thickness of element {}–{} is D/6 or more: the thin-wall equation does not apply, so its wall check fails. It needs a thick-wall design.", e.from, e.to));
                    }
                    walls.push(w);
                }
            }
        }
    }
    for n in &shared {
        warnings.push(format!("Two or more supports at node {n} hold the same direction rigidly: they share its load equally, since the split between rigid supports cannot be calculated. Keep one, or make one flexible."));
    }
    let mut stats = stats;
    stats.ms = t0.elapsed().as_secs_f64() * 1e3;
    let sifs = parts.iter().map(|p| p.sif).collect();
    Ok(Results { geom: g, cases: results, walls, sifs, stats, warnings, version: env!("CARGO_PKG_VERSION") })
}

/// The reactions' total force and moment about the origin.
fn sum_reactions(g: &Geom, r: &[Reaction]) -> [f64; 6] {
    let mut s = [0.0; 6];
    for x in r {
        let Some(&pt) = g.node_point.get(&x.node) else { continue };
        let f = [x.f[0], x.f[1], x.f[2]];
        let mo = cross(g.points[pt].p, f);
        for d in 0..3 {
            s[d] += f[d];
            s[3 + d] += x.f[3 + d] + mo[d];
        }
    }
    s
}

/// The equilibrium check: the largest imbalance of the forces, and of the moments (against the
/// forces times the model's size), relative to the size of the loads (or of the reactions when
/// nothing is applied, as in a thermal case), and never less than the model's weight or 1 N.
fn imbalance(g: &Geom, r: &CaseResult, wscale: f64) -> f64 {
    let scale_f = r.applied[..3].iter().map(|v| v.abs()).sum::<f64>().max(r.reactions.iter().map(|x| x.f[..3].iter().map(|v| v.abs()).sum::<f64>()).sum::<f64>()).max(wscale).max(1.0);
    let reach = g.points.iter().map(|p| norm(p.p)).fold(1.0, f64::max);
    let scale_m = (scale_f * reach).max(r.reactions.iter().map(|x| x.f[3..].iter().map(|v| v.abs()).sum::<f64>()).sum::<f64>());
    let df = (0..3).map(|d| (r.applied[d] + r.reacted[d]).abs()).fold(0.0, f64::max) / scale_f.max(1e-300);
    let dm = (3..6).map(|d| (r.applied[d] + r.reacted[d]).abs()).fold(0.0, f64::max) / scale_m.max(1e-300);
    df.max(dm)
}
