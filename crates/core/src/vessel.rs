//! Pressure vessels (0.3.1): a vessel is a row of parts (heads, cylinders, cones) from the bottom
//! (or the left, for a horizontal vessel) to the top. Each part is checked for internal pressure
//! with the thin-shell membrane equations of pressure vessel design, on its corroded dimensions,
//! with the static head of the liquid at its lowest point:
//!
//! - cylinder (hoop): t = P R / (S E − 0.6 P), MAWP = S E t / (R + 0.6 t); longitudinal:
//!   t = P R / (2 S E + 0.4 P);
//! - sphere and hemispherical head: t = P R / (2 S E − 0.2 P), MAWP = 2 S E t / (R + 0.2 t);
//! - 2:1 ellipsoidal head: t = P D / (2 S E − 0.2 P), MAWP = 2 S E t / (D + 0.2 t);
//! - torispherical head (crown radius L, knuckle radius r): M = (3 + √(L / r)) / 4,
//!   t = P L M / (2 S E − 0.2 P), MAWP = 2 S E t / (L M + 0.2 t);
//! - cone (half apex angle α, D the larger inside diameter): t = P D / (2 cos α (S E − 0.6 P)),
//!   MAWP = 2 S E t cos α / (D + 1.2 t cos α).
//!
//! R and D are inside dimensions grown by the corrosion allowance, t the thickness left after
//! corrosion and the plate's under-tolerance. The vessel's MAWP (at the top, hot and corroded) is
//! the lowest of the parts' less their static head; the MAP is the same new and cold; the
//! hydrotest pressure 1.3 × MAWP × (S cold / S hot). The allowable stresses, the joint
//! efficiencies and the yield strength are the user's inputs, pre-filled with typical values.
//! Weights and volumes come from each part's profile turned about the axis.
//!
//! Nozzles (0.3.2) are checked by area replacement: the metal the opening removes, d × tr (d the
//! corroded opening, tr the wall the part needs for pressure with a joint efficiency of 1), must be
//! made up within the reinforcement limits by the shell's excess thickness A1 = max(d, 2 (t + tn))
//! (t − tr), the neck's A2 = 5 (tn − trn) min(t, tn), the welds (leg²) and a reinforcing pad
//! (Dp − d − 2 tn) te with Dp up to 2 d. The neck itself must be at least the larger of its own
//! pressure thickness and the smaller of the shell's required thickness and the standard-wall
//! pipe's minimum (87.5 % of its nominal), each with the corrosion allowance. Small openings (a
//! finished opening up to 60.3 mm, or up to 88.9 mm in a wall up to 10 mm) need no reinforcement
//! calculation.
//!
//! Horizontal vessels on two saddles (0.3.3) are checked by Zick's method (1951): the vessel as a
//! beam carrying its heavier weight (operating or full of water) on two saddles; the longitudinal
//! stress at mid-span and at the saddles (an unstiffened shell's effective section K1, K1*), the
//! tangential shear (K2, or K3 where the heads stiffen the shell, A ≤ R/2), the circumferential
//! stress at the saddle horn (K6) and the compression at the bottom (K5), with the constants from
//! their closed forms in the contact angle.

use serde::{Deserialize, Serialize};

/// The kind of a vessel part.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PartKind {
    /// A cylindrical shell course of axial length `len`.
    Cylinder,
    /// A 2:1 semi-ellipsoidal head (depth D/4).
    Ellipsoidal,
    /// A torispherical head: crown radius and knuckle radius as fractions of the inside diameter
    /// (1.0 and 0.06 for the common shape).
    Torispherical { crown: f64, knuckle: f64 },
    /// A hemispherical head (depth D/2).
    Hemispherical,
    /// A cone from this part's diameter to `d_end` over the axial length `len`.
    Cone { d_end: f64 },
}

/// One part of a vessel: its name, kind, inside diameter (m, at its start for a cone), axial length
/// (m; cylinders and cones), nominal thickness (m) and joint efficiency.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Part {
    pub name: String,
    #[serde(flatten)]
    pub kind: PartKind,
    pub d: f64,
    pub len: f64,
    pub t: f64,
    pub e: f64,
}

impl Default for Part {
    fn default() -> Self {
        Part { name: "Shell".into(), kind: PartKind::Cylinder, d: 1.5, len: 3.0, t: 0.014, e: 1.0 }
    }
}

/// A pressure vessel and its design data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Vessel {
    pub name: String,
    /// Vertical (parts from the bottom up) or horizontal (from left to right).
    pub vertical: bool,
    /// Internal design pressure at the top (Pa, gauge) and design temperature (K).
    pub p: f64,
    pub temp: f64,
    /// Allowable stress at the design temperature and at the test (ambient) temperature, and the
    /// yield strength at the test temperature (Pa): the user's inputs.
    pub s: f64,
    pub s_cold: f64,
    pub sy: f64,
    /// Corrosion allowance and the plate's under-tolerance (m).
    pub corrosion: f64,
    pub under: f64,
    /// Density of the metal (kg/m³).
    pub rho: f64,
    /// The operating liquid's density (kg/m³) and its level above the bottom tangent line (vertical)
    /// or above the bottom of the shell (horizontal) (m).
    pub liquid: f64,
    pub level: f64,
    /// Extra weight of internals, platforms and insulation (kg).
    pub extra: f64,
    pub parts: Vec<Part>,
    /// Nozzles (0.3.2).
    #[serde(default)]
    pub nozzles: Vec<VNozzle>,
    /// Two saddles, for a horizontal vessel (0.3.3).
    #[serde(default)]
    pub saddles: Option<Saddles>,
    /// A skirt with its base ring and anchor bolts, for a vertical vessel (0.3.4).
    #[serde(default)]
    pub skirt: Option<Skirt>,
    /// Wind and seismic on a vertical vessel (0.3.4): the shape factor, the factor on the outside
    /// diameter for ladders, platforms and insulation, the horizontal seismic coefficient (g), the
    /// metal's elastic modulus (Pa) for the buckling allowable, and the base's elevation (m) for the
    /// site's wind profile. The wind pressure is the model's (Analysis settings).
    #[serde(default = "cd0")]
    pub wind_cd: f64,
    #[serde(default = "dfac0")]
    pub wind_dfac: f64,
    #[serde(default)]
    pub seis: f64,
    #[serde(default = "emod0")]
    pub emod: f64,
    #[serde(default)]
    pub base_el: f64,
    /// Bolted flanges and covers (0.3.5).
    #[serde(default)]
    pub flanges: Vec<BodyFlange>,
    /// Lifting lugs and trunnions (0.3.6).
    #[serde(default)]
    pub lifting: Option<Lifting>,
    /// Legs instead of a skirt, for a vertical vessel (0.3.6).
    #[serde(default)]
    pub legs: Option<Legs>,
}

/// Lifting: plate lugs (number, thickness, width at the base, height of the hole above the base,
/// radius of the metal round the hole, hole and pin diameters, fillet weld leg at the base) and the
/// sling's angle from vertical; pipe trunnions (outside diameter, wall, arm to the load; 0 = none);
/// the impact factor on the empty weight, and the lug and trunnion metal's yield strength.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Lifting {
    pub lugs: u32,
    pub t: f64,
    pub w: f64,
    pub h: f64,
    pub r: f64,
    pub hole: f64,
    pub pin: f64,
    pub weld: f64,
    pub sling: f64,
    pub tr_od: f64,
    pub tr_t: f64,
    pub tr_arm: f64,
    pub impact: f64,
    pub sy: f64,
}

impl Default for Lifting {
    fn default() -> Self {
        Lifting { lugs: 2, t: 0.03, w: 0.25, h: 0.12, r: 0.075, hole: 0.055, pin: 0.05, weld: 0.012, sling: 30.0, tr_od: 0.0, tr_t: 0.0, tr_arm: 0.0, impact: 1.5, sy: 250e6 }
    }
}

/// Legs: number, length from the base to the bottom tangent line, their section (area, section
/// modulus, radius of gyration), the circle they stand on, and their yield strength.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Legs {
    pub n: u32,
    pub len: f64,
    pub area: f64,
    pub z: f64,
    pub rg: f64,
    pub circle: f64,
    pub sy: f64,
}

impl Default for Legs {
    fn default() -> Self {
        // Four 6" Sch 80 pipe legs (A 5.42e-3 m², Z 1.89e-4 m³, r 0.0559 m).
        Legs { n: 4, len: 1.5, area: 5.42e-3, z: 1.89e-4, rg: 0.0559, circle: 1.6, sy: 240e6 }
    }
}

/// A row of the lifting or leg checks.
#[derive(Clone, Debug, Serialize)]
pub struct CheckRow {
    pub what: String,
    pub value: f64,
    pub allow: f64,
    pub ratio: f64,
    pub pass: bool,
}

fn crow(what: &str, value: f64, allow: f64) -> CheckRow {
    let ratio = if allow > 0.0 { value.abs() / allow } else { f64::INFINITY };
    CheckRow { what: what.into(), value, allow, ratio, pass: ratio <= 1.0 + 1e-12 }
}

/// Lifting lugs and trunnions under the empty weight times the impact factor, by statics.
pub fn lifting_check(l: &Lifting, empty_kg: f64) -> Vec<CheckRow> {
    let mut rows = vec![];
    let w = l.impact * empty_kg * G;
    if l.lugs > 0 && l.t > 0.0 {
        let phi = l.sling.to_radians();
        let f = w / l.lugs as f64 / phi.cos();
        rows.push(crow("Lug: tension across the hole", f / ((2.0 * l.r - l.hole) * l.t), 0.6 * l.sy));
        rows.push(crow("Lug: shear tear-out", f / (2.0 * (l.r - l.hole / 2.0) * l.t), 0.4 * l.sy));
        rows.push(crow("Lug: pin bearing", f / (l.pin * l.t), 0.9 * l.sy));
        let (fv, fh) = (f * phi.cos(), f * phi.sin());
        rows.push(crow("Lug: tension and bending at the base", fv / (l.w * l.t) + 6.0 * fh * l.h / (l.t * l.w * l.w), 0.66 * l.sy));
        if l.weld > 0.0 {
            let throat = 0.707 * l.weld;
            let fn_ = fv / (2.0 * throat * l.w) + 6.0 * fh * l.h / (2.0 * throat * l.w * l.w);
            let fs = fh / (2.0 * throat * l.w);
            rows.push(crow("Lug: base fillet welds", (fn_ * fn_ + fs * fs).sqrt(), 0.4 * l.sy));
        }
    }
    if l.tr_od > 0.0 && l.tr_t > 0.0 {
        // Two trunnions share the lift; each a cantilever from the shell to its load.
        let f = w / 2.0;
        let id = l.tr_od - 2.0 * l.tr_t;
        let pi = std::f64::consts::PI;
        let z = pi / 32.0 * (l.tr_od.powi(4) - id.powi(4)) / l.tr_od;
        let a = pi / 4.0 * (l.tr_od * l.tr_od - id * id);
        rows.push(crow("Trunnion: bending at the shell", f * l.tr_arm / z, 0.66 * l.sy));
        rows.push(crow("Trunnion: shear", 2.0 * f / a, 0.4 * l.sy));
    }
    rows
}

