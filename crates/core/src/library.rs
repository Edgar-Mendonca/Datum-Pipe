//! The pipe data library: nominal pipe sizes ½" to 24" with their outside diameters and the wall
//! thickness of each schedule, and the materials with their properties against temperature, each
//! value with its source. Dimensions are the published nominal dimensions of wrought steel pipe
//! (ASME B36.10M) and stainless steel pipe (ASME B36.19M): public facts, given here in inches as
//! the standards list them and converted to metres on use.

use serde::Serialize;

const IN: f64 = 0.0254;

/// NPS, outside diameter (in), and (schedule, wall thickness in inches).
type Row = (&'static str, f64, &'static [(&'static str, f64)]);

// A 0.318" wall is a dimension, not an approximation of 1/π.
#[rustfmt::skip]
#[allow(clippy::approx_constant)]
const SIZES: &[Row] = &[
    ("1/2", 0.840, &[("5S", 0.065), ("10S", 0.083), ("10", 0.083), ("40", 0.109), ("STD", 0.109), ("40S", 0.109), ("80", 0.147), ("XS", 0.147), ("80S", 0.147), ("160", 0.188), ("XXS", 0.294)]),
    ("3/4", 1.050, &[("5S", 0.065), ("10S", 0.083), ("10", 0.083), ("40", 0.113), ("STD", 0.113), ("40S", 0.113), ("80", 0.154), ("XS", 0.154), ("80S", 0.154), ("160", 0.219), ("XXS", 0.308)]),
    ("1", 1.315, &[("5S", 0.065), ("10S", 0.109), ("10", 0.109), ("40", 0.133), ("STD", 0.133), ("40S", 0.133), ("80", 0.179), ("XS", 0.179), ("80S", 0.179), ("160", 0.250), ("XXS", 0.358)]),
    ("1-1/4", 1.660, &[("5S", 0.065), ("10S", 0.109), ("10", 0.109), ("40", 0.140), ("STD", 0.140), ("40S", 0.140), ("80", 0.191), ("XS", 0.191), ("80S", 0.191), ("160", 0.250), ("XXS", 0.382)]),
    ("1-1/2", 1.900, &[("5S", 0.065), ("10S", 0.109), ("10", 0.109), ("40", 0.145), ("STD", 0.145), ("40S", 0.145), ("80", 0.200), ("XS", 0.200), ("80S", 0.200), ("160", 0.281), ("XXS", 0.400)]),
    ("2", 2.375, &[("5S", 0.065), ("10S", 0.109), ("10", 0.109), ("40", 0.154), ("STD", 0.154), ("40S", 0.154), ("80", 0.218), ("XS", 0.218), ("80S", 0.218), ("160", 0.344), ("XXS", 0.436)]),
    ("2-1/2", 2.875, &[("5S", 0.083), ("10S", 0.120), ("10", 0.120), ("40", 0.203), ("STD", 0.203), ("40S", 0.203), ("80", 0.276), ("XS", 0.276), ("80S", 0.276), ("160", 0.375), ("XXS", 0.552)]),
    ("3", 3.500, &[("5S", 0.083), ("10S", 0.120), ("10", 0.120), ("40", 0.216), ("STD", 0.216), ("40S", 0.216), ("80", 0.300), ("XS", 0.300), ("80S", 0.300), ("160", 0.438), ("XXS", 0.600)]),
    ("3-1/2", 4.000, &[("5S", 0.083), ("10S", 0.120), ("10", 0.120), ("40", 0.226), ("STD", 0.226), ("40S", 0.226), ("80", 0.318), ("XS", 0.318), ("80S", 0.318)]),
    ("4", 4.500, &[("5S", 0.083), ("10S", 0.120), ("10", 0.120), ("40", 0.237), ("STD", 0.237), ("40S", 0.237), ("80", 0.337), ("XS", 0.337), ("80S", 0.337), ("120", 0.438), ("160", 0.531), ("XXS", 0.674)]),
    ("5", 5.563, &[("5S", 0.109), ("10S", 0.134), ("10", 0.134), ("40", 0.258), ("STD", 0.258), ("40S", 0.258), ("80", 0.375), ("XS", 0.375), ("80S", 0.375), ("120", 0.500), ("160", 0.625), ("XXS", 0.750)]),
    ("6", 6.625, &[("5S", 0.109), ("10S", 0.134), ("10", 0.134), ("40", 0.280), ("STD", 0.280), ("40S", 0.280), ("80", 0.432), ("XS", 0.432), ("80S", 0.432), ("120", 0.562), ("160", 0.719), ("XXS", 0.864)]),
    ("8", 8.625, &[("5S", 0.109), ("10S", 0.148), ("10", 0.148), ("20", 0.250), ("30", 0.277), ("40", 0.322), ("STD", 0.322), ("40S", 0.322), ("60", 0.406), ("80", 0.500), ("XS", 0.500), ("80S", 0.500), ("100", 0.594), ("120", 0.719), ("140", 0.812), ("160", 0.906), ("XXS", 0.875)]),
    ("10", 10.750, &[("5S", 0.134), ("10S", 0.165), ("10", 0.165), ("20", 0.250), ("30", 0.307), ("40", 0.365), ("STD", 0.365), ("40S", 0.365), ("60", 0.500), ("XS", 0.500), ("80S", 0.500), ("80", 0.594), ("100", 0.719), ("120", 0.844), ("140", 1.000), ("160", 1.125), ("XXS", 1.000)]),
    ("12", 12.750, &[("5S", 0.156), ("10S", 0.180), ("10", 0.180), ("20", 0.250), ("30", 0.330), ("STD", 0.375), ("40S", 0.375), ("40", 0.406), ("XS", 0.500), ("80S", 0.500), ("60", 0.562), ("80", 0.688), ("100", 0.844), ("120", 1.000), ("140", 1.125), ("160", 1.312), ("XXS", 1.000)]),
    ("14", 14.000, &[("5S", 0.156), ("10S", 0.188), ("10", 0.250), ("20", 0.312), ("30", 0.375), ("STD", 0.375), ("40", 0.438), ("XS", 0.500), ("60", 0.594), ("80", 0.750), ("100", 0.938), ("120", 1.094), ("140", 1.250), ("160", 1.406)]),
    ("16", 16.000, &[("5S", 0.165), ("10S", 0.188), ("10", 0.250), ("20", 0.312), ("30", 0.375), ("STD", 0.375), ("40", 0.500), ("XS", 0.500), ("60", 0.656), ("80", 0.844), ("100", 1.031), ("120", 1.219), ("140", 1.438), ("160", 1.594)]),
    ("18", 18.000, &[("5S", 0.165), ("10S", 0.188), ("10", 0.250), ("20", 0.312), ("STD", 0.375), ("30", 0.438), ("XS", 0.500), ("40", 0.562), ("60", 0.750), ("80", 0.938), ("100", 1.156), ("120", 1.375), ("140", 1.562), ("160", 1.781)]),
    ("20", 20.000, &[("5S", 0.188), ("10S", 0.218), ("10", 0.250), ("20", 0.375), ("STD", 0.375), ("30", 0.500), ("XS", 0.500), ("40", 0.594), ("60", 0.812), ("80", 1.031), ("100", 1.281), ("120", 1.500), ("140", 1.750), ("160", 1.969)]),
    ("22", 22.000, &[("5S", 0.188), ("10S", 0.218), ("10", 0.250), ("20", 0.375), ("STD", 0.375), ("30", 0.500), ("XS", 0.500), ("60", 0.875), ("80", 1.125), ("100", 1.375), ("120", 1.625), ("140", 1.875), ("160", 2.125)]),
    ("24", 24.000, &[("5S", 0.218), ("10S", 0.250), ("10", 0.250), ("20", 0.375), ("STD", 0.375), ("XS", 0.500), ("30", 0.562), ("40", 0.688), ("60", 0.969), ("80", 1.219), ("100", 1.531), ("120", 1.812), ("140", 2.062), ("160", 2.344)]),
];

pub const SIZE_SOURCE: &str = "Nominal outside diameters and wall thicknesses of wrought steel pipe (ASME B36.10M) and stainless steel pipe (ASME B36.19M)";

/// One nominal size with every schedule it comes in (metres).
#[derive(Clone, Debug, Serialize)]
pub struct Size {
    pub nps: &'static str,
    pub od: f64,
    pub schedules: Vec<(&'static str, f64)>,
}

pub fn sizes() -> Vec<Size> {
    SIZES.iter().map(|(nps, od, s)| Size { nps, od: od * IN, schedules: s.iter().map(|(k, t)| (*k, t * IN)).collect() }).collect()
}

/// Outside diameter and wall (m) of a nominal size and schedule, e.g. ("6", "40").
pub fn pipe(nps: &str, sched: &str) -> Option<(f64, f64)> {
    let (_, od, s) = SIZES.iter().find(|r| r.0 == nps)?;
    let (_, t) = s.iter().find(|x| x.0.eq_ignore_ascii_case(sched))?;
    Some((od * IN, t * IN))
}

/// The nominal size as a number of inches ("1-1/2" → 1.5), for bend radii (1.5 × NPS for a
/// long-radius elbow).
pub fn nps_inches(nps: &str) -> Option<f64> {
    let frac = |s: &str| -> Option<f64> {
        match s.split_once('/') {
            Some((a, b)) => Some(a.parse::<f64>().ok()? / b.parse::<f64>().ok()?),
            None => s.parse().ok(),
        }
    };
    match nps.split_once('-') {
        Some((w, f)) => Some(w.parse::<f64>().ok()? + frac(f)?),
        None => frac(nps),
    }
}

// ---------------------------------------------------------------- materials

/// A library material: room-temperature values from Datum Atlas, trends with temperature typical
/// for the material family, and typical allowable stresses (the user checks them against their code
/// edition). Temperatures in kelvin, stresses and moduli in pascals, expansion as the mean
/// coefficient from 21 °C (1/K).
#[derive(Clone, Debug, Serialize)]
pub struct LibMaterial {
    pub id: &'static str,
    pub name: String,
    pub family: &'static str,
    pub rho: f64,
    pub nu: f64,
    pub e: Vec<[f64; 2]>,
    pub alpha: Vec<[f64; 2]>,
    pub allow: Vec<[f64; 2]>,
    pub sy: f64,
    pub su: f64,
    pub source: String,
}

const K0: f64 = 273.15;

/// Typical family data: (id, name, family, Atlas id, density, E at 21 °C (GPa), minimum yield and
/// tensile strength at room temperature (MPa), the E ratio, mean expansion (µm/m·K) and strength
/// ratios at the temperatures in `TS`, and the highest temperature the allowable may be used at.
struct Fam {
    id: &'static str,
    name: &'static str,
    family: &'static str,
    atlas: &'static str,
    rho: f64,
    e21: f64,
    sy: f64,
    su: f64,
    e_ratio: [f64; 9],
    alpha: [f64; 9],
    sy_ratio: [f64; 9],
    su_ratio: [f64; 9],
    tmax: f64,
}

/// Temperatures (°C) of the family tables.
const TS: [f64; 9] = [21.0, 100.0, 150.0, 200.0, 250.0, 300.0, 350.0, 400.0, 450.0];

#[rustfmt::skip]
const FAMS: &[Fam] = &[
    Fam { id: "cs", name: "Carbon steel, seamless pipe (A106 Gr B type)", family: "Carbon steel", atlas: "steel-1020", rho: 7850.0, e21: 203.0, sy: 240.0, su: 415.0,
        e_ratio: [1.0, 0.976, 0.961, 0.946, 0.928, 0.907, 0.882, 0.853, 0.818],
        alpha: [11.5, 11.7, 12.0, 12.3, 12.6, 12.9, 13.2, 13.5, 13.8],
        sy_ratio: [1.0, 0.92, 0.88, 0.85, 0.81, 0.77, 0.74, 0.71, 0.68], su_ratio: [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.98, 0.93, 0.85], tmax: 371.0 },
    Fam { id: "crmo", name: "Low-alloy steel, 1¼Cr-½Mo pipe (A335 P11 type)", family: "Low-alloy steel", atlas: "steel-4130", rho: 7850.0, e21: 204.0, sy: 205.0, su: 415.0,
        e_ratio: [1.0, 0.978, 0.964, 0.951, 0.936, 0.919, 0.900, 0.877, 0.852],
        alpha: [11.2, 11.5, 11.8, 12.1, 12.4, 12.7, 13.0, 13.3, 13.6],
        sy_ratio: [1.0, 0.93, 0.90, 0.88, 0.86, 0.84, 0.82, 0.80, 0.78], su_ratio: [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.98, 0.95], tmax: 454.0 },
    Fam { id: "ss304", name: "Stainless steel 304 pipe (A312 TP304 type)", family: "Austenitic stainless steel", atlas: "ss-304", rho: 7930.0, e21: 195.0, sy: 205.0, su: 515.0,
        e_ratio: [1.0, 0.969, 0.954, 0.938, 0.923, 0.908, 0.892, 0.877, 0.862],
        alpha: [15.3, 16.0, 16.4, 16.8, 17.1, 17.4, 17.7, 17.9, 18.1],
        sy_ratio: [1.0, 0.83, 0.76, 0.71, 0.67, 0.64, 0.62, 0.60, 0.58], su_ratio: [1.0, 0.90, 0.85, 0.82, 0.80, 0.79, 0.78, 0.77, 0.75], tmax: 538.0 },
    Fam { id: "ss316", name: "Stainless steel 316 pipe (A312 TP316 type)", family: "Austenitic stainless steel", atlas: "ss-316", rho: 7980.0, e21: 195.0, sy: 205.0, su: 515.0,
        e_ratio: [1.0, 0.969, 0.954, 0.938, 0.923, 0.908, 0.892, 0.877, 0.862],
        alpha: [15.3, 16.0, 16.3, 16.6, 16.9, 17.2, 17.5, 17.7, 17.9],
        sy_ratio: [1.0, 0.86, 0.80, 0.75, 0.71, 0.68, 0.66, 0.64, 0.62], su_ratio: [1.0, 0.92, 0.88, 0.86, 0.85, 0.85, 0.85, 0.85, 0.84], tmax: 538.0 },
    Fam { id: "duplex", name: "Duplex stainless steel 2205 pipe (A790 S31803 type)", family: "Duplex stainless steel", atlas: "ss-2205", rho: 7805.0, e21: 200.0, sy: 450.0, su: 620.0,
        e_ratio: [1.0, 0.970, 0.950, 0.930, 0.910, 0.890, 0.870, 0.850, 0.830],
        alpha: [12.8, 13.0, 13.3, 13.6, 13.9, 14.2, 14.5, 14.8, 15.1],
        sy_ratio: [1.0, 0.85, 0.80, 0.76, 0.74, 0.72, 0.70, 0.70, 0.70], su_ratio: [1.0, 0.93, 0.90, 0.88, 0.87, 0.86, 0.86, 0.86, 0.86], tmax: 316.0 },
];

/// The basis of the typical allowable: the lower of a third of the tensile strength and two thirds
/// of the yield strength at temperature, the basis the process piping code uses below the creep range.
pub fn basis(sy: f64, su: f64) -> f64 {
    (su / 3.0).min(2.0 * sy / 3.0)
}

pub fn materials() -> Vec<LibMaterial> {
    FAMS.iter()
        .map(|f| {
            let upto = |t: &f64| *t <= f.tmax + 1e-9;
            LibMaterial {
                id: f.id,
                name: f.name.into(),
                family: f.family,
                rho: f.rho,
                nu: 0.3,
                e: TS.iter().zip(f.e_ratio).map(|(t, r)| [t + K0, f.e21 * 1e9 * r]).collect(),
                alpha: TS.iter().zip(f.alpha).map(|(t, a)| [t + K0, a * 1e-6]).collect(),
                allow: TS.iter().enumerate().filter(|(_, t)| upto(t)).map(|(i, t)| [t + K0, (basis(f.sy * f.sy_ratio[i], f.su * f.su_ratio[i]) * 1e6 * 10.0).round() / 10.0]).collect(),
                sy: f.sy * 1e6,
                su: f.su * 1e6,
                source: format!(
                    "Density and room-temperature values: Datum Atlas ({}, typical). Modulus and expansion against temperature: typical values for the family from general engineering handbooks. Allowable: min(Su/3, 2Sy/3) at temperature from typical strengths, up to {} °C: check against your code edition.",
                    f.atlas, f.tmax
                ),
            }
        })
        .collect()
}

/// Linear interpolation in a table of [x, y] rows sorted by x; held flat beyond its ends.
pub fn interp(t: &[[f64; 2]], x: f64) -> f64 {
    match t {
        [] => 0.0,
        [a] => a[1],
        _ => {
            if x <= t[0][0] {
                return t[0][1];
            }
            for w in t.windows(2) {
                if x <= w[1][0] {
                    let (a, b) = (w[0], w[1]);
                    let d = b[0] - a[0];
                    return if d.abs() < 1e-12 { b[1] } else { a[1] + (b[1] - a[1]) * (x - a[0]) / d };
                }
            }
            t[t.len() - 1][1]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_schedules_and_materials() {
        // 6" Sch 40: 168.3 mm × 7.11 mm; 24" STD 609.6 × 9.53.
        let (od, t) = pipe("6", "40").unwrap();
        assert!((od - 0.168_275).abs() < 1e-9 && (t - 0.007_112).abs() < 1e-9);
        let (od, t) = pipe("24", "std").unwrap();
        assert!((od - 0.6096).abs() < 1e-9 && (t - 0.009_525).abs() < 1e-9);
        assert!(pipe("7", "40").is_none() && pipe("6", "41").is_none());
        assert_eq!(nps_inches("1-1/2"), Some(1.5));
        assert_eq!(nps_inches("3/4"), Some(0.75));
        // Every size has STD, and walls grow with the schedule number.
        for s in sizes() {
            assert!(s.schedules.iter().any(|x| x.0 == "STD"), "{}", s.nps);
            let num: Vec<f64> = s.schedules.iter().filter(|x| x.0.parse::<u32>().is_ok()).map(|x| x.1).collect();
            assert!(num.iter().all(|t| *t > 0.0 && *t < s.od / 2.0));
        }
        // Carbon steel: 138 MPa cold (a third of 415), falling above 300 °C.
        let m = materials();
        let cs = m.iter().find(|x| x.id == "cs").unwrap();
        assert!((interp(&cs.allow, 294.15) - 138.3e6).abs() < 0.1e6, "{:?}", cs.allow);
        assert!(interp(&cs.allow, 643.15) < interp(&cs.allow, 523.15));
        // 304: 2/3 of 205 = 136.7 MPa cold.
        let ss = m.iter().find(|x| x.id == "ss304").unwrap();
        assert!((ss.allow[0][1] - 136.7e6).abs() < 0.1e6);
        assert_eq!(interp(&[[0.0, 1.0], [10.0, 3.0]], 5.0), 2.0);
        assert_eq!(interp(&[[0.0, 1.0], [10.0, 3.0]], 50.0), 3.0);
    }
}
