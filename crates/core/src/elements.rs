//! The 3D pipe element, built from its exact flexibility.
//!
//! Every segment (straight pipe, an arc of a bend, a reducer, a rigid part) is treated the same
//! way: with its start held, a force and moment P at its end produce resultants f = F and
//! m = M + (p_end − p(s)) × F at each section along the centreline, and the end moves by
//! δ = (∫ Bᵀ C B ds) P, with C the section's compliance: 1/EA along the pipe and 1/(κGA) across
//! it for the forces (shear deformation, κ = 0.5 for a thin tube), 1/GJ for torsion and k/EI for
//! bending (k is the bend's flexibility factor, 1 for straight pipe). The 6×6 flexibility is
//! inverted to the stiffness of the end, and moved to the start by equilibrium, giving the full
//! 12×12 element. Distributed loads (weight) and free thermal expansion give the equivalent
//! nodal loads the same way. For straight pipe of constant section the integrals are exact
//! (4-point Gauss on polynomials); arcs and reducers use 16 sub-intervals.

use crate::mat::*;
use serde::Serialize;

/// Shear coefficient of a thin-walled circular tube.
pub const KAPPA: f64 = 0.5;
/// A rigid element is this many times stiffer than the pipe it is on.
pub const RIGID_FACTOR: f64 = 1000.0;

/// Section properties of a pipe (m, m², m⁴, m³).
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Section {
    pub od: f64,
    pub wall: f64,
    pub a: f64,
    pub i: f64,
    pub z: f64,
}

impl Section {
    pub fn new(od: f64, wall: f64) -> Section {
        let id = (od - 2.0 * wall).max(0.0);
        let a = std::f64::consts::PI / 4.0 * (od * od - id * id);
        let i = std::f64::consts::PI / 64.0 * (od.powi(4) - id.powi(4));
        Section { od, wall, a, i, z: i / (od / 2.0) }
    }
    pub fn id(&self) -> f64 {
        (self.od - 2.0 * self.wall).max(0.0)
    }
}

/// The centreline of a segment by arc length.
#[derive(Clone, Copy, Debug)]
pub enum Line {
    Straight { a: V, b: V },
    Arc { centre: V, ra: V, tan0: V, r: f64, angle: f64 },
}

impl Line {
    pub fn length(&self) -> f64 {
        match *self {
            Line::Straight { a, b } => dist(a, b),
            Line::Arc { r, angle, .. } => r * angle,
        }
    }
    /// Position and direction at arc length s.
    pub fn at(&self, s: f64) -> (V, V) {
        match *self {
            Line::Straight { a, b } => {
                let t = unit(sub(b, a));
                (add(a, scale(t, s)), t)
            }
            Line::Arc { centre, ra, tan0, r, .. } => {
                let phi = s / r;
                let rp = add(scale(ra, phi.cos()), scale(tan0, phi.sin()));
                let t = add(scale(ra, -phi.sin()), scale(tan0, phi.cos()));
                (add(centre, scale(rp, r)), t)
            }
        }
    }
    /// ∫ p ds from s0 to s1.
    pub fn pint(&self, s0: f64, s1: f64) -> V {
        match *self {
            Line::Straight { a, b } => {
                let t = unit(sub(b, a));
                add(scale(a, s1 - s0), scale(t, (s1 * s1 - s0 * s0) / 2.0))
            }
            Line::Arc { centre, ra, tan0, r, .. } => {
                let (p0, p1) = (s0 / r, s1 / r);
                add(scale(centre, s1 - s0), add(scale(ra, r * r * (p1.sin() - p0.sin())), scale(tan0, r * r * (p0.cos() - p1.cos()))))
            }
        }
    }
    pub fn end(&self) -> V {
        self.at(self.length()).0
    }
    pub fn start(&self) -> V {
        self.at(0.0).0
    }
}

/// What a segment is made of, for its stiffness: the section at fraction s of its length (a
/// reducer changes along it), E, G, the bend's flexibility factor, and whether it is rigid.
pub struct Props<'a> {
    pub sec: &'a dyn Fn(f64) -> Section,
    pub e: f64,
    pub g: f64,
    pub k: f64,
    pub rigid: bool,
}

/// The compliance C at a section, applied to a resultant [f; m] with the pipe direction t.
fn comply(p: &Props, s: f64, t: V, fm: &[f64; 6]) -> [f64; 6] {
    let sec = (p.sec)(s);
    let r = if p.rigid { 1.0 / RIGID_FACTOR } else { 1.0 };
    let (ea, gas, gj, ei) = (p.e * sec.a, p.g * sec.a * KAPPA, p.g * 2.0 * sec.i, p.e * sec.i / p.k);
    let f = [fm[0], fm[1], fm[2]];
    let m = [fm[3], fm[4], fm[5]];
    let (fa, ma) = (dot(f, t), dot(m, t));
    let fs = sub(f, scale(t, fa));
    let ms = sub(m, scale(t, ma));
    let df = add(scale(t, r * fa / ea), scale(fs, r / gas));
    let dm = add(scale(t, r * ma / gj), scale(ms, r / ei));
    [df[0], df[1], df[2], dm[0], dm[1], dm[2]]
}

