//! Buried pipe (0.5.4): the soil around a buried pipe as springs along it, each elastic up to the
//! soil's ultimate resistance and then yielding at that load (elastic, perfectly plastic). The
//! ultimate resistances per length of pipe come from classical soil mechanics, worked out here
//! from first principles; every one can be replaced by your own value:
//!
//! - Along the pipe: the friction of the soil pressing on its surface, the vertical pressure γH and
//!   the horizontal pressure K0 γH averaged round the pipe, K0 = 1 − sin φ (Jaky, 1944), with the
//!   interface friction angle f φ; plus adhesion α c in clay: Tu = π D (γ H (1 + K0)/2 tan(f φ) + α c).
//! - Sideways: the passive less the active Rankine pressure (Rankine, 1857) on the pipe's
//!   projected face at depth H, Kp = tan²(45° + φ/2), Ka = tan²(45° − φ/2):
//!   Pu = D (γ H (Kp − Ka) + 2 c (√Kp + √Ka)).
//! - Upwards: the weight of the soil block above the pipe plus the shear on its two vertical sides
//!   (the vertical slip surface model): Qu = γ H D + K0 γ H² tan φ + 2 c H.
//! - Downwards: the bearing capacity of a strip at depth H (Prandtl, 1920; Reissner, 1924; Vesić,
//!   1973): Qd = D (c Nc + γ H Nq + γ D Nγ / 2), Nq = e^(π tan φ) tan²(45° + φ/2),
//!   Nc = (Nq − 1) cot φ (π + 2 for φ = 0), Nγ = 2 (Nq + 1) tan φ.
//!
//! The yield movements (how far the pipe moves before the soil yields) are inputs with typical
//! values. Soil properties vary a lot: take them from the site's soil report.

use serde::{Deserialize, Serialize};

/// One soil: its properties, the pipe's cover (depth to its centre line), and the yield movements.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Soil {
    pub id: u32,
    pub name: String,
    /// Depth from the ground surface to the pipe's centre line (m).
    pub depth: f64,
    /// Unit weight γ (N/m³), friction angle φ (degrees), cohesion c (Pa).
    pub gamma: f64,
    pub phi: f64,
    pub c: f64,
    /// Interface friction factor f (the pipe surface's friction angle is f φ) and adhesion factor α.
    pub f: f64,
    pub alpha: f64,
    /// Movements at which the soil yields: along, across, up, down (m).
    pub y_axial: f64,
    pub y_lateral: f64,
    pub y_up: f64,
    pub y_down: f64,
    /// Your own ultimate resistances per length (N/m) along, across, up and down; 0 calculates them.
    pub ult: [f64; 4],
}

impl Default for Soil {
    fn default() -> Self {
        Soil { id: 0, name: "Sand".into(), depth: 1.5, gamma: 18_000.0, phi: 30.0, c: 0.0, f: 0.8, alpha: 0.5, y_axial: 0.005, y_lateral: 0.04, y_up: 0.015, y_down: 0.02, ult: [0.0; 4] }
    }
}

impl Soil {
    pub fn valid(&self) -> bool {
        let pos = |v: f64| v > 0.0 && v.is_finite();
        let nonneg = |v: f64| v >= 0.0 && v.is_finite();
        pos(self.depth) && pos(self.gamma) && nonneg(self.phi) && self.phi < 60.0 && nonneg(self.c) && (0.0..=1.0).contains(&self.f) && (0.0..=1.0).contains(&self.alpha) && pos(self.y_axial) && pos(self.y_lateral) && pos(self.y_up) && pos(self.y_down) && self.ult.iter().all(|v| nonneg(*v)) && (self.phi > 0.0 || self.c > 0.0 || self.ult.iter().all(|v| *v > 0.0))
    }

    /// The ultimate resistances per length (N/m) of a pipe of outside diameter `d`: along, across,
    /// up and down; your own values where given.
    pub fn ultimate(&self, d: f64) -> [f64; 4] {
        let pi = std::f64::consts::PI;
        let (h, g, c) = (self.depth, self.gamma, self.c);
        let phi = self.phi.to_radians();
        let k0 = 1.0 - phi.sin();
        let kp = (pi / 4.0 + phi / 2.0).tan().powi(2);
        let ka = (pi / 4.0 - phi / 2.0).tan().powi(2);
        let nq = (pi * phi.tan()).exp() * kp;
        let nc = if phi > 1e-9 { (nq - 1.0) / phi.tan() } else { pi + 2.0 };
        let ng = 2.0 * (nq + 1.0) * phi.tan();
        let calc = [
            pi * d * (g * h * (1.0 + k0) / 2.0 * (self.f * phi).tan() + self.alpha * c),
            d * (g * h * (kp - ka) + 2.0 * c * (kp.sqrt() + ka.sqrt())),
            g * h * d + k0 * g * h * h * phi.tan() + 2.0 * c * h,
            d * (c * nc + g * h * nq + g * d * ng / 2.0),
        ];
        std::array::from_fn(|i| if self.ult[i] > 0.0 { self.ult[i] } else { calc[i] })
    }

    /// The yield movements along, across, up and down (m).
    pub fn yields(&self) -> [f64; 4] {
        [self.y_axial, self.y_lateral, self.y_up, self.y_down]
    }
}

/// The soil's load on one buried point in one case: the force on the pipe (N, global) and whether
/// the soil has yielded there in any direction.
#[derive(Clone, Debug, Serialize)]
pub struct SoilLoad {
    pub element: u32,
    pub node: Option<u32>,
    pub point: usize,
    pub force: [f64; 3],
    pub yielded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resistances_by_hand() {
        // Sand: γ 18 kN/m³, φ 30°, H 1.5 m, D 0.2 m, f 0.8, no cohesion.
        let s = Soil { depth: 1.5, gamma: 18e3, phi: 30.0, c: 0.0, f: 0.8, ..Soil::default() };
        let u = s.ultimate(0.2);
        let (k0, kp, ka) = (0.5, 3.0, 1.0 / 3.0);
        let pi = std::f64::consts::PI;
        assert!((u[0] - pi * 0.2 * 18e3 * 1.5 * (1.0 + k0) / 2.0 * (24f64.to_radians()).tan()).abs() < 1e-6, "{}", u[0]);
        assert!((u[1] - 0.2 * 18e3 * 1.5 * (kp - ka)).abs() < 1e-6, "{}", u[1]);
        assert!((u[2] - (18e3 * 1.5 * 0.2 + k0 * 18e3 * 2.25 * (30f64.to_radians()).tan())).abs() < 1e-6);
        // Nq for 30° is 18.40 (the classical value).
        let nq = (pi * (30f64.to_radians()).tan()).exp() * kp;
        assert!((nq - 18.401).abs() < 1e-3, "{nq}");
        // Clay, φ = 0: Nc = π + 2 and the sideways value 4 c D.
        let clay = Soil { phi: 0.0, c: 25e3, ..Soil::default() };
        let u = clay.ultimate(0.3);
        assert!((u[1] - 4.0 * 25e3 * 0.3).abs() < 1e-6, "{}", u[1]);
        assert!((u[3] - 0.3 * (25e3 * (pi + 2.0) + 18e3 * 1.5)).abs() < 1e-6, "{}", u[3]);
        // Your own values replace the calculated ones.
        let own = Soil { ult: [1.0, 2.0, 3.0, 4.0], ..Soil::default() };
        assert_eq!(own.ultimate(0.2), [1.0, 2.0, 3.0, 4.0]);
    }
}