/// Legs: each carries the weight over the legs plus the overturning moment over the leg circle
/// (4 M / (n D)), against Euler buckling of a cantilever leg with a factor of 2 and 0.6 Sy, with the
/// base shear's bending in the leg (fixed at the shell, free at the base: V / n × L).
pub fn legs_check(g: &Legs, weight_n: f64, moment: f64, shear: f64, e: f64) -> Vec<CheckRow> {
    let n = g.n.max(1) as f64;
    let p = weight_n / n + 4.0 * moment / (n * g.circle);
    let fa = p / g.area;
    let kl_r = 2.0 * g.len / g.rg;
    let euler = std::f64::consts::PI.powi(2) * e / (kl_r * kl_r);
    let fa_allow = (euler / 2.0).min(0.6 * g.sy);
    let fb = shear / n * g.len / g.z;
    let fb_allow = 0.66 * g.sy;
    vec![
        crow("Legs: axial (weight and overturning)", fa, fa_allow),
        crow("Legs: bending from the base shear", fb, fb_allow),
        crow("Legs: combined fa/Fa + fb/Fb", fa / fa_allow + fb / fb_allow, 1.0),
    ]
}

/// A bolted flange or cover at the vessel's design pressure: a ring (loose) flange or a blind
/// cover; outside, inside and bolt circle diameters and thickness; the gasket's outside and inside
/// diameters and its factors m and y; the bolts (number, root area each); the allowable stresses of
/// the bolts at ambient and design, and of the flange at ambient and design (all m, m², Pa).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BodyFlange {
    pub name: String,
    pub blind: bool,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub t: f64,
    pub g_od: f64,
    pub g_id: f64,
    pub m: f64,
    pub y: f64,
    pub bolts: u32,
    pub bolt_area: f64,
    pub sa: f64,
    pub sb: f64,
    pub sfa: f64,
    pub sfo: f64,
}

impl Default for BodyFlange {
    fn default() -> Self {
        BodyFlange { name: "Cover".into(), blind: true, a: 0.775, b: 0.0, c: 0.686, t: 0.05, g_od: 0.584, g_id: 0.540, m: 2.0, y: 11e6, bolts: 24, bolt_area: 599e-6, sa: 172e6, sb: 172e6, sfa: 138e6, sfo: 138e6 }
    }
}

/// A flange's result: the gasket's effective width and diameter, the bolt loads and areas, and the
/// flange stresses (ring) or required thickness (blind), operating and gasket seating.
#[derive(Clone, Debug, Serialize)]
pub struct FlangeResult {
    pub name: String,
    pub b_eff: f64,
    pub g: f64,
    pub h: f64,
    pub wm1: f64,
    pub wm2: f64,
    pub am: f64,
    pub ab: f64,
    pub w_seat: f64,
    /// Ring: the moments and tangential stresses (operating, seating) and Y. Blind: the required
    /// thickness (operating, seating).
    pub m_op: f64,
    pub m_seat: f64,
    pub y_factor: f64,
    pub op: f64,
    pub seat: f64,
    pub op_allow: f64,
    pub seat_allow: f64,
    pub bolts_pass: bool,
    pub pass: bool,
}

/// The factor Y of a flat ring flange of diameter ratio K = A / B (from the plate theory of bolted
/// flanges): Y = (0.66845 + 5.71690 K² log₁₀K / (K² − 1)) / (K − 1).
pub fn ring_y(k: f64) -> f64 {
    (0.66845 + 5.71690 * k * k * k.log10() / (k * k - 1.0)) / (k - 1.0)
}

/// Check a bolted flange or cover at pressure p.
pub fn flange_check(f: &BodyFlange, p: f64) -> FlangeResult {
    let pi = std::f64::consts::PI;
    // The gasket: its seating width N/2, and the effective width b (b0 up to 6.35 mm, else 2.52 √b0 in mm).
    let n = (f.g_od - f.g_id) / 2.0;
    let b0 = n / 2.0;
    let b = if b0 <= 0.00635 { b0 } else { 0.00252 * (b0 * 1000.0).sqrt() };
    let g = if b0 <= 0.00635 { (f.g_od + f.g_id) / 2.0 } else { f.g_od - 2.0 * b };
    let h = pi / 4.0 * g * g * p;
    let hp = 2.0 * b * pi * g * f.m * p;
    let wm1 = h + hp;
    let wm2 = pi * b * g * f.y;
    let am = (wm1 / f.sb).max(wm2 / f.sa);
    let ab = f.bolts as f64 * f.bolt_area;
    let w_seat = (am + ab) * f.sa / 2.0;
    let hg_arm = (f.c - g) / 2.0;
    let (m_op, m_seat, y_factor, op, seat, op_allow, seat_allow);
    if f.blind {
        // A flat bolted cover: t = d √(C P / S + 1.9 W hG / (S d³)), C = 0.3, d = G.
        let d = g;
        op = d * (0.3 * p / f.sfo + 1.9 * wm1 * hg_arm / (f.sfo * d.powi(3))).sqrt();
        seat = d * (1.9 * w_seat * hg_arm / (f.sfa * d.powi(3))).sqrt();
        (m_op, m_seat, y_factor) = (wm1 * hg_arm, w_seat * hg_arm, 0.0);
        (op_allow, seat_allow) = (f.t, f.t);
    } else {
        // A ring flange: the end force inside B, the force on the face between B and G and the gasket
        // load, each with its arm to the bolt circle; the tangential stress Y Mo / (t² B).
        let hd = pi / 4.0 * f.b * f.b * p;
        let hd_arm = (f.c - f.b) / 2.0;
        let ht = h - hd;
        let ht_arm = (hd_arm + hg_arm) / 2.0;
        let hg = wm1 - h;
        m_op = hd * hd_arm + ht * ht_arm + hg * hg_arm;
        m_seat = w_seat * hg_arm;
        y_factor = ring_y(f.a / f.b);
        op = y_factor * m_op / (f.t * f.t * f.b);
        seat = y_factor * m_seat / (f.t * f.t * f.b);
        (op_allow, seat_allow) = (f.sfo, f.sfa);
    }
    let bolts_pass = ab + 1e-15 >= am;
    let pass = bolts_pass && if f.blind { f.t + 1e-12 >= op.max(seat) } else { op <= op_allow * (1.0 + 1e-12) && seat <= seat_allow * (1.0 + 1e-12) };
    FlangeResult { name: f.name.clone(), b_eff: b, g, h, wm1, wm2, am, ab, w_seat, m_op, m_seat, y_factor, op, seat, op_allow, seat_allow, bolts_pass, pass }
}

fn cd0() -> f64 {
    0.7
}
fn dfac0() -> f64 {
    1.2
}
fn emod0() -> f64 {
    195e9
}

/// A vertical vessel's skirt (height from the base to the bottom tangent line, thickness; its
/// outside diameter is the shell's), base ring (outside and inside diameters, thickness) and anchor
/// bolts (number, root diameter, bolt circle, allowable stress), and the foundation's allowable
/// bearing pressure (all m, Pa).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Skirt {
    pub h: f64,
    pub t: f64,
    pub ring_od: f64,
    pub ring_id: f64,
    pub ring_t: f64,
    pub bolts: u32,
    pub bolt_d: f64,
    pub bc: f64,
    pub bolt_s: f64,
    pub bearing: f64,
}

impl Default for Skirt {
    fn default() -> Self {
        Skirt { h: 2.0, t: 0.012, ring_od: 2.1, ring_id: 1.6, ring_t: 0.025, bolts: 8, bolt_d: 0.0237, bc: 1.95, bolt_s: 140e6, bearing: 7e6 }
    }
}

/// One horizontal section of a vertical vessel: where, what it is, the wind and seismic moments
/// above it, the operating weight above, its thickness and radius, and the longitudinal stresses
/// (tension with pressure, compression without) against their allowables.
#[derive(Clone, Debug, Serialize)]
pub struct SectionRow {
    pub name: String,
    pub z: f64,
    pub m_wind: f64,
    pub m_seis: f64,
    pub weight: f64,
    pub tension: f64,
    pub compression: f64,
    pub allow_t: f64,
    pub allow_c: f64,
    pub pass: bool,
}

/// The vertical vessel's support check.
#[derive(Clone, Debug, Serialize)]
pub struct SupportResult {
    pub shear_wind: f64,
    pub moment_wind: f64,
    pub shear_seis: f64,
    pub moment_seis: f64,
    pub weight_op: f64,
    pub weight_empty: f64,
    pub sections: Vec<SectionRow>,
    /// Anchor bolts: tension per bolt (N, 0 when none) and its stress against the allowable.
    pub bolt_load: f64,
    pub bolt_stress: f64,
    pub bolt_allow: f64,
    /// Foundation bearing under the base ring, and the ring's required and given thickness.
    pub bearing: f64,
    pub bearing_allow: f64,
    pub ring_t_req: f64,
    pub ring_t: f64,
    pub pass: bool,
}

/// Two equal saddles under a horizontal vessel: the distance from each tangent line to its saddle's
/// centre (m), the contact angle (degrees), the saddle's width along the vessel (m), whether the
/// saddles are welded to the shell, and whether the shell has stiffening rings at the saddles.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Saddles {
    pub a: f64,
    pub theta: f64,
    pub b: f64,
    pub welded: bool,
    pub rings: bool,
}