/// The pieces of one element: its end stiffness, the move from end to start, and its line.
#[derive(Clone, Debug)]
pub struct Elem {
    pub line: Line,
    /// Stiffness of the end relative to the start (6×6).
    pub k: M6,
    /// Hᵀ: a force at the end moved to the start (F, M + r × F).
    pub ht: M6,
    pub subs: usize,
}

impl Elem {
    pub fn new(line: Line, p: &Props) -> Result<Elem, String> {
        let l = line.length();
        let subs = if matches!(line, Line::Straight { .. }) && (p.sec)(0.0).od == (p.sec)(1.0).od && (p.sec)(0.0).wall == (p.sec)(1.0).wall { 1 } else { 16 };
        let pe = line.end();
        let mut f = zero6();
        for k in 0..subs {
            for (x, w) in GAUSS4 {
                let s = (k as f64 + x) / subs as f64 * l;
                let (ps, t) = line.at(s);
                let b = shift(sub(pe, ps));
                // F += Bᵀ C B ds, column by column.
                for j in 0..6 {
                    let col = [b[0][j], b[1][j], b[2][j], b[3][j], b[4][j], b[5][j]];
                    let c = comply(p, s / l, t, &col);
                    let bt = mv6(&t6(&b), &c);
                    for i in 0..6 {
                        f[i][j] += bt[i] * w * l / subs as f64;
                    }
                }
            }
        }
        let k = inv6(&f).ok_or("An element has no stiffness: check its pipe size and material.")?;
        Ok(Elem { line, k, ht: shift(sub(pe, line.start())), subs })
    }

    /// The 12×12 stiffness [[Hᵀ K H, −Hᵀ K], [−K H, K]].
    pub fn stiffness(&self) -> [[f64; 12]; 12] {
        let h = t6(&self.ht);
        let htk = mul6(&self.ht, &self.k);
        let aa = mul6(&htk, &h);
        let kh = mul6(&self.k, &h);
        let mut out = [[0.0; 12]; 12];
        for i in 0..6 {
            for j in 0..6 {
                out[i][j] = aa[i][j];
                out[i][j + 6] = -htk[i][j];
                out[i + 6][j] = -kh[i][j];
                out[i + 6][j + 6] = self.k[i][j];
            }
        }
        out
    }

    /// The resultant at s from a uniform load q (N/m) on the part of the element beyond s:
    /// force q (L − s), moment ∫ (p(s') − p(s)) × q ds'.
    fn load_beyond(&self, s: f64, q: V) -> [f64; 6] {
        let l = self.line.length();
        let (ps, _) = self.line.at(s);
        let arm = sub(self.line.pint(s, l), scale(ps, l - s));
        let m = cross(arm, q);
        let f = scale(q, l - s);
        [f[0], f[1], f[2], m[0], m[1], m[2]]
    }

    /// The end's movement with the start held, from a uniform load q.
    fn delta_q(&self, q: V, p: &Props) -> [f64; 6] {
        let l = self.line.length();
        let pe = self.line.end();
        let mut d = [0.0; 6];
        for k in 0..self.subs {
            for (x, w) in GAUSS4 {
                let s = (k as f64 + x) / self.subs as f64 * l;
                let (ps, t) = self.line.at(s);
                let b = shift(sub(pe, ps));
                let c = comply(p, s / l, t, &self.load_beyond(s, q));
                let bt = mv6(&t6(&b), &c);
                for i in 0..6 {
                    d[i] += bt[i] * w * l / self.subs as f64;
                }
            }
        }
        d
    }

    /// Fixed-end forces of a uniform load q and a free thermal elongation `strain` (ΔL/L), as the
    /// loads they put on the nodes: (at start, at end). Also returns the end's fixed reaction, which
    /// the end forces add back.
    pub fn loads(&self, q: V, strain: f64, p: &Props) -> ([f64; 6], [f64; 6], [f64; 6]) {
        let mut dfix = [0.0; 6];
        if q != [0.0; 3] {
            dfix = self.delta_q(q, p);
        }
        let chord = sub(self.line.end(), self.line.start());
        let dt = scale(chord, strain);
        for i in 0..3 {
            dfix[i] += dt[i];
        }
        // The reaction at the end that keeps it from moving: −K δ.
        let rb = mv6(&self.k, &dfix).map(|v| -v);
        // The start's reaction by equilibrium of the whole element.
        let l = self.line.length();
        let tot = scale(q, l);
        let ps = self.line.start();
        let mq = cross(sub(self.line.pint(0.0, l), scale(ps, l)), q);
        let rbs = mv6(&self.ht, &rb);
        let ra = [-(tot[0] + rbs[0]), -(tot[1] + rbs[1]), -(tot[2] + rbs[2]), -(mq[0] + rbs[3]), -(mq[1] + rbs[4]), -(mq[2] + rbs[5])];
        (ra.map(|v| -v), rb.map(|v| -v), rb)
    }

