//! The verification benchmarks (brief §4): each is a model solved by the engine and compared with a
//! reference worked out independently: beam theory, the flexibility (elastic centre) method by hand
//! for heated L-bends, Z-bends and expansion loops, a bend's closed-form end rotation, and the
//! minimum wall equation (straight pipe and a bend's intrados), a tee branch's effective section
//! modulus, the axial force of pressure elongation between anchors, and a one-way rest and a gap
//! (the cantilever by beam theory, with the support touching or open), and friction at a rest
//! (sliding and sticking, by statics and Hooke's law), and wind, static seismic and the
//! occasional stress on a cantilever (beam theory), and a spring hanger sized by statics and free
//! thermal growth, a steam turbine's nozzle criteria with its nozzle loads by statics, a
//! flange's equivalent pressure, a relief valve's thrust at an open end, a cantilever from a
//! flexible nozzle (springs in series), an expansion joint's rates and pressure thrust, wind that changes with height in both
//! directions, a vertical vessel's shell and heads, a vessel nozzle's reinforcement, a horizontal vessel on two saddles, a vertical vessel on a skirt, a bolted cover, and lifting lugs. `run()` gives the rows of Help › Verification; the test fails on any row
//! over its tolerance, so the report is always true for the version that runs it.

use crate::doc::{Item, Op};
use crate::examples::table;
use crate::model::{BendKind, Comp, Load, Model, Pipe, SupKind, Support, TeeKind, K0};
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
    // 9. A bend's minimum wall at its intrados: 6" Sch 40 long-radius elbow (R = 228.6 mm),
    //    P = 10 MPa, S = 138 MPa, E = W = 1, Y = 0.4, no corrosion.
    {
        // By hand: 4R/D = 914.4 / 168.275 = 5.433962; I = 4.433962 / 3.433962 = 1.291209;
        // t = 10 × 168.275 / (2 (138 / 1.291209 + 4)) = 7.588391 mm.
        let i = crate::code::bend_factor(0.2286, 0.168_275).0;
        let w = crate::code::wall_at(0, 1, 10e6, K0 + 21.0, 138e6, 0.168_275, 0.007_112, 0.0, 0.125, 1.0, 1.0, 0.4, i, "bend");
        let src = "Bend factor from the membrane equilibrium of a torus, by hand";
        out.push(row(9, "Minimum wall of a bend at its intrados", "Bend factor I at the intrados", "ratio", 1.291_208_791, i, 1e-9, src));
        out.push(row(9, "Minimum wall of a bend at its intrados", "Pressure design thickness", "len", 0.007_588_391, w.t_design, 1e-6, src));
    }
    // 10. Pressure elongation: 6 m of 6" Sch 40 between two anchors at 10 MPa, no weight, no heat.
    //     The anchors hold the free growth back: N = E A ε, ε = (σL − ν σh) / E.
    {
        let rows: Vec<(u32, u32, f64, f64, f64)> = (0..6).map(|k| (10 * (k + 1), 10 * (k + 2), 1.0, 0.0, 0.0)).collect();
        let mut d = table("Pressure elongation", &rows);
        let mut s = d.m.settings.clone();
        s.g = 0.0;
        s.pressure_elongation = true;
        d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
        let mut c = d.m.conds[0].clone();
        c.p[0] = 10e6;
        d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
        anchor(&mut d, 10);
        anchor(&mut d, 70);
        let h = hand(&d.m);
        let p = &d.m.pipes[0];
        let di = p.od - 2.0 * p.wall;
        let (sl, sh) = (10e6 * di * di / (p.od * p.od - di * di), 10e6 * (p.od - p.wall) / (2.0 * p.wall));
        let want = h.e * h.a * (sl - d.m.materials[0].nu * sh) / h.e;
        let r = analyze(&d.m).unwrap();
        out.push(row(10, "Pressure elongation between two anchors", "Anchor force along the line", "force", -want, reaction(&r, "OPE1", 70)[0], TOL, "Hooke's law: N = E A (σL − ν σh) / E"));
    }
    // 11. A tee branch's effective section modulus: 6" Sch 40 run 10–20–30 along X with a welding
    //     tee at 20, a 2" Sch 40 branch 20–40 of 2 m along Z, 1 kN up at its end, no weight.
    {
        let mut d = table("Tee branch", &[(10, 20, 3.0, 0.0, 0.0), (20, 30, 3.0, 0.0, 0.0)]);
        let mut s = d.m.settings.clone();
        s.g = 0.0;
        d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
        let o = d.apply(Op::Add { item: Item::Pipe(Pipe::std(0, "2", "40")), after: None }).unwrap();
        let e = crate::model::Element { from: 20, to: 40, d: [0.0, 0.0, 2.0], pipe: o[0], material: d.m.materials[0].id, cond: d.m.conds[0].id, ..Default::default() };
        add(&mut d, Item::Element(e));
        let mut e = d.m.elements[0].clone();
        e.comp = Comp::Tee { tee: TeeKind::Welding, pad: 0.0 };
        d.apply(Op::Set { item: Item::Element(e) }).unwrap();
        anchor(&mut d, 10);
        anchor(&mut d, 30);
        add(&mut d, Item::Load(Load { node: 40, f: [0.0, 1e3, 0.0], m: [0.0; 3], ..Default::default() }));
        // By hand: h = 3.1 T / r = 0.273601, io = 0.9 / h^⅔ = 2.135488, ii = 0.75 io + 0.25 =
        // 1.851616; Ts = min(7.112, 1.851616 × 3.9116) = 7.112 mm; Ze = π (28.2067 mm)² × 7.112 mm =
        // 1.777650e-5 m³; the out-of-plane moment 2 kN·m with Io = 0.75 io = 1.601616:
        // SL = 1.601616 × 2000 / 1.777650e-5 = 180.1948 MPa.
        let r = analyze(&d.m).unwrap();
        let pt = r.geom.node_point[&20];
        let br = d.m.elements.iter().find(|e| e.to == 40).unwrap().id;
        let sl = case(&r, "SUS").stresses.iter().find(|x| x.point == pt && r.geom.segs[x.seg].element == br).map_or(f64::NAN, |x| x.stress);
        out.push(row(11, "A tee branch's effective section modulus", "Sustained stress in the branch at the tee", "stress", 180.194_774_5e6, sl, 1e-6, "Ze = π r₂² Ts with Markl's tee factors, by hand"));
    }
    // 12. A one-way rest under a cantilever's tip: 3 m of 6" Sch 40 anchored at 10, the rest at 60,
    //     10 kN at the tip. Pushed up, the pipe lifts off and the tip moves as a free cantilever;
    //     pushed down, the rest carries the whole load and the tip stays put.
    {
        let tip = |p: f64| {
            let mut d = table("One-way rest", &[(10, 20, 0.6, 0.0, 0.0), (20, 30, 0.6, 0.0, 0.0), (30, 40, 0.6, 0.0, 0.0), (40, 50, 0.6, 0.0, 0.0), (50, 60, 0.6, 0.0, 0.0)]);
            let mut s = d.m.settings.clone();
            s.g = 0.0;
            d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
            anchor(&mut d, 10);
            add(&mut d, Item::Support(Support { node: 60, kind: SupKind::Rest, one_way: true, ..Default::default() }));
            add(&mut d, Item::Load(Load { node: 60, f: [0.0, p, 0.0], m: [0.0; 3], ..Default::default() }));
            let h = hand(&d.m);
            (analyze(&d.m).unwrap(), h)
        };
        let (r, h) = tip(10e3);
        let free = 10e3 * 27.0 / (3.0 * h.e * h.i) + 10e3 * 3.0 / (crate::elements::KAPPA * h.g * h.a);
        let name = "A one-way rest under a cantilever's tip";
        out.push(row(12, name, "Pushed up: tip lifts as a free cantilever", "disp", free, disp(&r, "OPE1", 60)[1], TOL, "Beam theory: PL³/3EI + PL/κGA, rest open"));
        out.push(row(12, name, "Pushed up: rest load", "force", 0.0, reaction(&r, "OPE1", 60)[1], 1e-9, "The rest cannot pull"));
        let (r, _) = tip(-10e3);
        out.push(row(12, name, "Pushed down: rest load", "force", 10e3, reaction(&r, "OPE1", 60)[1], TOL, "Statics: the rest carries P"));
    }
    // 13. A gap closing: the same cantilever with a Z restraint at the tip that has a 5 mm gap on each
    //     side, and 10 kN along Z. The free tip would move P c (c = L³/3EI + L/κGA, about 38 mm), so
    //     the gap closes: the tip stops at 5 mm and the restraint takes P − g / c.
    {
        let mut d = table("Gap", &[(10, 20, 0.6, 0.0, 0.0), (20, 30, 0.6, 0.0, 0.0), (30, 40, 0.6, 0.0, 0.0), (40, 50, 0.6, 0.0, 0.0), (50, 60, 0.6, 0.0, 0.0)]);
        let mut s = d.m.settings.clone();
        s.g = 0.0;
        d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
        anchor(&mut d, 10);
        add(&mut d, Item::Support(Support { node: 60, kind: SupKind::Restraint { dir: [0.0, 0.0, 1.0] }, gap: 0.005, ..Default::default() }));
        add(&mut d, Item::Load(Load { node: 60, f: [0.0, 0.0, 10e3], m: [0.0; 3], ..Default::default() }));
        let h = hand(&d.m);
        let c = 27.0 / (3.0 * h.e * h.i) + 3.0 / (crate::elements::KAPPA * h.g * h.a);
        let r = analyze(&d.m).unwrap();
        let name = "A gap closing at a cantilever's tip";
        out.push(row(13, name, "Tip movement (the gap)", "disp", 0.005, disp(&r, "OPE1", 60)[2], TOL, "The gap: 5 mm"));
        out.push(row(13, name, "Restraint load", "force", -(10e3 - 0.005 / c), reaction(&r, "OPE1", 60)[2], TOL, "Beam theory: P − g / c, c = L³/3EI + L/κGA"));
    }
    // 14. Friction at a rest: 6 m of 6" Sch 40 along X, anchored at 10, a one-way rest with μ = 0.3
    //     at 20 carrying a load P straight down, heated to 200 °C, no weight. The rest carries P, so
    //     friction can hold at most μ P. With P = 10 kN the pipe slides: the rest pushes back μ P and
    //     the end moves ε L − μ P L / EA. With P = 10 MN the pipe sticks: it does not move, and the
    //     rest holds back E A ε (less than μ P).
    {
        let build = |p: f64| {
            let mut d = table("Friction", &[(10, 20, 6.0, 0.0, 0.0)]);
            let mut s = d.m.settings.clone();
            s.g = 0.0;
            d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
            let mut c = d.m.conds[0].clone();
            c.t[0] = K0 + 200.0;
            d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
            anchor(&mut d, 10);
            add(&mut d, Item::Support(Support { node: 20, kind: SupKind::Rest, one_way: true, mu: 0.3, ..Default::default() }));
            add(&mut d, Item::Load(Load { node: 20, f: [0.0, -p, 0.0], m: [0.0; 3], ..Default::default() }));
            let h = hand(&d.m);
            (analyze(&d.m).unwrap(), h)
        };
        let name = "Friction at a rest, heated";
        let (r, h) = build(10e3);
        out.push(row(14, name, "Sliding: friction force (μ P)", "force", -0.3 * 10e3, reaction(&r, "OPE1", 20)[0], TOL, "Coulomb friction: μ N, N = P by statics"));
        out.push(row(14, name, "Sliding: end movement", "disp", STRAIN_200 * 6.0 - 0.3 * 10e3 * 6.0 / (h.e * h.a), disp(&r, "OPE1", 20)[0], TOL, "Hooke's law: ε L − μ P L / EA"));
        let (r, h) = build(10e6);
        out.push(row(14, name, "Sticking: friction force (E A ε < μ P)", "force", -h.e * h.a * STRAIN_200, reaction(&r, "OPE1", 20)[0], TOL, "Hooke's law: E A ε"));
        out.push(row(14, name, "Sticking: end movement", "disp", 0.0, disp(&r, "OPE1", 20)[0], 1e-12, "The rest holds the pipe"));
    }
    // 15–17. Occasional loads on a cantilever: 3 m of 6" Sch 40 along X in 5 elements, anchored at 10.
    {
        let cant = |wind: f64, seis: [f64; 3], g: f64| {
            let mut d = table("Occasional", &[(10, 20, 0.6, 0.0, 0.0), (20, 30, 0.6, 0.0, 0.0), (30, 40, 0.6, 0.0, 0.0), (40, 50, 0.6, 0.0, 0.0), (50, 60, 0.6, 0.0, 0.0)]);
            let mut s = d.m.settings.clone();
            s.g = g;
            s.wind_q = wind;
            s.wind_cd = 0.7;
            s.wind_dir = [0.0, 0.0, 1.0];
            s.seis = seis;
            d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
            anchor(&mut d, 10);
            let h = hand(&d.m);
            (analyze(&d.m).unwrap(), h, d.m)
        };
        // 15. Wind of 1 kPa along Z with Cd = 0.7: q = 1000 × 0.7 × 0.168275 = 117.79 N/m across the pipe.
        let (r, h, m) = cant(1000.0, [0.0; 3], 0.0);
        let q = 1000.0 * 0.7 * m.pipes[0].od;
        let name = "Wind on a cantilever";
        out.push(row(15, name, "Tip deflection (OCC WIN)", "disp", q * 81.0 / (8.0 * h.e * h.i) + q * 9.0 / (2.0 * crate::elements::KAPPA * h.g * h.a), disp(&r, "OCC WIN", 60)[2], TOL, "Beam theory: qL⁴/8EI + qL²/2κGA, q = p Cd D"));
        out.push(row(15, name, "Root moment (OCC WIN)", "moment", q * 9.0 / 2.0, reaction(&r, "OCC WIN", 10)[4], TOL, "Statics: qL²/2"));
        // 16. Static seismic of 0.3 g along Z: q = 0.3 w, w the pipe's weight per metre.
        let (r, h, m) = cant(0.0, [0.0, 0.0, 0.3], 9.806_65);
        let w = m.materials[0].rho * h.a * 9.806_65;
        let name = "Static seismic on a cantilever";
        out.push(row(16, name, "Root force (OCC SEIS)", "force", -0.3 * w * 3.0, reaction(&r, "OCC SEIS", 10)[2], TOL, "Statics: 0.3 w L"));
        out.push(row(16, name, "Root moment (OCC SEIS)", "moment", 0.3 * w * 9.0 / 2.0, reaction(&r, "OCC SEIS", 10)[4], TOL, "Statics: 0.3 w L²/2"));
        // 17. The occasional stress at the root: the sustained w L²/2Z plus the seismic 0.3 w L²/2Z
        //     (straight pipe, no pressure: i = 1).
        let z = crate::code::z(m.pipes[0].od, m.pipes[0].wall);
        let pt = r.geom.node_point[&10];
        let s = case(&r, "OCC SEIS").stresses.iter().find(|x| x.point == pt).map_or(f64::NAN, |x| x.stress);
        out.push(row(17, "Occasional stress at the root of the cantilever", "Sustained plus seismic stress", "stress", 1.3 * w * 9.0 / (2.0 * z), s, TOL, "Para. 302.3.6: SL + S_occ, beam theory M/Z"));
    }
    // 18. A spring hanger: anchored at 10, a 4 m riser to 20, 6 m along X to 30 with a hanger at 30
    //     carrying 20 kN (no weight), heated to 200 °C. Held rigidly, the hanger carries the load:
    //     H = 20 kN. Carrying H, the line is free to grow from its one anchor: 30 rises with the
    //     riser, Δ = 4 m × ε. The generic table's first fit: A-10 and A-11 fail (range, variation),
    //     B-10 (12 820–23 070 N over 50 mm: k = 205 000 N/m) fits, so C = H + k Δ. In operation the
    //     spring carries C − k Δ = H again.
    {
        let mut d = table("Hanger", &[(10, 20, 0.0, 4.0, 0.0), (20, 30, 6.0, 0.0, 0.0)]);
        let mut s = d.m.settings.clone();
        s.g = 0.0;
        d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
        let mut c = d.m.conds[0].clone();
        c.t[0] = K0 + 200.0;
        d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
        anchor(&mut d, 10);
        add(&mut d, Item::Support(Support { node: 30, kind: SupKind::Hanger { k: 0.0, cold: 0.0 }, ..Default::default() }));
        add(&mut d, Item::Load(Load { node: 30, f: [0.0, -20e3, 0.0], m: [0.0; 3], ..Default::default() }));
        let r = analyze(&d.m).unwrap();
        let h = &r.hangers[0];
        let (k, travel) = (205_000.0, 4.0 * STRAIN_200);
        let name = "A spring hanger on a heated riser and run";
        out.push(row(18, name, "Hot load H", "force", 20e3, h.hot, TOL, "Statics: the load at the held hanger"));
        out.push(row(18, name, "Travel Δ", "disp", travel, h.travel, TOL, "Free thermal growth of the 4 m riser"));
        out.push(row(18, name, "Spring rate chosen (B-10)", "tstiff", k, h.k, 0.0, "The table's first size that fits"));
        out.push(row(18, name, "Cold load C = H + k Δ", "force", 20e3 + k * travel, h.cold, TOL, "Hooke's law"));
        out.push(row(18, name, "Hanger load in operation (OPE1)", "force", 20e3, reaction(&r, "OPE1", 30)[1], TOL, "C − k Δ = H"));
    }
    // 19. A steam turbine with two 6" nozzles: the exhaust at node 10 (the origin) with 2 m of pipe
    //     along X carrying 1000 lb down at its end, and an inlet at node 30, 5 m along X, with 1 m of
    //     pipe along Z carrying 500 lb along X; no weight. By statics the exhaust carries F = 1000 lb
    //     and M = 2 m × 1000 lb, the inlet 500 lb and 1 m × 500 lb; resolved at the exhaust the inlet's
    //     force passes through it (it lies along the line between them), so Fc = (500, −1000, 0) lb and
    //     Mc = (0, 1 m × 500 lb, −2 m × 1000 lb). Dc = √(6² + 6²) = 8.485 in (under 9).
    {
        use crate::equip::{EquipKind, Equipment, FT_LBF, LBF};
        let mut d = table("Turbine", &[(10, 20, 2.0, 0.0, 0.0), (30, 40, 0.0, 0.0, 1.0)]);
        let mut s = d.m.settings.clone();
        s.g = 0.0;
        d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
        d.apply(Op::SetStart { num: 30, p: [5.0, 0.0, 0.0] }).unwrap();
        anchor(&mut d, 10);
        anchor(&mut d, 30);
        let z1 = d.apply(Op::Add { item: Item::Nozzle(crate::model::Nozzle { node: 10, name: "Exhaust".into(), ..Default::default() }), after: None }).unwrap()[0];
        let z2 = d.apply(Op::Add { item: Item::Nozzle(crate::model::Nozzle { node: 30, name: "Inlet".into(), ..Default::default() }), after: None }).unwrap()[0];
        add(&mut d, Item::Load(Load { node: 20, f: [0.0, -1000.0 * LBF, 0.0], m: [0.0; 3], ..Default::default() }));
        add(&mut d, Item::Load(Load { node: 40, f: [500.0 * LBF, 0.0, 0.0], m: [0.0; 3], ..Default::default() }));
        d.apply(Op::SetEquipment { list: vec![Equipment { name: "T-301".into(), kind: EquipKind::Turbine, nozzles: vec![z1, z2], reference: z1, ..Default::default() }] }).unwrap();
        let r = analyze(&d.m).unwrap();
        let eq = &case(&r, "OPE1").equipment[0];
        let row_of = |w: &str| eq.rows.iter().find(|x| x.what.starts_with(w)).map_or(f64::NAN, |x| x.value);
        let m1 = 2.0 * 1000.0 * LBF / FT_LBF;
        let mc = ((500.0 * LBF).powi(2) + (2.0 * 1000.0 * LBF).powi(2)).sqrt() / FT_LBF;
        let fc = (500.0f64 * 500.0 + 1000.0 * 1000.0).sqrt();
        let name = "A steam turbine's nozzle criteria";
        let src = "Statics, and the turbine criteria as published equations";
        out.push(row(19, name, "Exhaust: 3F + M (lb, ft·lb)", "ratio", 3.0 * 1000.0 + m1, row_of("Exhaust"), TOL, src));
        out.push(row(19, name, "All nozzles at the exhaust: 2Fc + Mc", "ratio", 2.0 * fc + mc, row_of("All nozzles at"), TOL, src));
        out.push(row(19, name, "All nozzles: Fy (lb)", "ratio", 1000.0, row_of("All nozzles: Fy"), TOL, src));
        let allow = eq.rows.iter().find(|x| x.what.starts_with("All nozzles at")).map_or(f64::NAN, |x| x.allow);
        out.push(row(19, name, "Combined allowable 250·Dc", "ratio", 250.0 * 72.0f64.sqrt(), allow, TOL, "Dc = √(6² + 6²) in"));
    }
    // 20. A flange pair's leakage check: anchored at 10, 1 m of 6" Sch 40 to 20, a flange pair 20–21
    //     of 0.2 m, then 2 m to 30 carrying 5 kN down and 8 kN pulling along X; 10 bar; no weight. By
    //     statics the flange at 20 carries M = 5 kN × 2.2 m and F = 8 kN, so with G = 0.2 m
    //     P_eq = 16 M / (π G³) + 4 F / (π G²), and the ratio to a 50 bar rating is (10 bar + P_eq) / 50 bar.
    {
        let mut d = table("Flange", &[(10, 20, 1.0, 0.0, 0.0), (20, 21, 0.2, 0.0, 0.0), (21, 30, 2.0, 0.0, 0.0)]);
        let mut s = d.m.settings.clone();
        s.g = 0.0;
        d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
        let mut c = d.m.conds[0].clone();
        c.p[0] = 1e6;
        d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
        let mut e = d.m.elements[1].clone();
        e.comp = Comp::Rigid { rigid: crate::model::RigidKind::Flange, mass: 0.0 };
        let fid = e.id;
        d.apply(Op::Set { item: Item::Element(e) }).unwrap();
        anchor(&mut d, 10);
        add(&mut d, Item::Load(Load { node: 30, f: [8e3, -5e3, 0.0], m: [0.0; 3], ..Default::default() }));
        d.apply(Op::SetFlanges { list: vec![crate::flange::Flange { element: fid, gasket: 0.2, rating: 5e6 }] }).unwrap();
        let r = analyze(&d.m).unwrap();
        let fc = &case(&r, "OPE1").flanges[0];
        let pi = std::f64::consts::PI;
        let peq = 16.0 * 5e3 * 2.2 / (pi * 0.008) + 4.0 * 8e3 / (pi * 0.04);
        let name = "A flange pair's leakage (equivalent pressure)";
        let src = "Statics, and P_eq = 16M/πG³ + 4F/πG² (Kellogg, 1956)";
        out.push(row(20, name, "Equivalent pressure", "stress", peq, fc.equivalent, TOL, src));
        out.push(row(20, name, "Ratio to the rating", "ratio", (1e6 + peq) / 5e6, fc.ratio, TOL, src));
    }
    // 21. A relief valve's thrust: anchored at 10, 2 m of 6" Sch 40 along X to 20, then 1 m up to
    //     the open end 30; no weight. 10 kg/s of steam (k 1.3, M 18.015 kg/kmol) at 200 °C leaves
    //     upwards through the pipe's bore A. At sound speed V = √(2kRT/((k+1)M)) and the exit pressure
    //     P = (ṁ/A)√(2RT/(k(k+1)M)), so F = 2 × (ṁ V + (P − 101 325 Pa) A) with a dynamic load
    //     factor of 2 (the exit is choked: P is above the surroundings), pushing down at 30; by statics the anchor carries F and the moment F × 2 m.
    {
        let mut d = table("Relief", &[(10, 20, 2.0, 0.0, 0.0), (20, 30, 0.0, 1.0, 0.0)]);
        let mut s = d.m.settings.clone();
        s.g = 0.0;
        d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
        anchor(&mut d, 10);
        let rv = crate::relief::Relief { name: "PSV-1".into(), node: 30, flow: 10.0, ..Default::default() };
        d.apply(Op::SetReliefs { list: vec![rv] }).unwrap();
        let r = analyze(&d.m).unwrap();
        let p = &d.m.pipes[0];
        let a = std::f64::consts::PI / 4.0 * (p.od - 2.0 * p.wall).powi(2);
        let (rr, t, k, mw) = (8_314.462_618, K0 + 200.0, 1.3, 18.015);
        let v = (2.0 * k * rr * t / ((k + 1.0) * mw)).sqrt();
        let pe = 10.0 / a * (2.0 * rr * t / (k * (k + 1.0) * mw)).sqrt();
        let f = 2.0 * (10.0 * v + (pe - 101_325.0) * a);
        let occ = case(&r, "OCC RV");
        let re = occ.reactions.iter().find(|x| x.node == 10).map_or([f64::NAN; 6], |x| x.f);
        let name = "A relief valve's thrust at an open end";
        let src = "Momentum and pressure thrust at a sonic exit (ideal gas), and statics";
        out.push(row(21, name, "Thrust with the load factor", "force", f, r.reliefs[0].thrust, TOL, src));
        out.push(row(21, name, "Anchor force (up)", "force", f, re[1], TOL, src));
        out.push(row(21, name, "Anchor moment", "moment", 2.0 * f, re[5].abs(), TOL, src));
    }
    // 22. A cantilever from a flexible nozzle: 2 m of 6" Sch 40 along X from a nozzle on a vertical
    //     vessel (anchored at 10), 10 kN down at 20, no weight. The moment P L turns the nozzle by
    //     P L / kθ (longitudinal bending), the shear moves it by P / ks, so the tip goes down by
    //     P L² / kθ + P / ks more than on a rigid anchor; the anchor's moment is P L by statics.
    {
        let (p, l, kt, ks) = (10e3, 2.0, 1e6, 1e7);
        let run = |flex: bool| {
            let mut d = table("Nozzle", &[(10, 20, l, 0.0, 0.0)]);
            let mut s = d.m.settings.clone();
            s.g = 0.0;
            d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
            anchor(&mut d, 10);
            let stiff = if flex { [0.0, ks, 0.0, kt, 0.0] } else { [0.0; 5] };
            add(&mut d, Item::Nozzle(crate::model::Nozzle { node: 10, name: "N1".into(), axis: [1.0, 0.0, 0.0], stiff, ..Default::default() }));
            add(&mut d, Item::Load(Load { node: 20, f: [0.0, -p, 0.0], m: [0.0; 3], ..Default::default() }));
            analyze(&d.m).unwrap()
        };
        let (rf, rr) = (run(true), run(false));
        let name = "A cantilever from a flexible nozzle";
        let src = "Beam theory with springs in series: P L²/kθ + P/ks, and statics";
        out.push(row(22, name, "Extra tip deflection", "disp", p * l * l / kt + p / ks, disp(&rr, "OPE1", 20)[1] - disp(&rf, "OPE1", 20)[1], TOL, src));
        out.push(row(22, name, "Nozzle rotation", "rot", p * l / kt, -disp(&rf, "OPE1", 10)[5], TOL, src));
        out.push(row(22, name, "Anchor moment", "moment", p * l, reaction(&rf, "OPE1", 10)[5], TOL, src));
    }
    // 23. An expansion joint 0.3 m long with the maker's rates ka 200 N/mm, k_lat 1000 N/mm,
    //     kθ 30 kN·m/rad. (a) Between two anchors, the far one moved 5 mm sideways: F = k_lat δ.
    //     (b) A cantilever joint with 100 N·m at its end turns by M / kθ. (c) 3 m of 6" Sch 40 each
    //     side between anchors at 10 bar, effective area 0.03 m², no weight: the thrust P A pushes
    //     the ends apart, the pipe (k1 = EA / 3 m) and the joint share it, so each anchor carries
    //     P A k1 / (k1 + 2 ka).
    {
        let bel = crate::model::Bellows { axial: 2e5, lateral: 1e6, angular: 3e4, torsion: 0.0, area: 0.03, tied: false, mass: 0.0 };
        let setup = |rows: &[(u32, u32, f64, f64, f64)], at: u32| {
            let mut d = table("Bellows", rows);
            let mut s = d.m.settings.clone();
            s.g = 0.0;
            d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
            let mut e = d.m.elements.iter().find(|e| e.to == at).unwrap().clone();
            e.comp = Comp::Bellows(bel);
            d.apply(Op::Set { item: Item::Element(e) }).unwrap();
            d
        };
        let name = "An expansion joint's spring rates and pressure thrust";
        let src = "The maker's rates by definition, and statics with springs in series";
        let mut d = setup(&[(10, 20, 0.3, 0.0, 0.0)], 20);
        anchor(&mut d, 10);
        add(&mut d, Item::Support(Support { node: 20, kind: SupKind::Anchor, disp: [[0.0, 0.005, 0.0, 0.0, 0.0, 0.0], [0.0; 6], [0.0; 6]], ..Default::default() }));
        let r = analyze(&d.m).unwrap();
        out.push(row(23, name, "Lateral force for a 5 mm offset", "force", 1e6 * 0.005, reaction(&r, "OPE1", 20)[1], TOL, src));
        let mut d = setup(&[(10, 20, 0.3, 0.0, 0.0)], 20);
        anchor(&mut d, 10);
        add(&mut d, Item::Load(Load { node: 20, f: [0.0; 3], m: [0.0, 0.0, 100.0], ..Default::default() }));
        let r = analyze(&d.m).unwrap();
        out.push(row(23, name, "Rotation under 100 N·m", "rot", 100.0 / 3e4, disp(&r, "OPE1", 20)[5], TOL, src));
        let mut d = setup(&[(10, 20, 3.0, 0.0, 0.0), (20, 21, 0.3, 0.0, 0.0), (21, 30, 3.0, 0.0, 0.0)], 21);
        let mut c = d.m.conds[0].clone();
        c.p[0] = 1e6;
        c.t = [d.m.settings.ambient; 3];
        d.apply(Op::Set { item: Item::Cond(c) }).unwrap();
        anchor(&mut d, 10);
        anchor(&mut d, 30);
        let r = analyze(&d.m).unwrap();
        let p = &d.m.pipes[0];
        let e = crate::library::interp(&d.m.materials[0].e, d.m.settings.ambient);
        let k1 = e * std::f64::consts::PI / 4.0 * (p.od.powi(2) - (p.od - 2.0 * p.wall).powi(2)) / 3.0;
        let pa = 1e6 * 0.03;
        out.push(row(23, name, "Anchor force from the pressure thrust", "force", pa * k1 / (k1 + 2.0 * 2e5), -reaction(&r, "SUS", 30)[0], TOL, src));
    }
    // 24. Wind that changes with height: a 10 m vertical cantilever of 6" Sch 40 in 1 m elements,
    //     anchored at the ground, wind along X from a profile rising from 0 at 0 m to 1 kPa at 10 m,
    //     Cd 0.7, no weight. Each element carries its average pressure (exact for a straight-line
    //     profile), so the base shear is the area under the profile, 0.7 D × 5000 N/m, and the base
    //     moment the sum of each element's load times its mid-height. In both directions, the
    //     reversed case gives the same shear the other way.
    {
        let rows: Vec<(u32, u32, f64, f64, f64)> = (0..10).map(|i| (10 * (i + 1), 10 * (i + 2), 0.0, 1.0, 0.0)).collect();
        let mut d = table("Wind profile", &rows);
        let mut s = d.m.settings.clone();
        s.g = 0.0;
        s.wind_profile = vec![[0.0, 0.0], [10.0, 1000.0]];
        s.wind_cd = 0.7;
        s.wind_dir = [1.0, 0.0, 0.0];
        s.occ_both = true;
        d.apply(Op::SetSettings { s, units: "SI".into() }).unwrap();
        anchor(&mut d, 10);
        let r = analyze(&d.m).unwrap();
        let cd_d = 0.7 * d.m.pipes[0].od;
        let shear = cd_d * 1000.0 * 10.0 / 2.0;
        let moment: f64 = (0..10).map(|i| cd_d * 100.0 * (i as f64 + 0.5) * (i as f64 + 0.5)).sum();
        let name = "Wind that changes with height, both ways";
        let src = "Statics: the area under the profile and its moment, element by element";
        out.push(row(24, name, "Base shear (OCC WIN)", "force", shear, -reaction(&r, "OCC WIN", 10)[0], TOL, src));
        out.push(row(24, name, "Base moment (OCC WIN)", "moment", moment, reaction(&r, "OCC WIN", 10)[5], TOL, src));
        out.push(row(24, name, "Base shear the other way (OCC −WIN)", "force", -shear, -reaction(&r, "OCC −WIN", 10)[0], TOL, src));
    }
    // 25. The vessel example V-101 by hand: 1.8 m inside diameter, 16 bar, S 138 MPa, 3 mm corrosion.
    //     The shell (E 0.85) at the bottom tangent line carries 16 bar plus 850 kg/m³ × g × 1.8 m:
    //     t = P R / (S E − 0.6 P) with R = 903 mm; the bottom head (E 1) at its pole, 0.45 m lower:
    //     t = P D / (2 S E − 0.2 P) with D = 1806 mm. The MAWP at the top is the shell's
    //     S E t / (R + 0.6 t) with t = 13 mm, less its static head; the test pressure 1.3 × MAWP.
    {
        let m = crate::examples::get("separator").unwrap();
        let r = crate::vessel::check(&m.vessels[0]).unwrap();
        let g = 9.806_65;
        let (s, se) = (138e6, 138e6 * 0.85);
        let p_sh = 16e5 + 850.0 * g * 1.8;
        let t_sh = p_sh * 0.903 / (se - 0.6 * p_sh);
        let p_hd = 16e5 + 850.0 * g * 2.25;
        let t_hd = p_hd * 1.806 / (2.0 * s - 0.2 * p_hd);
        let mawp = se * 0.013 / (0.903 + 0.6 * 0.013) - 850.0 * g * 1.8;
        let name = "A vertical vessel's shell and heads (V-101)";
        let src = "Thin-shell membrane equations with the liquid's static head";
        out.push(row(25, name, "Shell thickness for pressure", "len", t_sh, r.parts[1].t_req, TOL, src));
        out.push(row(25, name, "Bottom head thickness for pressure", "len", t_hd, r.parts[0].t_req, TOL, src));
        out.push(row(25, name, "MAWP at the top (shell governs)", "press", mawp, r.mawp, TOL, src));
        out.push(row(25, name, "Hydrotest pressure", "press", 1.3 * mawp, r.test, TOL, src));
        let vol = std::f64::consts::PI * 0.9f64.powi(2) * 6.0 + 2.0 * std::f64::consts::PI * 1.8f64.powi(3) / 24.0;
        out.push(row(25, name, "Volume", "ratio", vol, r.volume, 2e-3, "π R² L + 2 × π D³ / 24"));
    }
    // 26. V-101's 8" Sch 80 inlet with a 380 × 16 mm pad, by area replacement: d = OD − 2 t + 2 c,
    //     tr the shell's need with E = 1 at its pressure, A = d tr; available A1 = max(d, 2 (t + tn))
    //     (t − tr), A2 = 5 (tn − trn) min(t, tn), A41 = A42 = 10² mm², A5 = (Dp − d − 2 tn) te; the
    //     neck's minimum max(trn + c, min(tr + c, 0.875 t_std + c)). tn = 0.875 × 12.7 − 3 mm.
    {
        let m = crate::examples::get("separator").unwrap();
        let r = crate::vessel::check(&m.vessels[0]).unwrap();
        let z = &r.nozzles[0];
        let g = 9.806_65;
        let p = 16e5 + 850.0 * g * 1.8;
        let (od, tw): (f64, f64) = (8.625 * 0.0254, 0.500 * 0.0254);
        let d = od - 2.0 * tw + 0.006;
        let tr = p * (0.9 + 0.003) / (138e6 - 0.6 * p);
        let (t, tn): (f64, f64) = (0.013, 0.875 * tw - 0.003);
        let trn = p * d / 2.0 / (138e6 - 0.6 * p);
        let avail = d.max(2.0 * (t + tn)) * (t - tr) + 5.0 * (tn - trn) * t.min(tn) + 2.0 * 0.01f64.powi(2) + (0.38 - d - 2.0 * tn) * 0.016;
        let std = 0.875 * 0.322 * 0.0254;
        let name = "A vessel nozzle's reinforcement (V-101, N1)";
        let src = "Area replacement within the reinforcement limits";
        out.push(row(26, name, "Area required d tr", "area", d * tr, z.a_req, TOL, src));
        out.push(row(26, name, "Area available", "area", avail, z.a_avail, TOL, src));
        out.push(row(26, name, "Neck's minimum thickness", "len", (trn + 0.003).max((tr + 0.003).min(std + 0.003)), z.t_neck_req, TOL, src));
    }
    // 27. D-201 on two 120° saddles by Zick's method, with the published table constants for 120°
    //     (K3 0.880, K6 0.0528, a quarter of it for A/R ≤ 0.5): Q the half weight full of water;
    //     mid-span moment M2 = Q L/4 [(1 + 2 (R² − H²)/L²)/(1 + 4H/3L) − 4A/L]; shear K3 Q/(R t);
    //     horn −Q/(4 t (b + 1.56 √(R t))) − 12 K6 Q R/(L t²) (here L = 9 m < 8R = 9.64 m).
    {
        let m = crate::examples::get("drum").unwrap();
        let r = crate::vessel::check(&m.vessels[0]).unwrap();
        let sd = r.saddles.as_ref().unwrap();
        let q = r.test_weight.max(r.operating) * 9.806_65 / 2.0;
        let t: f64 = 0.014 - 0.003;
        let rm = (2.4 + t) / 2.0 + 0.003;
        let (l, h, a) = (9.0, 0.6, 0.6);
        let m2 = q * l / 4.0 * ((1.0 + 2.0 * (rm * rm - h * h) / (l * l)) / (1.0 + 4.0 * h / (3.0 * l)) - 4.0 * a / l);
        assert!(l < 8.0 * rm);
        let horn = -q / (4.0 * t * (0.3 + 1.56 * (rm * t).sqrt())) - 12.0 * (0.0528 / 4.0) * q * rm / (l * t * t);
        let find = |w: &str| sd.rows.iter().find(|x| x.what.starts_with(w)).map_or(f64::NAN, |x| x.stress);
        let name = "A horizontal vessel on two saddles (D-201)";
        let src = "Zick's method (1951), the 120° table constants";
        out.push(row(27, name, "Mid-span moment M2", "moment", m2, sd.m2, TOL, "Statics of the vessel as a beam on two saddles"));
        out.push(row(27, name, "Tangential shear (stiffened by the head)", "stress", 0.880 * q / (rm * t), find("Tangential shear in the shell"), 2e-3, src));
        out.push(row(27, name, "Circumferential stress at the horn", "stress", horn, find("Circumferential at the saddle horn"), 2e-3, src));
    }
    // 28. V-101 on its skirt: 1 kPa wind × Cd 0.7 × 1.2 on the projected outline (the skirt and the
    //     shell at their outside diameter 1832 mm over 2.5 m + 6 m, the top head π a h / 2 + 2 t h,
    //     the bottom head inside the skirt); 0.15 g on the operating weight and the skirt's metal;
    //     the anchor bolts 4 M / (N BC) − W / N on their root area; the base ring l √(3 σ / S).
    {
        let m = crate::examples::get("separator").unwrap();
        let r = crate::vessel::check_in(&m.vessels[0], &m.settings).unwrap();
        let su = r.support.as_ref().unwrap();
        let g = 9.806_65;
        let pi = std::f64::consts::PI;
        let od = 1.8 + 2.0 * 0.016;
        let area = od * (2.5 + 6.0) + pi * 0.9 * 0.45 / 2.0 + 2.0 * 0.016 * 0.45;
        let skirt = pi * (od - 0.012) * 0.012 * 2.5 * 7850.0;
        let mo = su.moment_wind.max(su.moment_seis);
        let bolt = (4.0 * mo / (12.0 * 2.05) - r.empty * g / 12.0) / (pi / 4.0 * 0.0262f64.powi(2));
        let ring = (2.25 - od) / 2.0 * (3.0 * su.bearing / 138e6).sqrt();
        let name = "A vertical vessel on a skirt (V-101)";
        out.push(row(28, name, "Wind base shear", "force", 1000.0 * 0.7 * 1.2 * area, su.shear_wind, 1e-3, "Drag on the projected outline"));
        out.push(row(28, name, "Seismic base shear", "force", 0.15 * g * (r.operating + skirt), su.shear_seis, 1e-3, "Newton's second law: c m g"));
        out.push(row(28, name, "Anchor bolt stress", "stress", bolt, su.bolt_stress, TOL, "Statics: 4M/(N BC) − W/N over the root area"));
        out.push(row(28, name, "Base ring thickness", "len", ring, su.ring_t_req, TOL, "A cantilever strip: l √(3 σ / S)"));
    }
    // 29. V-101's manway cover, a bolted blind flange at 16 bar: gasket 584/540 mm, m 2, y 11 MPa,
    //     24 bolts of 599 mm² root area, bolt circle 686 mm, S 138 MPa (flange), 172 MPa (bolts).
    //     b = 2.52 √11 mm, G = 584 − 2b; Wm1 = π/4 G² P + 2b π G m P, Wm2 = π b G y; the cover's
    //     thickness in operation G √(0.3 P/S + 1.9 Wm1 hG/(S G³)) and at seating with
    //     W = (Am + Ab) Sa / 2 and no pressure.
    {
        let m = crate::examples::get("separator").unwrap();
        let r = crate::vessel::check_in(&m.vessels[0], &m.settings).unwrap();
        let f = &r.flanges[0];
        let pi = std::f64::consts::PI;
        let b = 2.52e-3 * 11f64.sqrt();
        let g = 0.584 - 2.0 * b;
        let p = 16e5;
        let wm1 = pi / 4.0 * g * g * p + 2.0 * b * pi * g * 2.0 * p;
        let wm2 = pi * b * g * 11e6;
        let am = (wm1 / 172e6).max(wm2 / 172e6);
        let w = (am + 24.0 * 599e-6) * 172e6 / 2.0;
        let hg = (0.686 - g) / 2.0;
        let name = "A bolted cover (V-101, M1)";
        let src = "The bolted flange method: gasket loads, bolt loads, a flat cover in bending";
        out.push(row(29, name, "Bolt load in operation Wm1", "force", wm1, f.wm1, TOL, src));
        out.push(row(29, name, "Cover thickness, operating", "len", g * (0.3 * p / 138e6 + 1.9 * wm1 * hg / (138e6 * g.powi(3))).sqrt(), f.op, TOL, src));
        out.push(row(29, name, "Cover thickness, gasket seating", "len", g * (1.9 * w * hg / (138e6 * g.powi(3))).sqrt(), f.seat, TOL, src));
    }
    // 30. V-101's two lifting lugs: the empty weight × 1.5 over 2 lugs at 30° from vertical,
    //     F = 1.5 W / 2 / cos 30°; across the hole F / ((2 r − d) t); at the base
    //     F cos φ / (w t) + 6 F sin φ h / (t w²) (30 × 250 mm lug, hole 120 mm up, r 75, d 55 mm).
    {
        let m = crate::examples::get("separator").unwrap();
        let r = crate::vessel::check_in(&m.vessels[0], &m.settings).unwrap();
        let f = 1.5 * r.empty * 9.806_65 / 2.0 / 30f64.to_radians().cos();
        let (fv, fh) = (f * 30f64.to_radians().cos(), f * 30f64.to_radians().sin());
        let find = |w: &str| r.lifting.iter().find(|x| x.what.starts_with(w)).map_or(f64::NAN, |x| x.value);
        let name = "A vessel's lifting lugs (V-101)";
        let src = "Statics of a plate lug";
        out.push(row(30, name, "Tension across the hole", "stress", f / ((0.15 - 0.055) * 0.03), find("Lug: tension"), TOL, src));
        out.push(row(30, name, "Tension and bending at the base", "stress", fv / (0.25 * 0.03) + 6.0 * fh * 0.12 / (0.03 * 0.0625), find("Lug: tension and bending"), TOL, src));
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
        for b in 1..=30 {
            assert!(rows.iter().any(|r| r.bench == b), "benchmark {b} is missing");
        }
    }
}