impl Default for Saddles {
    fn default() -> Self {
        Saddles { a: 0.6, theta: 120.0, b: 0.3, welded: true, rings: false }
    }
}

/// One stress of the saddle check, its allowable, and the ratio.
#[derive(Clone, Debug, Serialize)]
pub struct SaddleRow {
    pub what: String,
    pub stress: f64,
    pub allow: f64,
    pub ratio: f64,
    pub pass: bool,
}

/// The saddle check: the load on each saddle, the bending moments, the constants and the stresses.
#[derive(Clone, Debug, Serialize)]
pub struct SaddleResult {
    pub q: f64,
    pub m1: f64,
    pub m2: f64,
    pub k: [f64; 6],
    pub rows: Vec<SaddleRow>,
    pub pass: bool,
}

/// The saddle constants of Zick's method for contact angle θ (radians) and A / R: K1 and K1* (the
/// effective section at an unstiffened saddle, top and bottom), K2 (shear, shell not stiffened),
/// K3 (shear, shell stiffened by the head), K5 (compression at the bottom), K6 (the horn).
pub fn zick_k(theta: f64, a_over_r: f64) -> [f64; 6] {
    let pi = std::f64::consts::PI;
    let d = pi / 6.0 + 5.0 * theta / 12.0;
    let (sd, cd) = (d.sin(), d.cos());
    let num = d + sd * cd - 2.0 * sd * sd / d;
    let k1 = num / (pi * (sd / d - cd));
    let k1s = num / (pi * (1.0 - sd / d));
    let al = 0.95 * (pi - theta / 2.0);
    let (sa, ca) = (al.sin(), al.cos());
    let den = pi - al + sa * ca;
    let k2 = sa / den;
    let k3 = sa / pi * (al - sa * ca) / den;
    let k5 = (1.0 + ca) / den;
    let b = pi - theta / 2.0;
    let (sb, cb) = (b.sin(), b.cos());
    let dd = (sb / b).powi(2) - 0.5 - (2.0 * b).sin() / (4.0 * b);
    let k6full = (0.75 * cb * (sb / b).powi(2) - 5.0 * sb * cb * cb / (4.0 * b) + cb.powi(3) / 2.0 - sb / (4.0 * b) + cb / 4.0 - b * sb * dd) / (2.0 * pi * dd);
    // A / R from 0.5 to 1: from a quarter of the value to the full value.
    let f = ((a_over_r - 0.5) / 0.5).clamp(0.0, 1.0);
    let k6 = k6full * (0.25 + 0.75 * f);
    [k1, k1s, k2, k3, k5, k6]
}

/// A nozzle on a vessel: the part it is on, where (cylinders and cones: the distance from the
/// part's start along the axis; heads: the offset from the axis), its direction round the vessel
/// (degrees, for the drawing: vertical vessels 0 to the right, 180 to the left; horizontal 0 on
/// top, 180 underneath), the neck pipe (outside diameter and nominal wall, pipe with a 12.5 %
/// under-tolerance when `pipe`), its nominal size for the standard-wall minimum, the outside fillet
/// weld leg, and a reinforcing pad (outside diameter, thickness, its weld leg; 0 = none).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VNozzle {
    pub name: String,
    pub part: usize,
    pub pos: f64,
    pub angle: f64,
    pub nps: String,
    pub od: f64,
    pub t: f64,
    pub pipe: bool,
    pub leg: f64,
    pub pad_od: f64,
    pub pad_t: f64,
    pub pad_leg: f64,
    /// Outward projection from the shell (m), for the drawing.
    pub proj: f64,
    /// The piping model's nozzle point this nozzle carries (0 = none): its loads from the piping
    /// analysis are checked on this nozzle's neck (0.3.7).
    #[serde(default)]
    pub link: u32,
}

/// A linked nozzle's neck under the piping loads of one case: the longitudinal stress of pressure,
/// axial force and bending on the neck's corroded section, combined with torsion by the maximum
/// shear stress, against 1.5 S.
#[derive(Clone, Debug, Serialize)]
pub struct LinkCheck {
    pub vessel: String,
    pub nozzle: String,
    pub case: String,
    pub local: [f64; 6],
    pub stress: f64,
    pub allow: f64,
    pub ratio: f64,
    pub pass: bool,
}

/// The neck stress of every linked vessel nozzle in every case of the piping analysis.
pub fn link_checks(m: &crate::model::Model, cases: &[crate::solve::CaseResult]) -> Vec<LinkCheck> {
    let mut out = vec![];
    let pi = std::f64::consts::PI;
    for v in &m.vessels {
        for z in v.nozzles.iter().filter(|z| z.link != 0) {
            let tn = if z.pipe { z.t * 0.875 } else { z.t } - v.corrosion;
            let d_in = z.od - 2.0 * z.t + 2.0 * v.corrosion;
            let dm = d_in + tn;
            let (a, zs) = (pi * dm * tn, pi * dm * dm * tn / 4.0);
            for c in cases {
                let Some(nl) = c.nozzles.iter().find(|x| x.nozzle == z.link) else { continue };
                let l = nl.local;
                let p = if matches!(c.kind, crate::model::CaseKind::Exp | crate::model::CaseKind::Occ) { 0.0 } else { v.p };
                let sl = p * d_in / (4.0 * tn) + l[0].abs() / a + (l[4] * l[4] + l[5] * l[5]).sqrt() / zs;
                let st = l[3].abs() / (2.0 * zs);
                let stress = (sl * sl + 4.0 * st * st).sqrt();
                let allow = 1.5 * v.s;
                let ratio = stress / allow;
                out.push(LinkCheck { vessel: v.name.clone(), nozzle: z.name.clone(), case: c.name.clone(), local: l, stress, allow, ratio, pass: ratio <= 1.0 + 1e-12 });
            }
        }
    }
    out
}

impl Default for VNozzle {
    fn default() -> Self {
        let (od, t) = crate::library::pipe("4", "80").unwrap_or((0.1143, 0.00856));
        VNozzle { name: "N1".into(), part: 1, pos: 1.0, angle: 0.0, nps: "4".into(), od, t, pipe: true, leg: 0.008, pad_od: 0.0, pad_t: 0.0, pad_leg: 0.0, proj: 0.2, link: 0 }
    }
}

/// A nozzle's result: the areas (m²) and the neck's thickness check.
#[derive(Clone, Debug, Serialize)]
pub struct NozzleResult {
    pub name: String,
    pub d: f64,
    pub tr: f64,
    pub trn: f64,
    pub a_req: f64,
    pub a1: f64,
    pub a2: f64,
    pub a41: f64,
    pub a5: f64,
    pub a42: f64,
    pub a_avail: f64,
    /// A small opening that needs no reinforcement calculation.
    pub small: bool,
    pub area_pass: bool,
    pub t_neck_req: f64,
    pub t_neck: f64,
    pub neck_pass: bool,
    pub pass: bool,
    /// Where it is drawn: (radius, axial position) of the root and the direction (degrees).
    pub at: [f64; 2],
    pub warnings: Vec<String>,
}

impl Default for Vessel {
    fn default() -> Self {
        Vessel {
            name: "V-1".into(),
            vertical: true,
            p: 10e5,
            temp: 273.15 + 150.0,
            s: 138e6,
            s_cold: 138e6,
            sy: 260e6,
            corrosion: 0.003,
            under: 0.0,
            rho: 7850.0,
            liquid: 1000.0,
            level: 0.0,
            extra: 0.0,
            parts: vec![
                Part { name: "Bottom head".into(), kind: PartKind::Ellipsoidal, d: 1.5, len: 0.0, t: 0.014, e: 1.0 },
                Part { name: "Shell".into(), kind: PartKind::Cylinder, d: 1.5, len: 4.0, t: 0.014, e: 1.0 },
                Part { name: "Top head".into(), kind: PartKind::Ellipsoidal, d: 1.5, len: 0.0, t: 0.014, e: 1.0 },
            ],
            nozzles: vec![],
            saddles: None,
            skirt: None,
            wind_cd: cd0(),
            wind_dfac: dfac0(),
            seis: 0.0,
            emod: emod0(),
            base_el: 0.0,
            flanges: vec![],
            lifting: None,
            legs: None,
        }
    }
}

/// One part's result.
#[derive(Clone, Debug, Serialize)]
pub struct PartResult {
    pub name: String,
    /// The part's lowest and highest points along the axis (m, from the bottom tangent line).
    pub z: [f64; 2],
    /// The pressure it is designed for: design pressure plus the static head at its lowest point.
    pub p: f64,
    pub static_head: f64,
    /// Required thickness for pressure, with corrosion and under-tolerance added, and nominal.
    pub t_req: f64,
    pub t_min: f64,
    pub t_nom: f64,
    /// MAWP hot and corroded at the part, and at the top (less its static head); MAP new and cold.
    pub mawp: f64,
    pub mawp_top: f64,
    pub map: f64,
    /// Membrane stress in the hydrotest (new, with the test's static head) and its limit 0.9 Sy.
    pub test_stress: f64,
    pub test_allow: f64,
    pub pass: bool,
    /// The equation used, for the report.
    pub formula: &'static str,
    /// Inside volume (m³), metal weight (kg), inside surface (m²).
    pub volume: f64,
    pub metal: f64,
}

/// The whole vessel's result, and its outline for the drawing.
#[derive(Clone, Debug, Serialize)]
pub struct VesselResult {
    pub name: String,
    pub parts: Vec<PartResult>,
    pub mawp: f64,
    pub map: f64,
    pub test: f64,
    /// The part that sets the MAWP.
    pub governs: Option<usize>,
    pub volume: f64,
    pub empty: f64,
    pub operating: f64,
    pub test_weight: f64,
    pub pass: bool,
    /// The outline: (radius, axial position) of the outside surface from the bottom (or left) end to
    /// the top, for the drawing; and the inside radius at the same points.
    pub outline: Vec<[f64; 3]>,
    pub nozzles: Vec<NozzleResult>,
    pub saddles: Option<SaddleResult>,
    pub support: Option<SupportResult>,
    pub flanges: Vec<FlangeResult>,
    pub lifting: Vec<CheckRow>,
    pub legs: Vec<CheckRow>,
    pub warnings: Vec<String>,
}

const G: f64 = 9.806_65;

/// One vessel's check for the window: its result, or why it could not be checked.
#[derive(Clone, Debug, Serialize)]
pub struct VesselCheck {
    pub result: Option<VesselResult>,
    pub error: Option<String>,
}