    /// The force and moment on the element at its end from the end node, given the movement of the
    /// end relative to the start (δ = d_end − H d_start), the thermal strain and the fixed-end
    /// reaction of the distributed load.
    pub fn end_force(&self, d_start: &[f64; 6], d_end: &[f64; 6], strain: f64, rb_q: &[f64; 6]) -> [f64; 6] {
        let hd = mv6(&t6(&self.ht), d_start);
        let chord = sub(self.line.end(), self.line.start());
        let mut delta = [0.0; 6];
        for i in 0..6 {
            delta[i] = d_end[i] - hd[i];
        }
        for i in 0..3 {
            delta[i] -= chord[i] * strain;
        }
        let kd = mv6(&self.k, &delta);
        let mut out = [0.0; 6];
        for i in 0..6 {
            out[i] = kd[i] + rb_q[i];
        }
        out
    }

    /// The internal resultant at arc length s: the force and moment (about the section) that the
    /// part towards the start exerts on the part towards the end.
    pub fn section(&self, s: f64, end_force: &[f64; 6], q: V) -> ([f64; 3], [f64; 3], V) {
        let (ps, t) = self.line.at(s);
        let pe = self.line.end();
        let lb = self.load_beyond(s, q);
        let f = [end_force[0] + lb[0], end_force[1] + lb[1], end_force[2] + lb[2]];
        let r = sub(pe, ps);
        let rf = cross(r, [end_force[0], end_force[1], end_force[2]]);
        let m = [end_force[3] + rf[0] + lb[3], end_force[4] + rf[1] + lb[4], end_force[5] + rf[2] + lb[5]];
        (scale(f, -1.0), scale(m, -1.0), t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steel(sec: &dyn Fn(f64) -> Section, k: f64) -> Props<'_> {
        Props { sec, e: 200e9, g: 200e9 / 2.6, k, rigid: false }
    }

    #[test]
    fn a_straight_pipe_is_a_timoshenko_beam() {
        let s = Section::new(0.168_275, 0.007_112);
        let f = |_: f64| s;
        let p = steel(&f, 1.0);
        let l = 4.0;
        let e = Elem::new(Line::Straight { a: [0.0; 3], b: [l, 0.0, 0.0] }, &p).unwrap();
        // Tip force along Y: δ = PL³/3EI + PL/κGA; rotation PL²/2EI.
        let fl = crate::mat::inv6(&e.k).unwrap();
        let want = l.powi(3) / (3.0 * p.e * s.i) + l / (KAPPA * p.g * s.a);
        assert!((fl[1][1] - want).abs() / want < 1e-12, "{} {want}", fl[1][1]);
        assert!((fl[5][1] - l * l / (2.0 * p.e * s.i)).abs() / fl[5][1] < 1e-12);
        assert!((fl[0][0] - l / (p.e * s.a)).abs() * p.e * s.a / l < 1e-12);
        // The 12×12 stiffness moves rigidly without forces.
        let k = e.stiffness();
        let mut d = [0.0; 12];
        // A rotation about Z of 1e-3 at the start and end: uy at the end = θ L.
        d[5] = 1e-3;
        d[11] = 1e-3;
        d[7] = 1e-3 * l;
        for i in 0..12 {
            let f: f64 = (0..12).map(|j| k[i][j] * d[j]).sum();
            assert!(f.abs() < 1e-3, "row {i}: {f}");
        }
        // Uniform load w down, ends fixed: end moments wL²/12, end forces wL/2.
        let w = 1000.0;
        let (pa, pb, _) = e.loads([0.0, -w, 0.0], 0.0, &p);
        assert!((pa[1] + w * l / 2.0).abs() < 1e-6 && (pb[1] + w * l / 2.0).abs() < 1e-6, "{pa:?} {pb:?}");
        assert!((pa[5] + w * l * l / 12.0).abs() < 1e-6 && (pb[5] - w * l * l / 12.0).abs() < 1e-6, "{pa:?} {pb:?}");
    }

    #[test]
    fn an_arc_follows_its_closed_form() {
        // A 90° arc, start held, a moment about the bend's normal at the end: θ = k M R (π/2) / EI.
        let s = Section::new(0.168_275, 0.007_112);
        let f = |_: f64| s;
        let k = 2.7;
        let p = steel(&f, k);
        let r = 0.2286;
        let e = Elem::new(Line::Arc { centre: [0.0, r, 0.0], ra: [0.0, -1.0, 0.0], tan0: [1.0, 0.0, 0.0], r, angle: std::f64::consts::FRAC_PI_2 }, &p).unwrap();
        let fl = crate::mat::inv6(&e.k).unwrap();
        let want = k * r * std::f64::consts::FRAC_PI_2 / (p.e * s.i);
        assert!((fl[5][5] - want).abs() / want < 1e-10, "{} {want}", fl[5][5]);
        let end = e.line.end();
        assert!((end[0] - r).abs() < 1e-12 && (end[1] - r).abs() < 1e-12);
        let pi = e.line.pint(0.0, e.line.length());
        // ∫ p ds over the quarter arc: centroid of the arc times its length.
        let c = 2.0 * r / std::f64::consts::PI;
        assert!((pi[0] - c * r * std::f64::consts::FRAC_PI_2).abs() < 1e-12, "{pi:?}");
    }
}
