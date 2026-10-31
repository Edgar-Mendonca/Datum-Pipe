//! Flange leakage by the equivalent pressure method (M. W. Kellogg Company, Design of Piping
//! Systems, 1956): the axial force F and bending moment M at a flanged joint act on the gasket like
//! an extra pressure P_eq = 16 M / (π G³) + 4 F / (π G²), G the gasket's reaction diameter; the
//! design pressure plus P_eq is compared with the flange's pressure rating at temperature. G and the
//! rating are the user's inputs from their flange data (no standard's tables are reproduced). Axial
//! compression is counted as tension, which is on the safe side.

use serde::{Deserialize, Serialize};

/// A flanged joint to check: the rigid flange element, the gasket reaction diameter (m) and the
/// pressure rating at the operating temperature (Pa).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Flange {
    pub element: u32,
    pub gasket: f64,
    pub rating: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct FlangeCheck {
    pub element: u32,
    /// Where on the joint the loads are largest: the point's label.
    pub at: String,
    pub force: f64,
    pub moment: f64,
    pub pressure: f64,
    pub equivalent: f64,
    pub rating: f64,
    pub ratio: f64,
    pub pass: bool,
}

/// The equivalent pressure of an axial force and a bending moment on a gasket of diameter g.
pub fn equivalent(force: f64, moment: f64, g: f64) -> f64 {
    let pi = std::f64::consts::PI;
    16.0 * moment.abs() / (pi * g.powi(3)) + 4.0 * force.abs() / (pi * g * g)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_equivalent_pressure_by_hand() {
        // M = 10 kN·m, F = 20 kN on G = 0.2 m: 16e4/(π·0.008) + 8e4/(π·0.04) = 6.366e6 + 0.637e6 Pa.
        let p = super::equivalent(20e3, 10e3, 0.2);
        assert!((p - (16.0 * 10e3 / (std::f64::consts::PI * 0.008) + 4.0 * 20e3 / (std::f64::consts::PI * 0.04))).abs() < 1e-6);
        assert!((p - 7.002817e6).abs() < 10.0, "{p}");
    }
}