/// Every vessel of the model.
pub fn check_all(m: &crate::model::Model) -> Vec<VesselCheck> {
    m.vessels.iter().map(|v| match check_in(v, &m.settings) { Ok(r) => VesselCheck { result: Some(r), error: None }, Err(e) => VesselCheck { result: None, error: Some(e) } }).collect()
}

/// A head's depth (inside, m) for inside diameter d.
fn head_depth(k: &PartKind, d: f64) -> f64 {
    match k {
        PartKind::Ellipsoidal => d / 4.0,
        PartKind::Hemispherical => d / 2.0,
        PartKind::Torispherical { crown, knuckle } => {
            let (l, r) = (crown * d, knuckle * d);
            let a = d / 2.0 - r;
            l - ((l - r).powi(2) - a * a).max(0.0).sqrt()
        }
        _ => 0.0,
    }
}

/// The inside profile of a head as (radius, depth from the tangent line) from the tangent line
/// (radius d/2, depth 0) to the pole (radius 0).
fn head_profile(k: &PartKind, d: f64, n: usize) -> Vec<[f64; 2]> {
    let a = d / 2.0;
    match k {
        PartKind::Ellipsoidal | PartKind::Hemispherical => {
            let c = head_depth(k, d);
            (0..=n).map(|i| {
                let th = std::f64::consts::FRAC_PI_2 * i as f64 / n as f64;
                [a * th.cos(), c * th.sin()]
            }).collect()
        }
        PartKind::Torispherical { crown, knuckle } => {
            let (l, r) = (crown * d, knuckle * d);
            // The knuckle's centre is at (a − r, 0); the crown's centre on the axis at (0, −(L − depth)).
            let h = head_depth(k, d);
            let zc = h - l;
            let phi = ((a - r) / (l - r)).clamp(-1.0, 1.0).asin();
            let mut v = vec![];
            let nk = n / 2;
            for i in 0..=nk {
                let th = (std::f64::consts::FRAC_PI_2 - phi) * i as f64 / nk as f64;
                v.push([a - r + r * th.cos(), r * th.sin()]);
            }
            for i in 1..=n - nk {
                let th = phi * (1.0 - i as f64 / (n - nk) as f64);
                v.push([l * th.sin(), zc + l * th.cos()]);
            }
            v
        }
        _ => vec![[a, 0.0], [0.0, 0.0]],
    }
}

/// Volume (m³) and surface (m²) of a profile turned about the axis, the profile given as
/// (radius, axial position) points.
fn turned(p: &[[f64; 2]]) -> (f64, f64) {
    let (mut v, mut s) = (0.0, 0.0);
    for w in p.windows(2) {
        let (r1, z1, r2, z2) = (w[0][0], w[0][1], w[1][0], w[1][1]);
        let dz = (z2 - z1).abs();
        v += std::f64::consts::PI * dz * (r1 * r1 + r1 * r2 + r2 * r2) / 3.0;
        s += std::f64::consts::PI * (r1 + r2) * ((r2 - r1).powi(2) + (z2 - z1).powi(2)).sqrt();
    }
    (v, s)
}

/// Required thickness and MAWP of a part for pressure p, corroded inside dimensions (a cone's
/// end diameter too), thickness t available, stress s; also the equation's name.
pub fn pressure(k: &PartKind, d_in: f64, len: f64, p: f64, s: f64, e: f64, t: f64) -> (f64, f64, &'static str) {
    let r = d_in / 2.0;
    let se = s * e;
    match k {
        PartKind::Cylinder => {
            let hoop = p * r / (se - 0.6 * p);
            let long = p * r / (2.0 * se + 0.4 * p);
            (hoop.max(long), (se * t / (r + 0.6 * t)).min(2.0 * se * t / (r - 0.4 * t)), "t = P R / (S E − 0.6 P)")
        }
        PartKind::Hemispherical => (p * r / (2.0 * se - 0.2 * p), 2.0 * se * t / (r + 0.2 * t), "t = P R / (2 S E − 0.2 P)"),
        PartKind::Ellipsoidal => (p * d_in / (2.0 * se - 0.2 * p), 2.0 * se * t / (d_in + 0.2 * t), "t = P D / (2 S E − 0.2 P)"),
        PartKind::Torispherical { crown, knuckle } => {
            // Crown and knuckle radii of the corroded head: the formed radii grown by the corrosion.
            let l = crown * d_in;
            let m = (3.0 + (crown / knuckle).sqrt()) / 4.0;
            (p * l * m / (2.0 * se - 0.2 * p), 2.0 * se * t / (l * m + 0.2 * t), "t = P L M / (2 S E − 0.2 P)")
        }
        PartKind::Cone { d_end } => {
            let dl = d_in.max(*d_end);
            let a = cone_angle(d_in, *d_end, len);
            let c = a.cos();
            (p * dl / (2.0 * c * (se - 0.6 * p)), 2.0 * se * t * c / (dl + 1.2 * t * c), "t = P D / (2 cos α (S E − 0.6 P))")
        }
    }
}

/// A cone's half apex angle from its two diameters and axial length.
pub fn cone_angle(d1: f64, d2: f64, len: f64) -> f64 {
    ((d1 - d2).abs() / 2.0).atan2(len.max(1e-12))
}

impl Vessel {
    pub fn check_data(&self) -> Result<(), String> {
        let fin = |v: f64| v.is_finite();
        if !(self.p >= 0.0 && fin(self.p) && self.s > 0.0 && fin(self.s) && self.s_cold > 0.0 && fin(self.s_cold) && self.sy >= 0.0 && fin(self.sy)) {
            return Err(format!("The vessel {}: its pressure must be 0 or more and its allowable stresses above 0.", self.name));
        }
        if ![self.corrosion, self.under, self.rho, self.liquid, self.level, self.extra].iter().all(|v| fin(*v) && *v >= 0.0) || !fin(self.temp) {
            return Err(format!("The vessel {}: corrosion, under-tolerance, densities, liquid level and extra weight must be 0 or more.", self.name));
        }
        if self.parts.is_empty() {
            return Err(format!("The vessel {} needs at least one part.", self.name));
        }
        let n = self.parts.len();
        for (i, p) in self.parts.iter().enumerate() {
            if !(p.d > 0.0 && fin(p.d) && p.t > 0.0 && fin(p.t) && p.e > 0.0 && p.e <= 1.0 && p.len >= 0.0 && fin(p.len)) {
                return Err(format!("The part {} of {}: its diameter and thickness must be above 0 and its joint efficiency between 0 and 1.", p.name, self.name));
            }
            let head = !matches!(p.kind, PartKind::Cylinder | PartKind::Cone { .. });
            if head && i != 0 && i != n - 1 {
                return Err(format!("The head {} of {} must be the first or the last part.", p.name, self.name));
            }
            match p.kind {
                PartKind::Cylinder if p.len <= 0.0 => return Err(format!("The cylinder {} of {} needs a length.", p.name, self.name)),
                PartKind::Cone { d_end } if !(d_end > 0.0 && fin(d_end) && p.len > 0.0) => return Err(format!("The cone {} of {} needs its end diameter and a length.", p.name, self.name)),
                PartKind::Cone { d_end } if cone_angle(p.d, d_end, p.len).to_degrees() > 30.0 => return Err(format!("The cone {} of {}: its half apex angle is over 30°, where these equations do not apply.", p.name, self.name)),
                PartKind::Torispherical { crown, knuckle } if !(crown > 0.0 && crown <= 1.0 && (0.06..0.5).contains(&knuckle) && fin(crown) && fin(knuckle)) => {
                    return Err(format!("The torispherical head {} of {}: crown radius up to the diameter, knuckle radius from 6 % of it.", p.name, self.name))
                }
                _ => {}
            }
        }
        if let Some(k) = &self.skirt {
            if !self.vertical {
                return Err(format!("The vessel {} is horizontal: a skirt is for vertical vessels.", self.name));
            }
            if ![k.h, k.t, k.ring_od, k.ring_id, k.ring_t, k.bolt_d, k.bc, k.bolt_s, k.bearing].iter().all(|v| fin(*v) && *v >= 0.0) || !(k.h > 0.0 && k.t > 0.0 && k.ring_od > k.ring_id) {
                return Err(format!("The skirt of {}: its height and thickness must be above 0, the base ring's outside diameter larger than its inside, and the rest 0 or more.", self.name));
            }
        }
        if !(self.wind_cd >= 0.0 && fin(self.wind_cd) && self.wind_dfac >= 1.0 && fin(self.wind_dfac) && self.seis >= 0.0 && self.seis <= 5.0 && self.emod > 0.0 && fin(self.emod) && fin(self.base_el)) {
            return Err(format!("The vessel {}: the shape factor 0 or more, the diameter factor 1 or more, the seismic coefficient from 0 to 5 g and the modulus above 0.", self.name));
        }
        for f in &self.flanges {
            let ok = [f.a, f.b, f.c, f.t, f.g_od, f.g_id, f.m, f.y, f.bolt_area, f.sa, f.sb, f.sfa, f.sfo].iter().all(|v| fin(*v) && *v >= 0.0)
                && f.a > f.c && f.c > f.g_od && f.g_od > f.g_id && f.g_id >= 0.0 && f.t > 0.0 && f.bolts > 0 && f.bolt_area > 0.0 && f.sa > 0.0 && f.sb > 0.0 && f.sfa > 0.0 && f.sfo > 0.0
                && (f.blind || (f.b > 0.0 && f.b < f.g_id + 1e-9));
            if !ok {
                return Err(format!("The flange {} of {}: outside diameter > bolt circle > gasket outside > gasket inside (> bore for a ring flange), a thickness, bolts and allowable stresses above 0.", f.name, self.name));
            }
        }
        if let Some(l) = &self.lifting {
            if ![l.t, l.w, l.h, l.r, l.hole, l.pin, l.weld, l.sling, l.tr_od, l.tr_t, l.tr_arm, l.impact, l.sy].iter().all(|v| fin(*v) && *v >= 0.0) || l.sling >= 80.0 || l.impact < 1.0 || l.sy <= 0.0 || (l.lugs > 0 && !(l.t > 0.0 && l.w > 0.0 && l.r > l.hole / 2.0 && l.pin <= l.hole)) || (l.tr_od > 0.0 && 2.0 * l.tr_t >= l.tr_od) {
                return Err(format!("The lifting data of {}: sizes 0 or more, the hole inside the lug and the pin no larger than the hole, a sling angle under 80°, an impact factor of 1 or more.", self.name));
            }
        }
        if let Some(g) = &self.legs {
            if !self.vertical || self.skirt.is_some() {
                return Err(format!("The vessel {}: legs are for a vertical vessel without a skirt.", self.name));
            }
            if g.n < 3 || ![g.len, g.area, g.z, g.rg, g.circle, g.sy].iter().all(|v| fin(*v) && *v > 0.0) {
                return Err(format!("The legs of {}: three or more, with a length, section and circle above 0.", self.name));
            }
        }
        if let Some(s) = &self.saddles {
            if self.vertical {
                return Err(format!("The vessel {} is vertical: saddles are for horizontal vessels.", self.name));
            }
            if !(s.a >= 0.0 && fin(s.a) && s.b > 0.0 && fin(s.b) && (120.0..=180.0).contains(&s.theta)) {
                return Err(format!("The saddles of {}: the distance from the tangent line must be 0 or more, the width above 0 and the contact angle from 120° to 180°.", self.name));
            }
        }
        for z in &self.nozzles {
            if z.part >= n || ![z.pos, z.angle, z.od, z.t, z.leg, z.pad_od, z.pad_t, z.pad_leg, z.proj].iter().all(|v| fin(*v)) || !(z.od > 0.0 && z.t > 0.0 && 2.0 * z.t < z.od) || [z.leg, z.pad_od, z.pad_t, z.pad_leg, z.proj].iter().any(|v| *v < 0.0) {
                return Err(format!("The nozzle {} of {}: it needs a part of the vessel, an outside diameter larger than twice its wall, and sizes of 0 or more.", z.name, self.name));
            }
            if z.pad_od > 0.0 && z.pad_od <= z.od {
                return Err(format!("The nozzle {} of {}: its pad must be larger than the nozzle.", z.name, self.name));
            }
        }
        Ok(())
    }
}

