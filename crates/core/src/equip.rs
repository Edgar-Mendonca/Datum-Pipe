//! Rotating equipment: groups of nozzle points on one machine, checked against the criteria the
//! published standards give as equations. Steam turbines and compressors: each nozzle against
//! c1 F + c2 M ≤ c3 De (3F + M ≤ 500 De in the standard's units: lb, ft·lb, inches), and the loads
//! of all nozzles resolved at the reference nozzle against c1 Fc + c2 Mc ≤ c3 Dc, with limits on
//! each component; compressors with a multiplier (typically 1.85). Pumps: each nozzle's force and
//! moment components and resultants against the values of the pump standard's table, which are the
//! user's inputs (the table is not reproduced). Every coefficient is an input pre-filled with typical
//! values, to be checked against the edition of the standard that applies.

use crate::mat::*;
use crate::model::Model;
use serde::{Deserialize, Serialize};

/// N per lbf, N·m per ft·lbf, m per inch.
pub const LBF: f64 = 4.448_221_615_260_5;
pub const FT_LBF: f64 = 1.355_817_948_331_400_4;
pub const INCH: f64 = 0.0254;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EquipKind {
    #[default]
    Turbine,
    Compressor,
    Pump,
}

/// The turbine and compressor criteria's coefficients (the standard's units: lb, ft·lb, inches).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Coef {
    /// Each nozzle: c1 F + c2 M ≤ c3 De.
    pub single: [f64; 3],
    /// De is the nominal size up to `de_cap` inches, above it (a + D) / b.
    pub de_cap: f64,
    pub de_over: [f64; 2],
    /// All nozzles at the reference: c1 Fc + c2 Mc ≤ c3 Dc.
    pub combined: [f64; 3],
    /// Dc is the diameter of the nozzles' combined area up to `dc_cap` inches, above it (a + Dc) / b.
    pub dc_cap: f64,
    pub dc_over: [f64; 2],
    /// The combined components' limits as multiples of Dc: Fx, Fy, Fz, Mx, My, Mz (x along the shaft,
    /// y up, z across).
    pub comp: [f64; 6],
}

impl Default for Coef {
    fn default() -> Self {
        Coef { single: [3.0, 1.0, 500.0], de_cap: 8.0, de_over: [16.0, 3.0], combined: [2.0, 1.0, 250.0], dc_cap: 9.0, dc_over: [18.0, 3.0], comp: [50.0, 125.0, 100.0, 250.0, 125.0, 125.0] }
    }
}

/// A pump nozzle's allowable loads from the pump standard's table (N, N·m), in the equipment's axes;
/// 0 = not checked.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PumpAllow {
    pub nozzle: u32,
    pub f: [f64; 3],
    pub m: [f64; 3],
    pub fr: f64,
    pub mr: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Equipment {
    pub name: String,
    pub kind: EquipKind,
    /// The nozzle points on this machine.
    pub nozzles: Vec<u32>,
    /// Where all the loads are resolved (the exhaust or discharge nozzle); 0 = the largest.
    pub reference: u32,
    /// The shaft's direction (horizontal): the equipment's x axis; y is up, z = x × y.
    pub shaft: [f64; 3],
    /// A multiplier on every allowable (1 for turbines, typically 1.85 for compressors; 1 or 2 for a
    /// pump's table values).
    pub factor: f64,
    pub coef: Coef,
    pub pump: Vec<PumpAllow>,
}

impl Default for Equipment {
    fn default() -> Self {
        Equipment { name: "Equipment".into(), kind: EquipKind::Turbine, nozzles: vec![], reference: 0, shaft: [1.0, 0.0, 0.0], factor: 1.0, coef: Coef::default(), pump: vec![] }
    }
}

