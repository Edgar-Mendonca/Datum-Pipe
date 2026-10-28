//! The verification benchmarks (brief §4): each is a model solved by the engine and compared with a
//! reference worked out independently: beam theory, the flexibility (elastic centre) method by hand
//! for heated L-bends, Z-bends and expansion loops, a bend's closed-form end rotation, and the
//! minimum wall equation. `run()` gives the rows of Help › Verification; the test fails on any row
//! over its tolerance, so the report is always true for the version that runs it.

use crate::doc::{Item, Op};
use crate::examples::table;
use crate::model::{BendKind, Comp, Load, Model, SupKind, Support, K0};
use crate::solve::{analyze, Results};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Check {
    pub bench: u32,
    pub name: String,
    pub quantity: String,
    /// The unit key the window shows it in.
    pub q: &'static str,
    pub reference: f64,
    pub result: f64,
    pub error: f64,
    pub tolerance: f64,
    pub source: String,
    pub pass: bool,
}

fn row(bench: u32, name: &str, quantity: &str, q: &'static str, reference: f64, result: f64, tol: f64, source: &str) -> Check {
    let error = if reference.abs() > 0.0 { ((result - reference) / reference).abs() } else { result.abs() };
    Check { bench, name: name.into(), quantity: quantity.into(), q, reference, result, error, tolerance: tol, source: source.into(), pass: error <= tol && result.is_finite() }
}

const TOL: f64 = 1e-6;

/// A 6" Sch 40 carbon steel pipe's properties as a hand calculation would take them.
struct Hand {
    e: f64,
    g: f64,
    a: f64,
    i: f64,
}
fn hand(m: &Model) -> Hand {
    let p = &m.pipes[0];
    let mat = &m.materials[0];
    let e = crate::library::interp(&mat.e, m.settings.ambient);
    let id = p.od - 2.0 * p.wall;
    let pi = std::f64::consts::PI;
    Hand { e, g: e / (2.0 * (1.0 + mat.nu)), a: pi / 4.0 * (p.od.powi(2) - id.powi(2)), i: pi / 64.0 * (p.od.powi(4) - id.powi(4)) }
}

fn add(m: &mut crate::doc::Doc, item: Item) {
    m.apply(Op::Add { item, after: None }).expect("benchmark edit");
}
fn anchor(d: &mut crate::doc::Doc, node: u32) {
    add(d, Item::Support(Support { node, kind: SupKind::Anchor, ..Default::default() }));
}
fn case<'a>(r: &'a Results, name: &str) -> &'a crate::solve::CaseResult {
    r.cases.iter().find(|c| c.name == name).expect("case")
}
fn disp(r: &Results, c: &str, node: u32) -> [f64; 6] {
    case(r, c).disp[r.geom.node_point[&node]]
}
fn reaction(r: &Results, c: &str, node: u32) -> [f64; 6] {
    case(r, c).reactions.iter().find(|x| x.node == node).map(|x| x.f).unwrap_or([0.0; 6])
}

/// The hand flexibility method for a plane frame of straight legs held at both ends (the
/// elastic centre method): with the start held, unit loads at the end give N, V and M along every
/// leg; the flexibility is ∫ (Ni Nj / EA + Vi Vj / κGA + Mi Mj / EI) ds (Simpson's rule, exact for
/// these polynomials), and the end's reaction to a free thermal growth δ is −F⁻¹ δ. Returns the
/// end's reaction (Fx, Fy, Mz) in the plane of the legs.
pub fn plane_frame(pts: &[[f64; 2]], h: (f64, f64, f64), strain: f64) -> [f64; 3] {
    let (ea, gas, ei) = h;
    let end = pts[pts.len() - 1];
    let mut f = [[0.0; 3]; 3];
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let l = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        let t = [(b[0] - a[0]) / l, (b[1] - a[1]) / l];
        let nrm = [-t[1], t[0]];
        for (s, wt) in [(0.0, 1.0 / 6.0), (0.5, 4.0 / 6.0), (1.0, 1.0 / 6.0)] {
            let p = [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s];
            let r = [end[0] - p[0], end[1] - p[1]];
            // Unit loads at the end: Fx, Fy, Mz.
            let res = |k: usize| -> (f64, f64, f64) {
                let (fx, fy, mz) = [(1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)][k];
                (fx * t[0] + fy * t[1], fx * nrm[0] + fy * nrm[1], mz + r[0] * fy - r[1] * fx)
            };
            for i in 0..3 {
                for j in 0..3 {
                    let (ni, vi, mi) = res(i);
                    let (nj, vj, mj) = res(j);
                    f[i][j] += wt * l * (ni * nj / ea + vi * vj / gas + mi * mj / ei);
                }
            }
        }
    }
    let d = [strain * (end[0] - pts[0][0]), strain * (end[1] - pts[0][1]), 0.0];
    // Solve F R = −d (3×3, Cramer's rule).
    let det = |m: &[[f64; 3]; 3]| m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    let d0 = det(&f);
    let mut out = [0.0; 3];
    for k in 0..3 {
        let mut mk = f;
        for i in 0..3 {
            mk[i][k] = -d[i];
        }
        out[k] = det(&mk) / d0;
    }
    out
}