/// The nozzle checks of a vessel, given each part's design pressure and axial span.
fn nozzles(v: &Vessel, pressures: &[f64], spans: &[[f64; 2]]) -> Vec<NozzleResult> {
    v.nozzles.iter().map(|z| {
        let p = &v.parts[z.part];
        let ca = v.corrosion;
        let pd = pressures[z.part];
        let mut warnings = vec![];
        // The part's required thickness for pressure with E = 1 (the opening is away from the welds).
        let kind = match p.kind { PartKind::Cone { d_end } => PartKind::Cone { d_end: d_end + 2.0 * ca }, ref k => k.clone() };
        let d_part = match p.kind { PartKind::Cone { d_end } => p.d + (d_end - p.d) * (z.pos / p.len.max(1e-9)).clamp(0.0, 1.0), _ => p.d };
        let (tr, _, _) = pressure(&kind, d_part + 2.0 * ca, p.len, pd, v.s, 1.0, 0.0);
        let t = p.t - ca - v.under;
        let tn_nom = if z.pipe { z.t * 0.875 } else { z.t };
        let tn = tn_nom - ca;
        let d = z.od - 2.0 * z.t + 2.0 * ca;
        let rn = d / 2.0;
        let trn = pd * rn / (v.s - 0.6 * pd);
        let a_req = d * tr;
        let a1 = d.max(2.0 * (t + tn)) * (t - tr);
        let a2 = (5.0 * (tn - trn) * t.min(tn)).max(0.0) * if tn > trn { 1.0 } else { 0.0 };
        let a41 = z.leg * z.leg;
        let (a5, a42) = if z.pad_od > 0.0 && z.pad_t > 0.0 {
            let dp = z.pad_od.min(2.0 * d);
            if z.pad_od > 2.0 * d {
                warnings.push(format!("The pad of {} reaches past the reinforcement limit (2 d = {:.0} mm): only the part inside it counts.", z.name, 2.0 * d * 1e3));
            }
            (((dp - d - 2.0 * tn).max(0.0)) * z.pad_t, z.pad_leg * z.pad_leg)
        } else {
            (0.0, 0.0)
        };
        let a_avail = a1.max(0.0) + a2 + a41 + a5 + a42;
        // Small openings need no reinforcement calculation: a finished opening up to 60.3 mm in any
        // wall, or up to 88.9 mm in a wall up to 10 mm.
        let small = d <= 0.0603 + 1e-9 || (d <= 0.0889 + 1e-9 && p.t <= 0.010 + 1e-9);
        let area_pass = small || a_avail + 1e-15 >= a_req;
        // The neck's minimum: its pressure thickness, but at least the smaller of the shell's need and
        // the standard-wall pipe's minimum, each with the corrosion allowance.
        let std = crate::library::pipe(&z.nps, "STD").filter(|s| (s.0 - z.od).abs() < 1e-4).map(|s| s.1 * 0.875);
        let tb1 = tr + ca;
        let tb = match std { Some(s) => tb1.min(s + ca), None => tb1 };
        let t_neck_req = (trn + ca).max(tb);
        let neck_pass = tn_nom + 1e-12 >= t_neck_req;
        if d > p.d / 2.0 && d > 0.508 {
            warnings.push(format!("{} is a large opening (over half the diameter and over 508 mm): area replacement alone may not be enough.", z.name));
        }
        if z.pipe && std.is_none() {
            warnings.push(format!("{}: no standard-wall pipe of that size in the library, so the neck's minimum uses the shell's need.", z.name));
        }
        // Where to draw it.
        let at = match p.kind {
            PartKind::Cylinder | PartKind::Cone { .. } => [d_part / 2.0 + p.t, spans[z.part][0] + z.pos],
            _ => {
                let h = head_depth(&p.kind, p.d);
                let (zt, sign) = if z.part == 0 { (spans[z.part][1], -1.0) } else { (spans[z.part][0], 1.0) };
                let r = z.pos.abs().min(p.d / 2.0 * 0.95);
                // The head's outside surface height at radius r (an ellipse is close enough to draw).
                let a = p.d / 2.0;
                let dz = h * (1.0 - (r / a).powi(2)).max(0.0).sqrt() + p.t;
                [z.pos, zt + sign * dz]
            }
        };
        NozzleResult { name: z.name.clone(), d, tr, trn, a_req, a1, a2, a41, a5, a42, a_avail, small, area_pass, t_neck_req, t_neck: tn_nom, neck_pass, pass: area_pass && neck_pass, at, warnings }
    }).collect()
}

/// The longitudinal compressive stress a cylinder can carry: classical axial buckling
/// 0.605 E t / R with a knockdown for imperfections and a safety factor, 0.0625 E t / R, and never
/// more than the allowable stress S.
pub fn compressive_allow(e: f64, t: f64, r: f64, s: f64) -> f64 {
    (0.0625 * e * t / r).min(s)
}

