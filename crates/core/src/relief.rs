//! Relief valve thrust: the reaction of a gas or vapour discharging to the open air through the end
//! of the outlet pipe. From the conservation of momentum, the force on the pipe at the exit is the
//! momentum flux plus the exit pressure above the surroundings over the exit area,
//! F = ṁ V + (P − Pa) A. With an ideal gas reaching sound speed at the exit (choked flow) from a
//! relieving temperature T (stagnation) the exit temperature is 2T/(k + 1), so
//! V = √(2 k R T / ((k + 1) M)) and P = (ṁ / A) √(2 R T / (k (k + 1) M)), R = 8314.46 J/(kmol·K).
//! This is the same result as the published open-discharge reaction equation of API RP 520 Part II
//! (in US units, W/366 · √(kT/((k + 1)M)) + A P). Where the exit pressure works out below the
//! surroundings the exit is not choked: the pressure term is then 0 and the sound speed is kept,
//! which overstates the thrust. The force is multiplied by a dynamic load factor (an input,
//! typically up to 2 for a valve that opens quickly: check against your code edition).

use serde::{Deserialize, Serialize};

/// The universal gas constant, J/(kmol·K).
pub const R: f64 = 8_314.462_618;

/// A relief valve discharging to the open air at the open end `node` of its outlet pipe.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Relief {
    pub name: String,
    /// The node at the outlet pipe's open end.
    pub node: u32,
    /// The relieving mass flow (kg/s).
    pub flow: f64,
    /// The ratio of specific heats, the relieving temperature (K) and the molar mass (kg/kmol).
    pub k: f64,
    pub temp: f64,
    pub mw: f64,
    /// The exit flow area (m²); 0 takes the bore of the pipe at the node.
    pub area: f64,
    /// The direction the gas leaves in; all 0 takes the pipe's own direction at the open end.
    pub dir: [f64; 3],
    /// The dynamic load factor.
    pub dlf: f64,
    /// The pressure of the surroundings (Pa).
    pub ambient: f64,
}

impl Default for Relief {
    fn default() -> Self {
        Relief { name: "PSV".into(), node: 0, flow: 1.0, k: 1.3, temp: 273.15 + 200.0, mw: 18.015, area: 0.0, dir: [0.0; 3], dlf: 2.0, ambient: 101_325.0 }
    }
}

/// The discharge at the exit: velocity (m/s), static pressure (Pa absolute), whether the exit is
/// choked, and the thrust before and after the dynamic load factor (N).
#[derive(Clone, Debug, Serialize)]
pub struct Thrust {
    pub velocity: f64,
    pub pressure: f64,
    pub choked: bool,
    pub stat: f64,
    pub thrust: f64,
}

/// The thrust of a relief valve with exit area `a` (m²).
pub fn thrust(r: &Relief, a: f64) -> Thrust {
    let (k, t, mw) = (r.k, r.temp, r.mw);
    let velocity = (2.0 * k * R * t / ((k + 1.0) * mw)).sqrt();
    let pressure = r.flow / a * (2.0 * R * t / (k * (k + 1.0) * mw)).sqrt();
    let choked = pressure > r.ambient;
    let stat = r.flow * velocity + (pressure - r.ambient).max(0.0) * a;
    Thrust { velocity, pressure, choked, stat, thrust: r.dlf * stat }
}

/// One relief valve's result: the thrust, where it acts and in which direction it pushes the pipe.
#[derive(Clone, Debug, Serialize)]
pub struct ReliefResult {
    pub name: String,
    pub node: u32,
    pub area: f64,
    pub velocity: f64,
    pub pressure: f64,
    pub choked: bool,
    pub stat: f64,
    pub thrust: f64,
    pub dlf: f64,
    /// The force on the pipe (N, global).
    pub force: [f64; 3],
}

impl Relief {
    pub fn valid(&self) -> bool {
        let fin = |v: f64| v.is_finite();
        self.flow > 0.0 && fin(self.flow) && self.k > 1.0 && self.k < 2.0 && self.temp > 0.0 && fin(self.temp) && self.mw > 0.0 && fin(self.mw) && self.area >= 0.0 && fin(self.area) && self.dir.iter().all(|v| fin(*v)) && self.dlf > 0.0 && self.dlf <= 10.0 && self.ambient >= 0.0 && fin(self.ambient)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agrees_with_the_published_us_form() {
        // 100 000 lb/h of steam (k 1.3, M 18.015) at 400 °F through a 6" Sch 40 bore (28.89 in²),
        // in the published form W/366 · √(kT/((k+1)M)) + A·P2, P2 psig at the exit.
        let (lb, inch, psi) = (0.453_592_37, 0.0254, 6894.757);
        let w = 100_000.0;
        let t_r = 400.0 + 459.67;
        let a_in = 28.89;
        let r = Relief { flow: w * lb / 3600.0, temp: t_r / 1.8, area: a_in * inch * inch, ..Relief::default() };
        let th = thrust(&r, r.area);
        let mom = w / 366.0 * (1.3 * t_r / (2.3 * 18.015)).sqrt();
        let p2 = th.pressure / psi - 14.696;
        let us = mom + a_in * p2.max(0.0);
        let ours = th.stat / (lb * 9.806_65);
        assert!((ours - us).abs() / us < 0.005, "ours {ours} lbf, published form {us} lbf");
        assert!(th.choked);
    }

    #[test]
    fn unchoked_exit_has_no_pressure_term() {
        let r = Relief { flow: 0.01, ..Relief::default() };
        let th = thrust(&r, 0.0186);
        assert!(!th.choked);
        assert!((th.stat - 0.01 * th.velocity).abs() < 1e-9);
        assert!((th.thrust - 2.0 * th.stat).abs() < 1e-9);
    }
}
