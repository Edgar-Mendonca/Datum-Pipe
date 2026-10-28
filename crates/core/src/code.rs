//! The code checks, calculated using the published equations of the process piping code
//! (ASME B31.3). Each function names the equation it follows; no tables or text of the code are
//! reproduced. Allowable stresses come from the user's material tables.
//!
//! Stress intensification and flexibility factors for bends and tees are the textbook
//! expressions in the flexibility characteristic h, as published in the code's appendix on
//! flexibility and stress intensification factors.

use serde::Serialize;

/// Section modulus and areas of a pipe of outside diameter `od` and wall `t` (m³, m²).
pub fn z(od: f64, t: f64) -> f64 {
    let id = (od - 2.0 * t).max(0.0);
    std::f64::consts::PI * (od.powi(4) - id.powi(4)) / (32.0 * od)
}
pub fn area(od: f64, t: f64) -> f64 {
    let id = (od - 2.0 * t).max(0.0);
    std::f64::consts::PI / 4.0 * (od * od - id * id)
}

/// Pressure design thickness of straight pipe under internal pressure (para. 304.1.2, eq. 3a):
/// t = P D / (2 (S E W + P Y)).
pub fn pressure_thickness(p: f64, d: f64, s: f64, e: f64, w: f64, y: f64) -> f64 {
    if p <= 0.0 { 0.0 } else { p * d / (2.0 * (s * e * w + p * y)) }
}

/// The hoop stress the same equation gives for an available thickness t:
/// S = P (D − 2 Y t) / (2 E W t).
pub fn hoop_stress(p: f64, d: f64, t: f64, e: f64, w: f64, y: f64) -> f64 {
    if t <= 0.0 { f64::INFINITY } else { p * (d - 2.0 * y * t) / (2.0 * e * w * t) }
}

/// The minimum wall check of one element in one operating case.
#[derive(Clone, Debug, Serialize)]
pub struct Wall {
    pub element: u32,
    pub case: u8,
    pub p: f64,
    pub t: f64,
    pub s: f64,
    /// Pressure design thickness, plus corrosion (tm), and the nominal wall that gives it after the
    /// mill under-tolerance.
    pub t_design: f64,
    pub t_min: f64,
    pub t_nominal_needed: f64,
    pub t_nominal: f64,
    pub hoop: f64,
    pub ratio: f64,
    pub pass: bool,
}

pub fn wall(element: u32, case: u8, p: f64, t_metal: f64, s: f64, od: f64, wall: f64, c: f64, mill: f64, e: f64, w: f64, y: f64) -> Wall {
    let td = pressure_thickness(p, od, s, e, w, y);
    let tm = td + c;
    let need = tm / (1.0 - mill);
    let avail = wall * (1.0 - mill) - c;
    let hoop = if p > 0.0 { hoop_stress(p, od, avail, e, w, y) } else { 0.0 };
    let ratio = if wall > 0.0 { need / wall } else { f64::INFINITY };
    Wall { element, case, p, t: t_metal, s, t_design: td, t_min: tm, t_nominal_needed: need, t_nominal: wall, hoop, ratio, pass: ratio <= 1.0 + 1e-12 }
}

/// Stress intensification and flexibility at a point: in-plane and out-of-plane SIFs and the
/// flexibility factor (bends only).
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Sif {
    pub ii: f64,
    pub io: f64,
    pub k: f64,
    pub h: f64,
}

impl Sif {
    pub const ONE: Sif = Sif { ii: 1.0, io: 1.0, k: 1.0, h: 0.0 };
}

/// A bend (welding elbow or pipe bend): h = T R / r², k = 1.65 / h, ii = 0.9 / h^(2/3),
/// io = 0.75 / h^(2/3), each at least 1. With pressure P and modulus Ec, k is divided by
/// 1 + 6 (P/Ec)(r/T)^(7/3)(R/r)^(1/3) and the SIFs by 1 + 3.25 (P/Ec)(r/T)^(5/2)(R/r)^(2/3).
pub fn bend(t: f64, od: f64, rb: f64, p: f64, ec: f64) -> Sif {
    let r = (od - t) / 2.0;
    let h = t * rb / (r * r);
    let (mut k, mut ii, mut io) = (1.65 / h, 0.9 / h.powf(2.0 / 3.0), 0.75 / h.powf(2.0 / 3.0));
    if p > 0.0 && ec > 0.0 {
        k /= 1.0 + 6.0 * (p / ec) * (r / t).powf(7.0 / 3.0) * (rb / r).powf(1.0 / 3.0);
        let c = 1.0 + 3.25 * (p / ec) * (r / t).powf(2.5) * (rb / r).powf(2.0 / 3.0);
        ii /= c;
        io /= c;
    }
    Sif { ii: ii.max(1.0), io: io.max(1.0), k: k.max(1.0), h }
}

/// A welding tee (ASME B16.9): h = 3.1 T / r; a fabricated branch without a pad: h = T / r; with a
/// pad of thickness Tr: h = (T + Tr/2)^(5/2) / (T^(3/2) r). Then io = 0.9 / h^(2/3) and
/// ii = 0.75 io + 0.25, each at least 1.
pub fn tee(kind: crate::model::TeeKind, t: f64, od: f64, pad: f64) -> Sif {
    use crate::model::TeeKind::*;
    let r = (od - t) / 2.0;
    let h = match kind {
        Welding => 3.1 * t / r,
        Fabricated => t / r,
        Reinforced => (t + pad.min(1.5 * t) / 2.0).powf(2.5) / (t.powf(1.5) * r),
    };
    let io = 0.9 / h.powf(2.0 / 3.0);
    Sif { ii: (0.75 * io + 0.25).max(1.0), io: io.max(1.0), k: 1.0, h }
}