/// A vertical vessel under its weight, wind and seismic, on a skirt: the moments and weight above
/// each section by statics, the shell's and the skirt's longitudinal stresses, the anchor bolts,
/// the foundation bearing and the base ring's thickness.
fn supports(v: &Vessel, site: &crate::model::Settings, spans: &[[f64; 2]], profiles: &[Vec<[f64; 2]>], empty: f64, operating: f64, warnings: &mut Vec<String>) -> SupportResult {
    let n = v.parts.len();
    let top = spans.iter().map(|s| s[1]).fold(f64::NEG_INFINITY, f64::max);
    let bottom = spans.iter().map(|s| s[0]).fold(f64::INFINITY, f64::min);
    let skirt_h = v.skirt.as_ref().map_or(0.0, |k| k.h);
    let zb = if v.skirt.is_some() { -skirt_h } else if let Some(g) = &v.legs { -g.len } else { bottom };
    let shell_od = v.parts.iter().filter(|p| matches!(p.kind, PartKind::Cylinder)).map(|p| p.d + 2.0 * p.t).fold(0.0, f64::max);
    // The outside radius at height z, from the outline of each part (the skirt below the tangent line).
    let r_at = |z: f64| -> f64 {
        let mut r: f64 = 0.0;
        for (i, pr) in profiles.iter().enumerate() {
            for w in pr.windows(2) {
                let (lo, hi) = (w[0][1].min(w[1][1]), w[0][1].max(w[1][1]));
                if z >= lo && z <= hi && hi > lo {
                    let f = (z - w[0][1]) / (w[1][1] - w[0][1]);
                    r = r.max(w[0][0] + (w[1][0] - w[0][0]) * f + v.parts[i].t);
                }
            }
        }
        if z < 0.0 && v.skirt.is_some() { r.max(shell_od / 2.0) } else { r }
    };
    // Mass per height: each part's metal and its share of the liquid over its span, the skirt's metal,
    // and the extra weight over the tangent length.
    let liq = operating - empty;
    let vol: f64 = profiles.iter().map(|p| turned(p).0).sum::<f64>().max(1e-12);
    let mut lumps: Vec<(f64, f64, f64)> = vec![]; // (z0, z1, kg)
    for (i, pr) in profiles.iter().enumerate() {
        let (pv, area) = turned(pr);
        let liquid_share = if v.level > 0.0 && v.liquid > 0.0 { liq * pv / vol } else { 0.0 };
        lumps.push((spans[i][0], spans[i][1], area * v.parts[i].t * v.rho + liquid_share));
    }
    if let Some(k) = &v.skirt {
        lumps.push((zb, 0.0, std::f64::consts::PI * (shell_od - k.t) * k.t * k.h * v.rho));
    }
    let tl = (spans[n - 1][0] - spans[0][1]).max(1e-9);
    lumps.push((spans[0][1], spans[n - 1][0], v.extra));
    let steps = 400;
    let dz = (top - zb) / steps as f64;
    let mut wind = vec![0.0; steps];
    let mut mass = vec![0.0; steps];
    let zc: Vec<f64> = (0..steps).map(|i| zb + (i as f64 + 0.5) * dz).collect();
    for i in 0..steps {
        let z = zc[i];
        let q = site.wind_at(v.base_el + (z - zb));
        wind[i] = q * v.wind_cd * v.wind_dfac * 2.0 * r_at(z) * dz;
        for &(a, b, kg) in &lumps {
            if b > a && z >= a && z < b {
                mass[i] += kg * dz / (b - a);
            }
        }
    }
    let _ = tl;
    let seis: Vec<f64> = mass.iter().map(|m| v.seis * m * G).collect();
    let above = |z: f64, f: &[f64]| -> (f64, f64) {
        let (mut sh, mut mo) = (0.0, 0.0);
        for i in 0..steps {
            if zc[i] > z {
                sh += f[i];
                mo += f[i] * (zc[i] - z);
            }
        }
        (sh, mo)
    };
    let weight_above = |z: f64, m: &[f64]| -> f64 { (0..steps).filter(|&i| zc[i] > z).map(|i| m[i] * G).sum() };
    let (shear_wind, moment_wind) = above(zb, &wind);
    let (shear_seis, moment_seis) = above(zb, &seis);
    let mut sections = vec![];
    let ca = v.corrosion;
    // Each cylindrical course at its bottom.
    for (i, p) in v.parts.iter().enumerate() {
        if !matches!(p.kind, PartKind::Cylinder) {
            continue;
        }
        let z = spans[i][0];
        let t = p.t - ca - v.under;
        let r = (p.d + 2.0 * ca + t) / 2.0;
        let (_, mw) = above(z, &wind);
        let (_, ms) = above(z, &seis);
        let m = mw.max(ms);
        let w = weight_above(z, &mass);
        let a = 2.0 * std::f64::consts::PI * r * t;
        let zmod = std::f64::consts::PI * r * r * t;
        let tension = v.p * r / (2.0 * t) + m / zmod - w / a;
        let compression = -m / zmod - w / a;
        let allow_c = compressive_allow(v.emod, t, r, v.s);
        sections.push(SectionRow { name: format!("{} (bottom)", p.name), z, m_wind: mw, m_seis: ms, weight: w, tension, compression, allow_t: v.s * p.e, allow_c, pass: tension <= v.s * p.e + 1e-6 && -compression <= allow_c + 1e-6 });
    }
    let (mut bolt_load, mut bolt_stress, mut bolt_allow, mut bearing, mut bearing_allow, mut ring_t_req, mut ring_t) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    if let Some(k) = &v.skirt {
        let pi = std::f64::consts::PI;
        let t = k.t - ca;
        let r = (shell_od - k.t) / 2.0;
        let a = 2.0 * pi * r * t;
        let zmod = pi * r * r * t;
        let allow_c = compressive_allow(v.emod, t, r, v.s);
        for (name, z) in [("Skirt (top)", -1e-6), ("Skirt (base)", zb)] {
            let (_, mw) = above(z, &wind);
            let (_, ms) = above(z, &seis);
            let m = mw.max(ms);
            // The skirt carries no pressure: compression with the operating weight, tension with the empty one.
            let w_op = weight_above(z, &mass);
            let w_empty = w_op * (empty / operating.max(1e-9));
            let compression = -m / zmod - w_op / a;
            let tension = m / zmod - w_empty / a;
            sections.push(SectionRow { name: name.into(), z, m_wind: mw, m_seis: ms, weight: w_op, tension, compression, allow_t: v.s, allow_c, pass: tension <= v.s + 1e-6 && -compression <= allow_c + 1e-6 });
        }
        // Anchor bolts: the moment over the bolt circle less the empty weight, shared by the bolts.
        let m = moment_wind.max(moment_seis);
        let we = empty * G;
        if k.bolts > 0 && k.bc > 0.0 && k.bolt_d > 0.0 {
            bolt_load = (4.0 * m / (k.bolts as f64 * k.bc) - we / k.bolts as f64).max(0.0);
            bolt_stress = bolt_load / (pi / 4.0 * k.bolt_d * k.bolt_d);
            bolt_allow = k.bolt_s;
        }
        // Bearing under the ring: the operating weight over its area plus the moment over its section modulus.
        let ab = pi / 4.0 * (k.ring_od.powi(2) - k.ring_id.powi(2));
        let zb_mod = pi * (k.ring_od.powi(4) - k.ring_id.powi(4)) / (32.0 * k.ring_od);
        bearing = operating * G / ab + m / zb_mod;
        bearing_allow = k.bearing;
        // The ring's outside projection as a cantilever strip under the bearing pressure.
        let l = ((k.ring_od - shell_od) / 2.0).max(0.0);
        ring_t_req = l * (3.0 * bearing / v.s).sqrt();
        ring_t = k.ring_t;
        if k.ring_od < shell_od {
            warnings.push(format!("The base ring of {} is smaller than the skirt.", v.name));
        }
    }
    let pass = sections.iter().all(|s| s.pass) && (bolt_allow == 0.0 || bolt_stress <= bolt_allow + 1e-6) && (bearing_allow == 0.0 || bearing <= bearing_allow + 1e-6) && ring_t + 1e-12 >= ring_t_req;
    SupportResult { shear_wind, moment_wind, shear_seis, moment_seis, weight_op: operating * G, weight_empty: empty * G, sections, bolt_load, bolt_stress, bolt_allow, bearing, bearing_allow, ring_t_req, ring_t, pass }
}

/// The saddle check by Zick's method (L. P. Zick, Stresses in large horizontal cylindrical pressure
/// vessels on two saddle supports, 1951): the vessel as a beam on two saddles carrying its heavier
/// weight (operating or full of water), the shell at the design pressure, hot and corroded.
fn saddle_check(v: &Vessel, s: &Saddles, weight: f64, spans: &[[f64; 2]], warnings: &mut Vec<String>) -> Result<SaddleResult, String> {
    let shells: Vec<usize> = (0..v.parts.len()).filter(|&i| matches!(v.parts[i].kind, PartKind::Cylinder)).collect();
    if shells.is_empty() || v.parts.iter().any(|p| matches!(p.kind, PartKind::Cone { .. })) {
        return Err(format!("The saddles of {} need a cylindrical shell between two heads (no cones).", v.name));
    }
    let shell = &v.parts[shells[0]];
    let n = v.parts.len();
    let ca = v.corrosion;
    let t = shell.t - ca - v.under;
    let r = (shell.d + t) / 2.0 + ca;
    let l = spans[n - 1][0] - spans[0][1];
    let h = head_depth(&v.parts[n - 1].kind, v.parts[n - 1].d);
    let th = v.parts[n - 1].t.min(v.parts[0].t) - ca - v.under;
    let a = s.a;
    if 2.0 * a >= l {
        return Err(format!("The saddles of {} are past the middle of the vessel.", v.name));
    }
    let q = weight * G / 2.0;
    let m1 = -q * a * (1.0 - (1.0 - a / l + (r * r - h * h) / (2.0 * a * l).max(1e-12)) / (1.0 + 4.0 * h / (3.0 * l)));
    let m1 = if a > 0.0 { m1 } else { 0.0 };
    let m2 = q * l / 4.0 * ((1.0 + 2.0 * (r * r - h * h) / (l * l)) / (1.0 + 4.0 * h / (3.0 * l)) - 4.0 * a / l);
    let theta = s.theta.to_radians();
    let k = zick_k(theta, a / r);
    let pi = std::f64::consts::PI;
    let pl = v.p * r / (2.0 * t);
    let z = pi * r * r * t;
    let stiff = s.rings || a <= r / 2.0;
    let (sa, sy) = (v.s, v.sy);
    let mut rows = vec![];
    let mut row = |what: &str, stress: f64, allow: f64| {
        let ratio = stress.abs() / allow;
        rows.push(SaddleRow { what: what.into(), stress, allow, ratio, pass: ratio <= 1.0 + 1e-12 });
    };
    // Longitudinal: pressure plus bending, at mid-span and at the saddles, with and without pressure.
    row("Longitudinal, mid-span (tension)", pl + m2.abs() / z, sa);
    row("Longitudinal, mid-span (compression, no pressure)", -m2.abs() / z, sa);
    let (zt, zb) = if stiff { (z, z) } else { (k[0] * z, k[1] * z) };
    row("Longitudinal, at the saddles (tension)", pl + m1.abs() / zt.min(zb), sa);
    row("Longitudinal, at the saddles (compression, no pressure)", -m1.abs() / zt.min(zb), sa);
    // Tangential shear at the saddle.
    let shear_load = q * (l - 2.0 * a) / (l + 4.0 * h / 3.0);
    if stiff && !s.rings {
        row("Tangential shear in the shell (stiffened by the head)", k[3] * q / (r * t), 0.8 * sa);
        row("Tangential shear in the head", k[3] * q / (r * th), 0.8 * sa);
    } else {
        row("Tangential shear in the shell", k[2] * shear_load / (r * t), 0.8 * sa);
    }
    // Circumferential stress at the saddle horn, and compression at the bottom.
    let spread = s.b + 1.56 * (r * t).sqrt();
    let horn = if l >= 8.0 * r { -q / (4.0 * t * spread) - 3.0 * k[5] * q / (2.0 * t * t) } else { -q / (4.0 * t * spread) - 12.0 * k[5] * q * r / (l * t * t) };
    if !s.rings {
        row("Circumferential at the saddle horn", horn, 1.25 * sa);
    } else {
        warnings.push(format!("The saddles of {} have stiffening rings: the rings themselves are not checked.", v.name));
    }
    let kw = if s.welded { 0.1 } else { 1.0 };
    if sy > 0.0 {
        row("Circumferential compression at the bottom", -k[4] * q * kw / (t * spread), 0.5 * sy);
    }
    let pass = rows.iter().all(|x| x.pass);
    Ok(SaddleResult { q, m1, m2, k, rows, pass })
}

/// Check a vessel with no wind (the site's wind comes with `check_in`).
pub fn check(v: &Vessel) -> Result<VesselResult, String> {
    check_in(v, &crate::model::Settings { wind_q: 0.0, wind_profile: vec![], ..Default::default() })
}