/// A model of straight legs in the XY plane, anchored at both ends, heated to `t_c` °C, without
/// weight: for the thermal benchmarks.
fn heated(name: &str, legs: &[(f64, f64)], t_c: f64) -> Model {
    let mut rows = vec![];
    for (k, (dx, dy)) in legs.iter().enumerate() {
        rows.push((10 * (k as u32 + 1), 10 * (k as u32 + 2), *dx, *dy, 0.0));
    }
    let mut d = table(name, &rows);
    let mut s = d.m.settings.clone();
    s.g = 0.0;
    d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
    let mut c = d.m.conds[0].clone();
    c.t[0] = K0 + t_c;
    d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
    anchor(&mut d, 10);
    anchor(&mut d, 10 * (legs.len() as u32 + 1));
    d.m
}

/// The thermal strain from 21 °C to 200 °C by hand: the mean coefficient at 200 °C in the carbon
/// steel table (12.3 µm/m·K) times 179 K.
const STRAIN_200: f64 = 12.3e-6 * 179.0;

pub fn run() -> Vec<Check> {
    let mut out = vec![];
    // 1. Cantilever: 3 m of 6" Sch 40 in 5 elements, anchored at 10, 10 kN down at the tip.
    {
        let mut d = table("Cantilever", &[(10, 20, 0.6, 0.0, 0.0), (20, 30, 0.6, 0.0, 0.0), (30, 40, 0.6, 0.0, 0.0), (40, 50, 0.6, 0.0, 0.0), (50, 60, 0.6, 0.0, 0.0)]);
        let mut s = d.m.settings.clone();
        s.g = 0.0;
        d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
        anchor(&mut d, 10);
        add(&mut d, Item::Load(Load { node: 60, f: [0.0, -10e3, 0.0], m: [0.0; 3], ..Default::default() }));
        let h = hand(&d.m);
        let (p, l) = (10e3, 3.0f64);
        let r = analyze(&d.m).unwrap();
        let want = p * l.powi(3) / (3.0 * h.e * h.i) + p * l / (crate::elements::KAPPA * h.g * h.a);
        out.push(row(1, "Cantilever with an end load", "Tip deflection", "disp", -want, disp(&r, "OPE1", 60)[1], TOL, "Beam theory: PL³/3EI + PL/κGA (shear, κ = 0.5)"));
        out.push(row(1, "Cantilever with an end load", "Root moment", "moment", p * l, reaction(&r, "OPE1", 10)[5], TOL, "Statics: P L"));
        out.push(row(1, "Cantilever with an end load", "Tip rotation", "rot", -p * l * l / (2.0 * h.e * h.i), disp(&r, "OPE1", 60)[5], TOL, "Beam theory: PL²/2EI"));
    }
    // 2. Own weight between two anchors: 6 m of 6" Sch 40 full of water, 10 elements.
    {
        let rows: Vec<(u32, u32, f64, f64, f64)> = (0..10).map(|k| (10 * (k + 1), 10 * (k + 2), 0.6, 0.0, 0.0)).collect();
        let mut d = table("Weight span", &rows);
        let mut p = d.m.pipes[0].clone();
        p.fluid_rho = 1000.0;
        d.apply(Op::Set { item: Item::Pipe(p.clone()) }).unwrap();
        anchor(&mut d, 10);
        anchor(&mut d, 110);
        let h = hand(&d.m);
        let pi = std::f64::consts::PI;
        let id = p.od - 2.0 * p.wall;
        let w = 9.806_65 * (d.m.materials[0].rho * pi / 4.0 * (p.od.powi(2) - id.powi(2)) + 1000.0 * pi / 4.0 * id * id);
        let l = 6.0f64;
        let r = analyze(&d.m).unwrap();
        let want = w * l.powi(4) / (384.0 * h.e * h.i) + w * l * l / (8.0 * crate::elements::KAPPA * h.g * h.a);
        out.push(row(2, "Pipe under its own weight between two anchors", "Mid-span deflection", "disp", -want, disp(&r, "SUS", 60)[1], TOL, "Beam theory: wL⁴/384EI + wL²/8κGA"));
        out.push(row(2, "Pipe under its own weight between two anchors", "End moment", "moment", w * l * l / 12.0, reaction(&r, "SUS", 10)[5], TOL, "Beam theory: wL²/12"));
        out.push(row(2, "Pipe under its own weight between two anchors", "End reaction", "force", w * l / 2.0, reaction(&r, "SUS", 10)[1], TOL, "Statics: wL/2"));
    }
    // 3. L-bend between anchors, heated to 200 °C: legs 4 m along X and 3 m along Y.
    {
        let m = heated("L-bend", &[(4.0, 0.0), (0.0, 3.0)], 200.0);
        let h = hand(&m);
        let want = plane_frame(&[[0.0, 0.0], [4.0, 0.0], [4.0, 3.0]], (h.e * h.a, crate::elements::KAPPA * h.g * h.a, h.e * h.i), STRAIN_200);
        let r = analyze(&m).unwrap();
        let rb = reaction(&r, "OPE1", 30);
        let src = "Flexibility (elastic centre) method by hand";
        out.push(row(3, "L-bend between two anchors, heated", "End force along X", "force", want[0], rb[0], TOL, src));
        out.push(row(3, "L-bend between two anchors, heated", "End force along Y", "force", want[1], rb[1], TOL, src));
        out.push(row(3, "L-bend between two anchors, heated", "End moment", "moment", want[2], rb[5], TOL, src));
    }
    // 4. Z-bend: 3 m along X, 4 m along Y, 3 m along X, heated to 200 °C.
    {
        let m = heated("Z-bend", &[(3.0, 0.0), (0.0, 4.0), (3.0, 0.0)], 200.0);
        let h = hand(&m);
        let want = plane_frame(&[[0.0, 0.0], [3.0, 0.0], [3.0, 4.0], [6.0, 4.0]], (h.e * h.a, crate::elements::KAPPA * h.g * h.a, h.e * h.i), STRAIN_200);
        let r = analyze(&m).unwrap();
        let rb = reaction(&r, "OPE1", 40);
        let src = "Flexibility (elastic centre) method by hand";
        out.push(row(4, "Z-bend between two anchors, heated", "End force along X", "force", want[0], rb[0], TOL, src));
        out.push(row(4, "Z-bend between two anchors, heated", "End force along Y", "force", want[1], rb[1], TOL, src));
        out.push(row(4, "Z-bend between two anchors, heated", "End moment", "moment", want[2], rb[5], TOL, src));
    }
    // 5. A 6" long-radius elbow, anchored at one end, a moment in its plane at the other:
    //    θ = k M (π/2) R / EI with k = 1.65 / h, h = T R / r².
    {
        let rb = 1.5 * 6.0 * 0.0254;
        let mut d = table("Elbow", &[(10, 20, rb, 0.0, 0.0), (20, 30, 0.0, rb, 0.0)]);
        let mut s = d.m.settings.clone();
        s.g = 0.0;
        d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
        let mut e = d.m.elements[0].clone();
        e.comp = Comp::Bend { bend: BendKind::Long, r: 0.0 };
        d.apply(Op::Set { item: Item::Element(e) }).unwrap();
        anchor(&mut d, 10);
        add(&mut d, Item::Load(Load { node: 30, f: [0.0; 3], m: [0.0, 0.0, 1000.0], ..Default::default() }));
        let h = hand(&d.m);
        let p = &d.m.pipes[0];
        let rm = (p.od - p.wall) / 2.0;
        let hh = p.wall * rb / (rm * rm);
        let k = 1.65 / hh;
        let want = k * 1000.0 * std::f64::consts::FRAC_PI_2 * rb / (h.e * h.i);
        let r = analyze(&d.m).unwrap();
        out.push(row(5, "A bend's flexibility (6\" long-radius elbow)", "End rotation under an in-plane moment", "rot", want, disp(&r, "OPE1", 30)[5], TOL, "k = 1.65/h, h = T·R/r²; θ = k M R (π/2) / EI"));
        out.push(row(5, "A bend's flexibility (6\" long-radius elbow)", "Flexibility factor k", "ratio", 6.590, k, 1e-3, "h = 0.2504 by hand"));
    }
    // 6. Minimum wall and hoop stress: 6" Sch 40, P = 10 MPa, S = 138 MPa, E = W = 1, Y = 0.4.
    {
        let w = crate::code::wall(0, 1, 10e6, K0 + 21.0, 138e6, 0.168_275, 0.007_112, 0.0015, 0.125, 1.0, 1.0, 0.4);
        // By hand: t = 10 × 168.275 / (2 (138 + 0.4 × 10)) = 5.925176 mm; + 1.5 mm corrosion =
        // 7.425176 mm; / 0.875 = 8.485915 mm nominal needed.
        let src = "Pressure design thickness equation (para. 304.1.2) by hand";
        out.push(row(6, "Minimum wall for internal pressure", "Pressure design thickness", "len", 0.005_925_176, w.t_design, 1e-6, src));
        out.push(row(6, "Minimum wall for internal pressure", "Nominal wall needed (corrosion, 12.5 % mill)", "len", 0.008_485_915, w.t_nominal_needed, 1e-6, src));
        // Hoop stress on the wall left: 7.112 × 0.875 − 1.5 = 4.723 mm: 10 (168.275 − 0.8 × 4.723) / (2 × 4.723) = 174.144 MPa.
        out.push(row(6, "Minimum wall for internal pressure", "Hoop stress on the remaining wall", "stress", 174.144_188e6, w.hoop, 1e-6, src));
    }
    // 7. Expansion loop: 10 m along X, 4 m up, 3 m along X, 4 m down, 10 m along X, anchored at both
    //    ends and heated to 200 °C.
    {
        let m = heated("Expansion loop", &[(10.0, 0.0), (0.0, 4.0), (3.0, 0.0), (0.0, -4.0), (10.0, 0.0)], 200.0);
        let h = hand(&m);
        let want = plane_frame(&[[0.0, 0.0], [10.0, 0.0], [10.0, 4.0], [13.0, 4.0], [13.0, 0.0], [23.0, 0.0]], (h.e * h.a, crate::elements::KAPPA * h.g * h.a, h.e * h.i), STRAIN_200);
        let r = analyze(&m).unwrap();
        let ra = reaction(&r, "OPE1", 60);
        let src = "Expansion loop by the elastic centre method";
        out.push(row(7, "Expansion loop between two anchors, heated", "Anchor force along the line", "force", want[0], ra[0], TOL, src));
        out.push(row(7, "Expansion loop between two anchors, heated", "Anchor moment", "moment", want[2], ra[5], TOL, src));
    }
    // 8. Equilibrium on every example: in every load case the reactions balance the loads.
    for e in crate::examples::list() {
        let Some(m) = crate::examples::get(e.id) else { continue };
        if m.elements.is_empty() {
            continue;
        }
        match analyze(&m) {
            Ok(r) => {
                let worst = r.cases.iter().map(|c| c.imbalance).fold(0.0, f64::max);
                out.push(row(8, &format!("Equilibrium: {}", e.name), "Largest imbalance in any load case", "ratio", 0.0, worst, 1e-9, "Statics: Σ reactions + Σ loads = 0"));
            }
            Err(x) => out.push(Check { bench: 8, name: format!("Equilibrium: {}", e.name), quantity: x, q: "ratio", reference: 0.0, result: f64::NAN, error: f64::NAN, tolerance: 1e-9, source: "Statics".into(), pass: false }),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_benchmark_passes() {
        let rows = super::run();
        let bad: Vec<String> = rows.iter().filter(|r| !r.pass).map(|r| format!("{} {} {}: {:e} vs {:e} ({:e})", r.bench, r.name, r.quantity, r.result, r.reference, r.error)).collect();
        assert!(bad.is_empty(), "{}", bad.join("\n"));
        for b in [1, 2, 3, 4, 5, 6, 7, 8] {
            assert!(rows.iter().any(|r| r.bench == b), "benchmark {b} is missing");
        }
    }
}
