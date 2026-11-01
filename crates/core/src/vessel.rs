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
    m.vessels.iter().map(|v| match check(v) { Ok(r) => VesselCheck { result: Some(r), error: None }, Err(e) => VesselCheck { result: None, error: Some(e) } }).collect()
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
        Ok(())
    }
}

/// Check a vessel.
pub fn check(v: &Vessel) -> Result<VesselResult, String> {
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
    let pass = out.iter().all(|p| p.pass);
    Ok(VesselResult { name: v.name.clone(), parts: out, mawp, map, test, governs, volume, empty, operating, test_weight, pass, outline, warnings })
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