/// A concentric reducer: i = 0.5 + 0.01 α (D2 / t2)^(1/2), at most 2.0, with α the cone angle in
/// degrees and D2, t2 the small end's diameter and wall.
pub fn reducer(alpha_deg: f64, d2: f64, t2: f64) -> Sif {
    let i = (0.5 + 0.01 * alpha_deg * (d2 / t2).sqrt()).clamp(1.0, 2.0);
    Sif { ii: i, io: i, k: 1.0, h: 0.0 }
}

/// The allowable stress range factor f = 6.0 N^(−0.2), at most 1.0 and at least 0.15 (eq. 1c).
pub fn range_factor(cycles: f64) -> f64 {
    (6.0 * cycles.max(1.0).powf(-0.2)).clamp(0.15, 1.0)
}

/// The allowable displacement stress range (eq. 1a): SA = f (1.25 Sc + 0.25 Sh); the liberal one
/// (eq. 1b): SA = f [1.25 (Sc + Sh) − SL].
pub fn allowable_range(f: f64, sc: f64, sh: f64, liberal: Option<f64>) -> f64 {
    match liberal {
        Some(sl) => (f * (1.25 * (sc + sh) - sl)).max(f * (1.25 * sc + 0.25 * sh)),
        None => f * (1.25 * sc + 0.25 * sh),
    }
}

/// The stress at a point from its section resultants: axial force Fa, torsion Mt, in-plane and
/// out-of-plane bending Mi and Mo.
#[derive(Clone, Copy, Debug, Default)]
pub struct Loads {
    pub fa: f64,
    pub mt: f64,
    pub mi: f64,
    pub mo: f64,
}

/// Sustained longitudinal stress (para. 320.2): SL = √((|Ia Fa/Ap| + Sb + Sp)² + (2 St)²), with
/// Sb = √((Ii Mi)² + (Io Mo)²) / Z, St = It Mt / (2 Z), Sp = P di² / (Do² − di²), the sustained
/// factors Ii = 0.75 ii and Io = 0.75 io (at least 1), Ia = It = 1, and Z, Ap and di for the wall
/// less the corrosion allowance.
pub fn sustained(l: Loads, sif: Sif, p: f64, od: f64, t_corroded: f64) -> f64 {
    let (zz, ap) = (z(od, t_corroded), area(od, t_corroded));
    let di = od - 2.0 * t_corroded;
    let (ii, io) = ((0.75 * sif.ii).max(1.0), (0.75 * sif.io).max(1.0));
    let sb = ((ii * l.mi).powi(2) + (io * l.mo).powi(2)).sqrt() / zz;
    let st = l.mt.abs() / (2.0 * zz);
    let sp = if p > 0.0 { p * di * di / (od * od - di * di) } else { 0.0 };
    ((l.fa.abs() / ap + sb + sp).powi(2) + (2.0 * st).powi(2)).sqrt()
}

/// Displacement stress range (para. 319.4.4): SE = √((|Sa| + Sb)² + (2 St)²), with Sa = Fa/Ap,
/// Sb = √((ii Mi)² + (io Mo)²) / Z and St = Mt / (2 Z), on the nominal wall.
pub fn expansion(l: Loads, sif: Sif, od: f64, t: f64) -> f64 {
    let (zz, ap) = (z(od, t), area(od, t));
    let sb = ((sif.ii * l.mi).powi(2) + (sif.io * l.mo).powi(2)).sqrt() / zz;
    let st = l.mt.abs() / (2.0 * zz);
    ((l.fa.abs() / ap + sb).powi(2) + (2.0 * st).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equations_by_hand() {
        // 6" Sch 40 (168.3 × 7.11 mm), P = 10 MPa, S = 138 MPa, E = W = 1, Y = 0.4:
        // t = 10 × 168.275 / (2 × (138 + 4)) = 5.92518 mm.
        let t = pressure_thickness(10e6, 0.168_275, 138e6, 1.0, 1.0, 0.4);
        assert!((t - 0.005_925_176).abs() < 1e-9, "{t}");
        // The hoop stress at that thickness is S again.
        assert!((hoop_stress(10e6, 0.168_275, t, 1.0, 1.0, 0.4) - 138e6).abs() < 1.0);
        // A 6" long-radius elbow: T = 7.112, R = 228.6, r = 80.58 mm: h = 0.2504, k = 6.59,
        // ii = 2.266, io = 1.888.
        let b = bend(0.007_112, 0.168_275, 0.2286, 0.0, 0.0);
        assert!((b.h - 0.250_37).abs() < 1e-4 && (b.k - 6.590).abs() < 1e-2 && (b.ii - 2.2655).abs() < 1e-3 && (b.io - 1.8879).abs() < 1e-3, "{b:?}");
        // A welding tee on the same pipe: h = 3.1 × 7.112 / 80.58 = 0.2736; io = 2.135, ii = 1.852.
        let w = tee(crate::model::TeeKind::Welding, 0.007_112, 0.168_275, 0.0);
        assert!((w.h - 0.273_6).abs() < 1e-3 && (w.io - 2.1354).abs() < 1e-3 && (w.ii - 1.8516).abs() < 1e-3, "{w:?}");
        assert_eq!(range_factor(7000.0), 1.0);
        assert!((range_factor(100_000.0) - 6.0 * 1e5f64.powf(-0.2)).abs() < 1e-12);
        assert!((allowable_range(1.0, 138e6, 138e6, None) - 207e6).abs() < 1.0);
        // Pure bending on straight pipe: SE = M / Z.
        let zz = z(0.168_275, 0.007_112);
        let se = expansion(Loads { mi: 1000.0, ..Default::default() }, Sif::ONE, 0.168_275, 0.007_112);
        assert!((se - 1000.0 / zz).abs() < 1e-6);
    }
}