/// Check a vessel at the model's site (its wind pressure or profile).
pub fn check_in(v: &Vessel, site: &crate::model::Settings) -> Result<VesselResult, String> {
    v.check_data()?;
    let n = v.parts.len();
    let mut warnings = vec![];
    // Axial positions: the bottom tangent line is at 0 (the first part's far end when it is a head).
    let mut z = 0.0;
    let mut spans = vec![[0.0; 2]; n];
    let mut profiles: Vec<Vec<[f64; 2]>> = vec![];
    for (i, p) in v.parts.iter().enumerate() {
        match p.kind {
            PartKind::Cylinder | PartKind::Cone { .. } => {
                spans[i] = [z, z + p.len];
                let d2 = if let PartKind::Cone { d_end } = p.kind { d_end } else { p.d };
                profiles.push(vec![[p.d / 2.0, z], [d2 / 2.0, z + p.len]]);
                z += p.len;
            }
            _ => {
                let h = head_depth(&p.kind, p.d);
                let prof = head_profile(&p.kind, p.d, 24);
                if i == 0 {
                    spans[i] = [-h, 0.0];
                    // From the pole up to the tangent line.
                    profiles.push(prof.iter().rev().map(|q| [q[0], -q[1]]).collect());
                } else {
                    spans[i] = [z, z + h];
                    profiles.push(prof.iter().map(|q| [q[0], z + q[1]]).collect());
                }
            }
        }
    }
    // Diameters must meet at every joint.
    for i in 1..n {
        let prev = match v.parts[i - 1].kind { PartKind::Cone { d_end } => d_end, _ => v.parts[i - 1].d };
        if (prev - v.parts[i].d).abs() > 1e-6 {
            warnings.push(format!("{} ends at {:.0} mm but {} starts at {:.0} mm.", v.parts[i - 1].name, prev * 1e3, v.parts[i].name, v.parts[i].d * 1e3));
        }
    }
    let bottom = spans.iter().map(|s| s[0]).fold(f64::INFINITY, f64::min);
    let head_at = |zlow: f64| if v.vertical { v.liquid * G * (v.level - zlow).max(0.0) } else { v.liquid * G * v.level };
    // The hydrotest: full of water; the static head at the part's lowest point below the top.
    let top = spans.iter().map(|s| s[1]).fold(f64::NEG_INFINITY, f64::max);
    let height = if v.vertical { top - bottom } else { v.parts.iter().map(|p| p.d).fold(0.0, f64::max) };
    let mut parts = vec![];
    let (mut volume, mut metal_total) = (0.0, 0.0);
    for (i, p) in v.parts.iter().enumerate() {
        let sh = head_at(spans[i][0]);
        let pd = v.p + sh;
        let d_corr = p.d + 2.0 * v.corrosion;
        let t_avail = p.t - v.corrosion - v.under;
        let kind = match p.kind { PartKind::Cone { d_end } => PartKind::Cone { d_end: d_end + 2.0 * v.corrosion }, ref k => k.clone() };
        let (treq, mawp, formula) = pressure(&kind, d_corr, p.len, pd, v.s, p.e, t_avail);
        let (_, map, _) = pressure(&p.kind, p.d, p.len, 0.0, v.s_cold, p.e, p.t - v.under);
        let t_min = treq + v.corrosion + v.under;
        let mawp_top = mawp - sh;
        // Hydrotest stress: new thickness, the test pressure at the top plus the water's head here.
        parts.push((i, sh, pd, treq, t_min, mawp, mawp_top, map, formula));
        // Metal: the surface times the thickness (the inside surface is close enough for a weight).
        let (vol, area) = turned(&profiles[i]);
        volume += vol;
        metal_total += area * p.t * v.rho;
    }
    let mawp = parts.iter().map(|x| x.6).fold(f64::INFINITY, f64::min);
    let map = parts.iter().map(|x| x.7).fold(f64::INFINITY, f64::min);
    let governs = parts.iter().position(|x| x.6 == mawp);
    let test = 1.3 * mawp * (v.s_cold / v.s);
    let mut out = vec![];
    for (i, sh, pd, treq, t_min, m, mt, mp, formula) in parts {
        let p = &v.parts[i];
        let pt = test + 1000.0 * G * if v.vertical { top - spans[i][0] } else { height };
        // The membrane stress the test gives this part (new thickness): the MAWP equation solved for S.
        let (_, unit, _) = pressure(&p.kind, p.d, p.len, 0.0, 1.0, 1.0, p.t - v.under);
        let ts = pt / unit;
        let allow = 0.9 * v.sy;
        let pass = p.t + 1e-12 >= t_min && (v.sy <= 0.0 || ts <= allow);
        let (vol, area) = turned(&profiles[i]);
        out.push(PartResult { name: p.name.clone(), z: spans[i], p: pd, static_head: sh, t_req: treq, t_min, t_nom: p.t, mawp: m, mawp_top: mt, map: mp, test_stress: ts, test_allow: allow, pass, formula, volume: vol, metal: area * p.t * v.rho });
    }
    let liquid_vol = liquid_volume(v, &profiles, bottom);
    let empty = metal_total + v.extra;
    let operating = empty + liquid_vol * v.liquid;
    let test_weight = empty + volume * 1000.0;
    // The outline: outside radius and inside radius along the axis.
    let mut outline = vec![];
    for (i, pr) in profiles.iter().enumerate() {
        let t = v.parts[i].t;
        for q in pr {
            outline.push([q[0] + t, q[1], q[0]]);
        }
    }
    let pressures: Vec<f64> = out.iter().map(|x| x.p).collect();
    let nz = nozzles(v, &pressures, &spans);
    for z in &nz {
        warnings.extend(z.warnings.iter().cloned());
    }
    let sad = match &v.saddles {
        Some(s) => Some(saddle_check(v, s, operating.max(test_weight), &spans, &mut warnings)?),
        None => None,
    };
    let sup = if v.vertical && (v.skirt.is_some() || v.legs.is_some() || site.has_wind() || v.seis > 0.0) {
        Some(supports(v, site, &spans, &profiles, empty, operating, &mut warnings))
    } else {
        None
    };
    let fl: Vec<FlangeResult> = v.flanges.iter().map(|f| flange_check(f, v.p)).collect();
    let lift = v.lifting.as_ref().map(|l| lifting_check(l, empty)).unwrap_or_default();
    let legs = match (&v.legs, &sup) {
        (Some(g), Some(s)) => legs_check(g, s.weight_op, s.moment_wind.max(s.moment_seis), s.shear_wind.max(s.shear_seis), v.emod),
        _ => vec![],
    };
    let pass = out.iter().all(|p| p.pass) && nz.iter().all(|z| z.pass) && sad.as_ref().is_none_or(|s| s.pass) && sup.as_ref().is_none_or(|s| s.pass) && fl.iter().all(|f| f.pass) && lift.iter().chain(&legs).all(|x| x.pass);
    Ok(VesselResult { name: v.name.clone(), parts: out, mawp, map, test, governs, volume, empty, operating, test_weight, pass, outline, nozzles: nz, saddles: sad, support: sup, flanges: fl, lifting: lift, legs, warnings })
}