/// One line of an equipment check: what, the value and its allowable (in `unit`), and the ratio.
#[derive(Clone, Debug, Serialize)]
pub struct Row {
    pub what: String,
    pub unit: &'static str,
    pub value: f64,
    pub allow: f64,
    pub ratio: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct EquipCheck {
    pub name: String,
    pub kind: EquipKind,
    pub rows: Vec<Row>,
    pub pass: bool,
}

/// What the piping puts on a nozzle (global force and moment) and where it is.
pub struct NozzleAt {
    pub id: u32,
    pub name: String,
    pub at: V,
    /// Nominal size (inches).
    pub size: f64,
    pub f: V,
    pub m: V,
}

/// The equipment's axes: x along the shaft (made horizontal), y up, z = x × y.
pub fn axes(shaft: [f64; 3]) -> (V, V, V) {
    let h = [shaft[0], 0.0, shaft[2]];
    let x = if norm(h) > 1e-9 { unit(h) } else { [1.0, 0.0, 0.0] };
    let y = [0.0, 1.0, 0.0];
    (x, y, cross(x, y))
}

fn row(what: String, unit: &'static str, value: f64, allow: f64) -> Row {
    let ratio = if allow > 0.0 { value / allow } else { 0.0 };
    Row { what, unit, value, allow, ratio }
}

/// Check one machine for one load case.
pub fn check(e: &Equipment, loads: &[NozzleAt]) -> EquipCheck {
    let mine: Vec<&NozzleAt> = e.nozzles.iter().filter_map(|id| loads.iter().find(|n| n.id == *id)).collect();
    let mut rows = vec![];
    let (x, y, z) = axes(e.shaft);
    let comps = |v: V| [dot(v, x), dot(v, y), dot(v, z)];
    let k = if e.factor > 0.0 { e.factor } else { 1.0 };
    match e.kind {
        EquipKind::Turbine | EquipKind::Compressor => {
            let c = &e.coef;
            for n in &mine {
                let (f, mm) = (norm(n.f) / LBF, norm(n.m) / FT_LBF);
                let de = if n.size <= c.de_cap { n.size } else { (c.de_over[0] + n.size) / c.de_over[1] };
                rows.push(row(format!("{}: {}F + {}M ≤ {}·De (De {:.2} in)", n.name, c.single[0], c.single[1], c.single[2], de), "lb, ft·lb", c.single[0] * f + c.single[1] * mm, c.single[2] * de * k));
            }
            if mine.len() > 1 || !mine.is_empty() {
                let refn = mine.iter().find(|n| n.id == e.reference).or_else(|| mine.iter().max_by(|a, b| a.size.total_cmp(&b.size))).copied();
                if let Some(r) = refn {
                    let (mut fc, mut mc) = ([0.0; 3], [0.0; 3]);
                    for n in &mine {
                        fc = add(fc, n.f);
                        mc = add(mc, add(n.m, cross(sub(n.at, r.at), n.f)));
                    }
                    let dc0 = mine.iter().map(|n| n.size * n.size).sum::<f64>().sqrt();
                    let dc = if dc0 <= c.dc_cap { dc0 } else { (c.dc_over[0] + dc0) / c.dc_over[1] };
                    let (f, mm) = (norm(fc) / LBF, norm(mc) / FT_LBF);
                    rows.push(row(format!("All nozzles at {}: {}Fc + {}Mc ≤ {}·Dc (Dc {:.2} in)", r.name, c.combined[0], c.combined[1], c.combined[2], dc), "lb, ft·lb", c.combined[0] * f + c.combined[1] * mm, c.combined[2] * dc * k));
                    let (cf, cm) = (comps(fc), comps(mc));
                    for (i, name) in ["Fx", "Fy", "Fz", "Mx", "My", "Mz"].iter().enumerate() {
                        let (v, unit) = if i < 3 { (cf[i].abs() / LBF, "lb") } else { (cm[i - 3].abs() / FT_LBF, "ft·lb") };
                        rows.push(row(format!("All nozzles: {name} ≤ {}·Dc", c.comp[i]), unit, v, c.comp[i] * dc * k));
                    }
                }
            }
        }
        EquipKind::Pump => {
            for n in &mine {
                let Some(a) = e.pump.iter().find(|p| p.nozzle == n.id) else { continue };
                let (cf, cm) = (comps(n.f), comps(n.m));
                for (i, name) in ["Fx", "Fy", "Fz"].iter().enumerate() {
                    if a.f[i] > 0.0 {
                        rows.push(row(format!("{}: {name}", n.name), "N", cf[i].abs(), a.f[i] * k));
                    }
                }
                if a.fr > 0.0 {
                    rows.push(row(format!("{}: FR", n.name), "N", norm(n.f), a.fr * k));
                }
                for (i, name) in ["Mx", "My", "Mz"].iter().enumerate() {
                    if a.m[i] > 0.0 {
                        rows.push(row(format!("{}: {name}", n.name), "N·m", cm[i].abs(), a.m[i] * k));
                    }
                }
                if a.mr > 0.0 {
                    rows.push(row(format!("{}: MR", n.name), "N·m", norm(n.m), a.mr * k));
                }
            }
        }
    }
    let pass = rows.iter().all(|r| r.ratio <= 1.0 + 1e-12);
    EquipCheck { name: e.name.clone(), kind: e.kind, rows, pass }
}

/// The nominal size of the pipe at a node (inches), from its library size or its diameter.
pub fn size_at(m: &Model, node: u32) -> f64 {
    let e = m.elements.iter().find(|e| e.from == node || e.to == node);
    let p = e.and_then(|e| m.pipe(if e.to == node { match &e.comp { crate::model::Comp::Reducer { to } => *to, _ => e.pipe } } else { e.pipe }));
    p.map_or(0.0, |p| crate::library::nps_inches(&p.nps).unwrap_or(p.od / INCH))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_criteria_by_hand() {
        // One 6" nozzle with 1000 lb and 2000 ft·lb: 3·1000 + 2000 = 5000 against 500·6 = 3000.
        let n = |id, at, size, f: V, m: V| NozzleAt { id, name: format!("N{id}"), at, size, f: scale(f, LBF), m: scale(m, FT_LBF) };
        let e = Equipment { nozzles: vec![1], ..Default::default() };
        let c = check(&e, &[n(1, [0.0; 3], 6.0, [1000.0, 0.0, 0.0], [0.0, 2000.0, 0.0])]);
        assert!((c.rows[0].value - 5000.0).abs() < 1e-9 && (c.rows[0].allow - 3000.0).abs() < 1e-9 && !c.pass);
        // A 12" nozzle: De = (16 + 12) / 3; a compressor's 1.85 on every allowable.
        let e = Equipment { kind: EquipKind::Compressor, factor: 1.85, nozzles: vec![1], ..Default::default() };
        let c = check(&e, &[n(1, [0.0; 3], 12.0, [100.0, 0.0, 0.0], [0.0; 3])]);
        assert!((c.rows[0].allow - 500.0 * 28.0 / 3.0 * 1.85).abs() < 1e-9);
        // Two nozzles (8" and 6") 1 m apart along X: Dc = √(64 + 36) = 10 → (18 + 10) / 3; the 6"
        // nozzle's 100 lb up at 1 m from the reference makes a moment about Z.
        let e = Equipment { nozzles: vec![1, 2], reference: 1, ..Default::default() };
        let c = check(&e, &[n(1, [0.0; 3], 8.0, [0.0; 3], [0.0; 3]), n(2, [1.0, 0.0, 0.0], 6.0, [0.0, 100.0, 0.0], [0.0; 3])]);
        let comb = c.rows.iter().find(|r| r.what.starts_with("All nozzles at")).unwrap();
        let mz = 100.0 * LBF * 1.0 / FT_LBF;
        assert!((comb.allow - 250.0 * 28.0 / 3.0).abs() < 1e-9 && (comb.value - (2.0 * 100.0 + mz)).abs() < 1e-9, "{comb:?}");
        // Pump: components against the user's table values, times the factor.
        let e = Equipment { kind: EquipKind::Pump, factor: 2.0, nozzles: vec![1], pump: vec![PumpAllow { nozzle: 1, f: [1000.0, 0.0, 0.0], ..Default::default() }], ..Default::default() };
        let c = check(&e, &[NozzleAt { id: 1, name: "S".into(), at: [0.0; 3], size: 6.0, f: [1500.0, 0.0, 0.0], m: [0.0; 3] }]);
        assert_eq!(c.rows.len(), 1);
        assert!((c.rows[0].ratio - 0.75).abs() < 1e-12 && c.pass);
    }
}