/// The liquid's volume: up to the level (vertical, from the bottom tangent line) or the part of the
/// circle below the level (horizontal).
fn liquid_volume(v: &Vessel, profiles: &[Vec<[f64; 2]>], _bottom: f64) -> f64 {
    if v.liquid <= 0.0 || v.level <= 0.0 {
        return 0.0;
    }
    if v.vertical {
        let mut vol = 0.0;
        for pr in profiles {
            // Clip each profile at the level.
            let mut clipped: Vec<[f64; 2]> = vec![];
            for w in pr.windows(2) {
                let (a, b) = (w[0], w[1]);
                let (lo, hi) = (a[1].min(b[1]), a[1].max(b[1]));
                if lo >= v.level {
                    continue;
                }
                if hi <= v.level {
                    clipped.push(a);
                    clipped.push(b);
                } else {
                    let f = (v.level - a[1]) / (b[1] - a[1]);
                    let c = [a[0] + (b[0] - a[0]) * f, v.level];
                    if a[1] < v.level { clipped.push(a); clipped.push(c); } else { clipped.push(c); clipped.push(b); }
                }
            }
            for w in clipped.chunks(2) {
                if w.len() == 2 {
                    vol += turned(&[w[0], w[1]]).0;
                }
            }
        }
        vol
    } else {
        // The fraction of each cross-section's area below the level, times the part's volume.
        let mut vol = 0.0;
        for (i, pr) in profiles.iter().enumerate() {
            let r = v.parts[i].d / 2.0;
            let h = v.level.min(2.0 * r);
            let seg = r * r * ((r - h) / r).acos() - (r - h) * (2.0 * r * h - h * h).max(0.0).sqrt();
            let frac = seg / (std::f64::consts::PI * r * r);
            vol += turned(pr).0 * frac;
        }
        vol
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol * b.abs().max(1e-30)
    }

    #[test]
    fn the_shell_and_head_equations_by_hand() {
        // P 1.5 MPa, S 138 MPa: a cylinder of inside radius 750 mm with E 0.85 needs 9.665 mm;
        // a 2:1 head of 1500 mm with E 0.85 needs 9.603 mm, a hemisphere 4.801 mm, and a
        // torispherical head with L = D, r = 0.06 D needs 0.885 P L / (S E − 0.1 P) = 17.00 mm.
        let (s, e, p) = (138e6, 0.85, 1.5e6);
        let (t, _, _) = pressure(&PartKind::Cylinder, 1.5, 1.0, p, s, e, 0.0);
        assert!(close(t, 1.5e6 * 0.75 / (117.3e6 - 0.9e6), 1e-12) && close(t * 1e3, 9.665, 1e-3), "{t}");
        let (t, _, _) = pressure(&PartKind::Ellipsoidal, 1.5, 0.0, p, s, e, 0.0);
        assert!(close(t * 1e3, 9.603, 1e-3), "{t}");
        let (t, _, _) = pressure(&PartKind::Hemispherical, 1.5, 0.0, p, s, e, 0.0);
        assert!(close(t * 1e3, 4.801, 1e-3), "{t}");
        let (t, _, _) = pressure(&PartKind::Torispherical { crown: 1.0, knuckle: 0.06 }, 1.5, 0.0, p, s, e, 0.0);
        let m = (3.0 + (1.0f64 / 0.06).sqrt()) / 4.0;
        assert!(close(t, p * 1.5 * m / (2.0 * 117.3e6 - 0.2 * p), 1e-12), "{t}");
        assert!(close(t * 1e3, 0.885 * 1.5e6 * 1500.0 / (117.3e6 - 0.15e6), 0.005), "{t}");
        // MAWP of an 11 mm corroded shell, R 753 mm: S E t / (R + 0.6 t) = 1.699 MPa.
        let (_, m, _) = pressure(&PartKind::Cylinder, 1.506, 1.0, 0.0, s, e, 0.011);
        assert!(close(m / 1e6, 1.699, 1e-3), "{m}");
        // A cone of 30° from 1.5 m, at the large end: P D / (2 cos α (S E − 0.6 P)).
        let len = 0.25 / 30f64.to_radians().tan();
        let (t, _, _) = pressure(&PartKind::Cone { d_end: 1.0 }, 1.5, len, p, s, e, 0.0);
        assert!(close(t, p * 1.5 / (2.0 * 30f64.to_radians().cos() * (117.3e6 - 0.9e6)), 1e-9), "{t}");
    }

    #[test]
    fn heads_volumes_and_depths() {
        // A 2:1 head's inside volume is π D³ / 24; a hemisphere's π D³ / 12.
        let (v, _) = turned(&head_profile(&PartKind::Ellipsoidal, 2.0, 400));
        assert!(close(v, std::f64::consts::PI * 8.0 / 24.0, 1e-3), "{v}");
        let (v, s) = turned(&head_profile(&PartKind::Hemispherical, 2.0, 400));
        assert!(close(v, std::f64::consts::PI * 8.0 / 12.0, 1e-3) && close(s, 2.0 * std::f64::consts::PI, 1e-3), "{v} {s}");
        // The common torispherical head (L = D, r = 0.06 D) is 1 − √(0.94² − 0.44²) = 0.16934 D deep.
        let h = head_depth(&PartKind::Torispherical { crown: 1.0, knuckle: 0.06 }, 1.0);
        assert!((h - 0.169_338).abs() < 1e-5, "{h}");
        let p = head_profile(&PartKind::Torispherical { crown: 1.0, knuckle: 0.06 }, 1.0, 48);
        assert!((p[0][0] - 0.5).abs() < 1e-12 && p.last().unwrap()[0].abs() < 1e-9 && (p.last().unwrap()[1] - h).abs() < 1e-9, "{:?}", p.last());
    }

    #[test]
    fn nozzle_area_rules() {
        // A 6" Sch 40 nozzle in a 1 m shell, 10 bar, S 138 MPa, no corrosion, t 10 mm, no pad.
        let (od, tn) = crate::library::pipe("6", "40").unwrap();
        let mut v = Vessel { corrosion: 0.0, p: 10e5, liquid: 0.0, ..Default::default() };
        for p in &mut v.parts {
            p.d = 1.0;
            p.t = 0.010;
        }
        v.nozzles = vec![VNozzle { part: 1, pos: 1.0, nps: "6".into(), od, t: tn, pipe: true, leg: 0.006, ..Default::default() }];
        let r = check(&v).unwrap();
        let z = &r.nozzles[0];
        let tr = 10e5 * 0.5 / (138e6 - 0.6e6);
        let d = od - 2.0 * tn;
        let t_n = tn * 0.875;
        let trn = 10e5 * d / 2.0 / (138e6 - 0.6e6);
        assert!(close(z.a_req, d * tr, 1e-12));
        assert!(close(z.a1, d.max(2.0 * (0.010 + t_n)) * (0.010 - tr), 1e-12));
        assert!(close(z.a2, 5.0 * (t_n - trn) * t_n.min(0.010), 1e-12));
        assert!(close(z.a_avail, z.a1 + z.a2 + 0.006f64.powi(2), 1e-12));
        // The neck's minimum: the smaller of the shell's need and the standard wall's 87.5 %.
        let std = crate::library::pipe("6", "STD").unwrap().1 * 0.875;
        assert!(close(z.t_neck_req, trn.max(tr.min(std)), 1e-12));
        // A pad past 2 d counts only up to 2 d, and says so.
        v.nozzles[0].pad_od = 3.0 * d;
        v.nozzles[0].pad_t = 0.01;
        let r = check(&v).unwrap();
        assert!(close(r.nozzles[0].a5, (2.0 * d - d - 2.0 * t_n) * 0.01, 1e-12));
        assert!(r.warnings.iter().any(|w| w.contains("reinforcement limit")));
        // A pad smaller than the nozzle is refused.
        v.nozzles[0].pad_od = od * 0.9;
        assert!(check(&v).is_err());
    }

    #[test]
    fn zick_constants_at_120_degrees() {
        // The published table values for a 120° saddle: K1 0.107 and 0.192, K2 1.171, K3 0.880,
        // K5 0.760, K6 0.0528 (A/R ≥ 1); at A/R ≤ 0.5 a quarter of K6.
        let k = zick_k(120f64.to_radians(), 1.0);
        for (got, want) in k.iter().zip([0.107, 0.192, 1.171, 0.880, 0.760, 0.0528]) {
            assert!((got - want).abs() < 1.5e-3, "{k:?}");
        }
        let k = zick_k(120f64.to_radians(), 0.4);
        assert!((k[5] - 0.0528 / 4.0).abs() < 1e-3, "{k:?}");
        // At 150° the effective section is larger and the horn constant smaller.
        let k150 = zick_k(150f64.to_radians(), 1.0);
        assert!(k150[0] > 0.107 && k150[5] < 0.0528, "{k150:?}");
    }

    #[test]
    fn bolted_flanges_by_hand() {
        // Y at K = 1.5: (0.66845 + 5.71690 × 2.25 × log₁₀1.5 / 1.25) / 0.5 = 4.961.
        assert!(close(ring_y(1.5), 4.96098, 1e-4), "{}", ring_y(1.5));
        // The cover: N = 22 mm, b0 = 11 mm > 6.35, b = 2.52 √11 mm, G = Go − 2b; H = π/4 G² P,
        // Hp = 2 b π G m P, Wm2 = π b G y; t = G √(0.3 P/S + 1.9 Wm1 hG/(S G³)).
        let f = BodyFlange::default();
        let r = flange_check(&f, 1.6e6);
        let b = 2.52e-3 * 11f64.sqrt();
        assert!(close(r.b_eff, b, 1e-9) && close(r.g, 0.584 - 2.0 * b, 1e-12));
        let g = 0.584 - 2.0 * b;
        let pi = std::f64::consts::PI;
        let wm1 = pi / 4.0 * g * g * 1.6e6 + 2.0 * b * pi * g * 2.0 * 1.6e6;
        assert!(close(r.wm1, wm1, 1e-12) && close(r.wm2, pi * b * g * 11e6, 1e-12));
        let hg = (0.686 - g) / 2.0;
        assert!(close(r.op, g * (0.3 * 1.6e6 / 138e6 + 1.9 * wm1 * hg / (138e6 * g.powi(3))).sqrt(), 1e-12));
        // A ring flange: the moments' arms from the bolt circle, stress Y Mo / (t² B).
        let ring = BodyFlange { blind: false, b: 0.5, t: 0.06, ..f };
        let r = flange_check(&ring, 1.6e6);
        let h = pi / 4.0 * g * g * 1.6e6;
        let hd = pi / 4.0 * 0.25 * 1.6e6;
        let (ad, ag) = ((0.686 - 0.5) / 2.0, hg);
        let mo = hd * ad + (h - hd) * (ad + ag) / 2.0 + (wm1 - h) * ag;
        assert!(close(r.m_op, mo, 1e-12) && close(r.op, ring_y(0.775 / 0.5) * mo / (0.0036 * 0.5), 1e-12));
    }

    #[test]
    fn legs_by_hand() {
        // Four legs of A 5.42e-3 m², Z 1.89e-4 m³, r 0.0559 m, 1.5 m long on a 1.6 m circle; 100 kN
        // weight, 40 kN·m overturning, 10 kN shear: P = W/4 + 4M/(4 × 1.6); Euler with K = 2.
        let g = Legs::default();
        let rows = legs_check(&g, 100e3, 40e3, 10e3, 200e9);
        let p = 100e3 / 4.0 + 4.0 * 40e3 / (4.0 * 1.6);
        let kl = 2.0 * 1.5 / 0.0559;
        let fa_allow = (std::f64::consts::PI.powi(2) * 200e9 / (kl * kl) / 2.0).min(0.6 * 240e6);
        assert!(close(rows[0].value, p / 5.42e-3, 1e-12) && close(rows[0].allow, fa_allow, 1e-12));
        assert!(close(rows[1].value, 10e3 / 4.0 * 1.5 / 1.89e-4, 1e-12));
        // A leg vessel's base moment comes from its legs' length.
        let mut v = Vessel { legs: Some(g), seis: 0.2, ..Default::default() };
        v.parts[1].len = 3.0;
        let r = check(&v).unwrap();
        assert_eq!(r.legs.len(), 3);
        assert!(r.support.as_ref().unwrap().moment_seis > r.support.as_ref().unwrap().shear_seis * 1.5);
        // Legs and a skirt together are refused.
        v.skirt = Some(Skirt::default());
        assert!(check(&v).is_err());
    }

    #[test]
    fn a_vertical_vessel_with_liquid() {
        let mut v = Vessel { liquid: 800.0, level: 2.0, ..Default::default() };
        v.parts[1].len = 6.0;
        let r = check(&v).unwrap();
        // The shell's lowest point is the bottom tangent line, 2 m under the level.
        assert!(close(r.parts[1].static_head, 800.0 * G * 2.0, 1e-12));
        // The bottom head goes down D/4 more; the top head is dry.
        assert!(close(r.parts[0].static_head, 800.0 * G * 2.375, 1e-12) && r.parts[2].static_head == 0.0);
        // Volume: the cylinder and two 2:1 heads.
        let exact = std::f64::consts::PI * 0.75f64.powi(2) * 6.0 + 2.0 * std::f64::consts::PI * 1.5f64.powi(3) / 24.0;
        assert!(close(r.volume, exact, 2e-3), "{} {exact}", r.volume);
        // The liquid: the bottom head full and 2 m of the shell.
        let liq = std::f64::consts::PI * 1.5f64.powi(3) / 24.0 + std::f64::consts::PI * 0.75f64.powi(2) * 2.0;
        assert!(close(r.operating - r.empty, liq * 800.0, 3e-3));
        assert!(r.governs.is_some() && r.mawp > 0.0 && close(r.test, 1.3 * r.mawp, 1e-12));
        // A head in the middle is refused.
        let mut bad = v.clone();
        bad.parts.swap(1, 2);
        assert!(check(&bad).is_err());
    }
}
