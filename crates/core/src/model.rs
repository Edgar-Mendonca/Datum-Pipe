//! The piping model, the way piping engineers think about it: an element table (From node, To node,
//! DX, DY, DZ, pipe, material, line conditions, component), with supports, nozzles and extra loads
//! at nodes, and the load cases. Everything is in SI base units (m, kg, N, Pa, K); the window
//! converts for display. Y is up. Node positions follow from the element deltas: the first node
//! of each connected piece keeps its coordinates and the others are laid out from it.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// The `.dpipe` format version; older files are migrated when read.
pub const FORMAT: u32 = 1;
/// Kelvin at 0 °C.
pub const K0: f64 = 273.15;
/// A support with stiffness 0 is rigid: this stiffness (N/m and N·m/rad).
pub const RIGID: f64 = 1e12;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// The node number shown everywhere (10, 20, 30 …).
    pub num: u32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Node {
    pub fn p(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

/// How a bend's radius is chosen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BendKind {
    /// 1.5 × the nominal size (long-radius elbow).
    #[default]
    Long,
    /// 1.0 × the nominal size (short-radius elbow).
    Short,
    /// The radius given.
    Custom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TeeKind {
    /// A forged welding tee (ASME B16.9).
    #[default]
    Welding,
    /// A fabricated branch connection without a pad.
    Fabricated,
    /// A fabricated branch with a reinforcing pad.
    Reinforced,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RigidKind {
    #[default]
    Valve,
    /// A pair of welding-neck flanges.
    Flange,
    /// A pair of slip-on flanges.
    Slipon,
    /// Any other rigid part (a strainer, an instrument).
    Rigid,
}

/// The component on an element. A bend or a tee sits at the element's To node.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Comp {
    #[default]
    Pipe,
    /// A bend at the To node, turning into the next element. `r` is the radius for a custom bend.
    Bend { bend: BendKind, r: f64 },
    /// A tee at the To node, where a branch joins. `pad` is the reinforcing pad's thickness.
    Tee { tee: TeeKind, pad: f64 },
    /// A concentric reducer from this element's pipe (at From) to `to` (at To).
    Reducer { to: u32 },
    /// A valve, flanged joint or other rigid part with its weight (kg, without the fluid).
    Rigid { rigid: RigidKind, mass: f64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Element {
    pub id: u32,
    pub from: u32,
    pub to: u32,
    /// DX, DY, DZ from the From node to the To node (m).
    pub d: [f64; 3],
    pub pipe: u32,
    pub material: u32,
    /// The line conditions (temperatures and pressures) this element runs at.
    pub cond: u32,
    pub comp: Comp,
    /// An extra uniform load along the element, downwards (N/m): a walkway, ice, a cable tray.
    pub extra: f64,
}

impl Default for Element {
    fn default() -> Self {
        Element { id: 0, from: 10, to: 20, d: [1.0, 0.0, 0.0], pipe: 0, material: 0, cond: 0, comp: Comp::Pipe, extra: 0.0 }
    }
}

impl Element {
    pub fn length(&self) -> f64 {
        (self.d[0].powi(2) + self.d[1].powi(2) + self.d[2].powi(2)).sqrt()
    }
}

/// A pipe section: its size, allowances, insulation, cladding and contents.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pipe {
    pub id: u32,
    pub name: String,
    /// The nominal size ("6") and schedule ("40") when it came from the library; empty for your own size.
    pub nps: String,
    pub sched: String,
    /// Outside diameter and nominal wall (m).
    pub od: f64,
    pub wall: f64,
    /// Corrosion and erosion allowance (m).
    pub corrosion: f64,
    /// Mill under-tolerance as a fraction of the wall (0.125 for 12.5 %).
    pub mill: f64,
    pub ins_t: f64,
    pub ins_rho: f64,
    pub clad_t: f64,
    pub clad_rho: f64,
    pub fluid_rho: f64,
}

impl Default for Pipe {
    fn default() -> Self {
        Pipe::std(0, "6", "40")
    }
}

impl Pipe {
    /// A library size, without insulation and empty.
    pub fn std(id: u32, nps: &str, sched: &str) -> Pipe {
        let (od, wall) = crate::library::pipe(nps, sched).unwrap_or((0.168_275, 0.007_112));
        Pipe { id, name: format!("{nps}\" Sch {sched}"), nps: nps.into(), sched: sched.into(), od, wall, corrosion: 0.0, mill: 0.125, ins_t: 0.0, ins_rho: 0.0, clad_t: 0.0, clad_rho: 0.0, fluid_rho: 0.0 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Material {
    pub id: u32,
    pub name: String,
    pub rho: f64,
    pub nu: f64,
    /// Elastic modulus against temperature: [K, Pa].
    pub e: Vec<[f64; 2]>,
    /// Mean thermal expansion coefficient from 21 °C against temperature: [K, 1/K].
    pub alpha: Vec<[f64; 2]>,
    /// Allowable stress against temperature: [K, Pa]. Your values: the library fills typical ones.
    pub allow: Vec<[f64; 2]>,
    /// Weld joint quality factor E and weld strength reduction factor W (hoop and minimum wall).
    pub quality: f64,
    pub weld: f64,
    /// The Y coefficient of the minimum wall equation.
    pub y: f64,
    pub source: String,
}

impl Default for Material {
    fn default() -> Self {
        Material::from_lib(0, "cs")
    }
}

impl Material {
    pub fn from_lib(id: u32, lib: &str) -> Material {
        let all = crate::library::materials();
        let m = all.iter().find(|m| m.id == lib).unwrap_or(&all[0]);
        Material { id, name: m.name.clone(), rho: m.rho, nu: m.nu, e: m.e.clone(), alpha: m.alpha.clone(), allow: m.allow.clone(), quality: 1.0, weld: 1.0, y: 0.4, source: m.source.clone() }
    }
}

/// Line conditions: up to three operating temperatures and pressures, and the hydrotest pressure.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Cond {
    pub id: u32,
    pub name: String,
    /// T1–T3 (K) and P1–P3 (Pa, gauge). A temperature equal to the installed temperature, and a
    /// pressure of 0, mean the case is not used by this line.
    pub t: [f64; 3],
    pub p: [f64; 3],
    pub hydro: f64,
}

impl Default for Cond {
    fn default() -> Self {
        Cond { id: 0, name: "Operating".into(), t: [K0 + 21.0; 3], p: [0.0; 3], hydro: 0.0 }
    }
}

/// What a support does.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SupKind {
    /// Holds all six directions.
    #[default]
    Anchor,
    /// Holds the pipe up (+Y). One-way when the support says so (the pipe can lift off), otherwise
    /// it also holds the pipe down (rests in files from before 0.1.7).
    Rest,
    /// Holds the pipe sideways: horizontally across a horizontal pipe, both horizontal directions
    /// on a vertical pipe.
    Guide,
    /// Holds the pipe along its axis.
    Linestop,
    /// Holds one direction (X, Y, Z or any skewed direction).
    Restraint { dir: [f64; 3] },
    /// A translational spring along a direction (N/m).
    Spring { dir: [f64; 3], k: f64 },
    /// A rotational spring about a direction (N·m/rad).
    Rotspring { dir: [f64; 3], k: f64 },
    /// A constant-force support carrying the given load upwards (N).
    Constant { load: f64 },
    /// A variable spring hanger (holds up, +Y), sized by the analysis from the spring table: its
    /// rate `k` (N/m) and cold (installed) load `cold` (N) are filled in on the copy that is solved
    /// (0 in the file). A spring that cannot be found becomes a constant-force support (k = 0).
    Hanger {
        #[serde(default)]
        k: f64,
        #[serde(default)]
        cold: f64,
    },
}

/// One spring size of the spring table: its name, rate (N/m) and working load range (N).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpringSize {
    pub name: String,
    pub k: f64,
    pub min: f64,
    pub max: f64,
}

impl Default for SpringSize {
    fn default() -> Self {
        SpringSize { name: String::new(), k: 1e5, min: 1000.0, max: 2000.0 }
    }
}

/// A generic spring table to start from: three series with working travels of 25, 50 and 100 mm,
/// each of twenty sizes whose top load grows by half each size from 400 N, working from 1/1.8 of
/// that, so the rate is (max − min) / travel. Not any maker's catalogue: replace it with your
/// supplier's sizes.
pub fn typical_springs() -> Vec<SpringSize> {
    let mut out = vec![];
    for (series, travel) in [("A", 0.025), ("B", 0.05), ("C", 0.1)] {
        for i in 0..20 {
            let max = 400.0 * 1.5f64.powi(i);
            let max = (max / 10.0).round() * 10.0;
            let min = (max / 1.8 / 10.0).round() * 10.0;
            out.push(SpringSize { name: format!("{series}-{:02}", i + 1), k: ((max - min) / travel).round(), min, max });
        }
    }
    out
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Support {
    pub id: u32,
    pub node: u32,
    pub kind: SupKind,
    /// Stiffness of a restraint (N/m; 0 = rigid).
    pub k: f64,
    /// Imposed displacements for cases 1–3 (equipment growth, settlement): dx, dy, dz (m) and
    /// rx, ry, rz (rad). Anchors use all six; directional restraints the part along their direction.
    pub disp: [[f64; 6]; 3],
    /// A tag for the support loads table ("PS-101").
    pub tag: String,
    /// A rest that only pushes up: the pipe can lift off it (rests only).
    pub one_way: bool,
    /// Clearance before the support takes load (m): on each side for a guide, line stop or
    /// restraint, below the pipe for a rest.
    pub gap: f64,
    /// Friction coefficient across the support's direction (0 = none): the pipe sticks while the
    /// force needed is within μ × the support's load, and slides against μ × that load beyond.
    pub mu: f64,
}

impl Default for Support {
    fn default() -> Self {
        Support { id: 0, node: 10, kind: SupKind::Anchor, k: 0.0, disp: [[0.0; 6]; 3], tag: String::new(), one_way: false, gap: 0.0, mu: 0.0 }
    }
}

impl Support {
    /// Whether the support's state depends on the solution (one-way, or with a gap): those are
    /// solved by iteration.
    pub fn conditional(&self) -> bool {
        let gappable = matches!(self.kind, SupKind::Rest | SupKind::Guide | SupKind::Linestop | SupKind::Restraint { .. });
        (self.one_way && self.kind == SupKind::Rest) || (gappable && self.gap > 0.0)
    }
    /// Whether the support has friction (rests, guides, line stops and restraints).
    pub fn has_friction(&self) -> bool {
        matches!(self.kind, SupKind::Rest | SupKind::Guide | SupKind::Linestop | SupKind::Restraint { .. }) && self.mu > 0.0
    }
}

/// An equipment nozzle point: the loads there are reported in the nozzle's axes and compared with
/// the allowable loads given (0 = not checked).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Nozzle {
    pub id: u32,
    pub node: u32,
    pub name: String,
    /// The nozzle axis (pointing from the equipment into the pipe); zero = along the pipe.
    pub axis: [f64; 3],
    /// Allowable axial force, shear force, torsion and bending moment (N, N·m).
    pub allow: [f64; 4],
}

impl Default for Nozzle {
    fn default() -> Self {
        Nozzle { id: 0, node: 10, name: "Nozzle".into(), axis: [0.0; 3], allow: [0.0; 4] }
    }
}

/// An extra force and moment at a node, sustained (it acts in every case with weight).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Load {
    pub id: u32,
    pub node: u32,
    pub f: [f64; 3],
    pub m: [f64; 3],
}

impl Default for Load {
    fn default() -> Self {
        Load { id: 0, node: 10, f: [0.0, -1000.0, 0.0], m: [0.0; 3] }
    }
}

/// The kind of load case decides which code check applies.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaseKind {
    /// Sustained: weight and pressure, checked against the hot allowable.
    #[default]
    Sus,
    /// Operating: everything acting at once; no code check (support and nozzle loads).
    Ope,
    /// Displacement stress range: checked against the allowable range.
    Exp,
    /// Hydrotest: weight full of water and the test pressure (loads for information).
    Hyd,
    /// Occasional: the sustained stress plus the stress of the occasional loads (a range from the
    /// operating case), checked against 1.33 Sh.
    Occ,
}

/// How a load case is built: a sum of basic loads, or the range between two other cases.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Recipe {
    /// W weight (pipe, contents, insulation, extra loads, constant supports); `water` fills with
    /// water instead of the contents; `t` thermal case 1–3 (0 none); `d` imposed displacements of
    /// case 1–3; `p` pressure of case 1–3, or 4 for the hydrotest pressure; `wind` and `seis` the
    /// occasional wind and static seismic loads of the settings.
    Sum {
        w: bool,
        water: bool,
        t: u8,
        d: u8,
        p: u8,
        #[serde(default)]
        wind: bool,
        #[serde(default)]
        seis: bool,
    },
    /// The results of case `a` minus case `b` (the expansion range).
    Range { a: u32, b: u32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LoadCase {
    pub id: u32,
    pub name: String,
    pub kind: CaseKind,
    pub recipe: Recipe,
}

impl Default for LoadCase {
    fn default() -> Self {
        LoadCase { id: 0, name: "SUS".into(), kind: CaseKind::Sus, recipe: Recipe::Sum { w: true, water: false, t: 0, d: 0, p: 1, wind: false, seis: false } }
    }
}

/// The title block of the report and the drawing.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    pub project: String,
    pub line: String,
    pub calc: String,
    pub by: String,
    pub checked: String,
    pub rev: String,
    pub client: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The installed (ambient) temperature (K).
    pub ambient: f64,
    /// Gravity (m/s²).
    pub g: f64,
    /// Include the hydrotest case.
    pub hydro: bool,
    /// Use the liberal allowable stress range (adds the unused part of the sustained allowable).
    pub liberal: bool,
    /// Equivalent full displacement cycles over the plant's life (sets the range factor f).
    pub cycles: f64,
    /// Correct bends' flexibility and intensification for pressure.
    pub bend_pressure: bool,
    /// Build the load cases from the model after every edit.
    pub auto_cases: bool,
    /// Add the axial growth of straight pipe under pressure to the operating cases.
    pub pressure_elongation: bool,
    /// Use the effective section modulus Ze = π r² Ts at a tee's branch (earlier code editions);
    /// off: the branch pipe's own section modulus.
    pub branch_ze: bool,
    /// Number new nodes in steps of this.
    pub step: u32,
    /// Wind: the design wind pressure (Pa, 0 = no wind), the shape (drag) factor, and the direction
    /// the wind blows (horizontal).
    pub wind_q: f64,
    pub wind_cd: f64,
    pub wind_dir: [f64; 3],
    /// Static seismic: the acceleration along X, Y and Z as a fraction of g (all 0 = none).
    pub seis: [f64; 3],
    /// Spring hangers: the spring table (in order of preference) and the largest load variation
    /// allowed, |k Δ| / hot load.
    pub springs: Vec<SpringSize>,
    pub variability: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { ambient: K0 + 21.0, g: 9.806_65, hydro: false, liberal: false, cycles: 7000.0, bend_pressure: true, auto_cases: true, pressure_elongation: false, branch_ze: true, step: 10, wind_q: 0.0, wind_cd: 0.7, wind_dir: [1.0, 0.0, 0.0], seis: [0.0; 3], springs: typical_springs(), variability: 0.25 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Model {
    pub format: u32,
    pub name: String,
    pub notes: String,
    /// "SI" or "US": how values are shown and typed.
    pub units: String,
    pub project: Project,
    pub settings: Settings,
    pub nodes: Vec<Node>,
    pub elements: Vec<Element>,
    pub pipes: Vec<Pipe>,
    pub materials: Vec<Material>,
    pub conds: Vec<Cond>,
    pub supports: Vec<Support>,
    pub nozzles: Vec<Nozzle>,
    pub loads: Vec<Load>,
    pub cases: Vec<LoadCase>,
    /// The next id for any item.
    pub next: u32,
}

impl Default for Model {
    fn default() -> Self {
        Model {
            format: FORMAT,
            name: "New piping model".into(),
            notes: String::new(),
            units: "SI".into(),
            project: Project::default(),
            settings: Settings::default(),
            nodes: vec![],
            elements: vec![],
            pipes: vec![Pipe::std(1, "6", "40")],
            materials: vec![Material::from_lib(2, "cs")],
            conds: vec![Cond { id: 3, ..Cond::default() }],
            supports: vec![],
            nozzles: vec![],
            loads: vec![],
            cases: vec![],
            next: 4,
        }
    }
}

impl Model {
    pub fn new() -> Model {
        let mut m = Model::default();
        crate::cases::rebuild(&mut m);
        m
    }

    /// Every id in use, and the largest.
    pub fn max_id(&self) -> u32 {
        let ids = self.elements.iter().map(|x| x.id).chain(self.pipes.iter().map(|x| x.id)).chain(self.materials.iter().map(|x| x.id)).chain(self.conds.iter().map(|x| x.id));
        let ids = ids.chain(self.supports.iter().map(|x| x.id)).chain(self.nozzles.iter().map(|x| x.id)).chain(self.loads.iter().map(|x| x.id)).chain(self.cases.iter().map(|x| x.id));
        ids.max().unwrap_or(0)
    }

    /// Ids handed out by `recommend` and other builders may clash; give them fresh numbers.
    pub fn fix_ids(&mut self) {
        let mut seen = HashSet::new();
        let mut next = self.next.max(self.max_id() + 1);
        for c in &mut self.cases {
            if c.id == 0 || !seen.insert(c.id) {
                c.id = next;
                next += 1;
                seen.insert(c.id);
            }
        }
        self.next = next.max(self.max_id() + 1);
    }

    pub fn node(&self, num: u32) -> Option<&Node> {
        self.nodes.iter().find(|n| n.num == num)
    }
    pub fn pipe(&self, id: u32) -> Option<&Pipe> {
        self.pipes.iter().find(|x| x.id == id)
    }
    pub fn material(&self, id: u32) -> Option<&Material> {
        self.materials.iter().find(|x| x.id == id)
    }
    pub fn cond(&self, id: u32) -> Option<&Cond> {
        self.conds.iter().find(|x| x.id == id)
    }

    /// The node numbers in table order (as they first appear).
    pub fn node_order(&self) -> Vec<u32> {
        let mut seen = HashSet::new();
        let mut out = vec![];
        for e in &self.elements {
            for n in [e.from, e.to] {
                if seen.insert(n) {
                    out.push(n);
                }
            }
        }
        out
    }

    /// Lays the nodes out from the element deltas. The first node of each connected piece keeps its
    /// coordinates (or starts at the origin); every other node is reached through the elements. An
    /// element that closes a loop must end where its To node already is.
    pub fn layout(&mut self) -> Result<(), String> {
        let old: HashMap<u32, [f64; 3]> = self.nodes.iter().map(|n| (n.num, n.p())).collect();
        let order = self.node_order();
        let mut adj: HashMap<u32, Vec<(u32, [f64; 3], u32)>> = HashMap::new();
        for e in &self.elements {
            adj.entry(e.from).or_default().push((e.to, e.d, e.id));
            adj.entry(e.to).or_default().push((e.from, [-e.d[0], -e.d[1], -e.d[2]], e.id));
        }
        let mut pos: HashMap<u32, [f64; 3]> = HashMap::new();
        for &root in &order {
            if pos.contains_key(&root) {
                continue;
            }
            pos.insert(root, old.get(&root).copied().unwrap_or([0.0; 3]));
            let mut stack = vec![root];
            while let Some(a) = stack.pop() {
                let pa = pos[&a];
                for &(b, d, id) in adj.get(&a).map(|v| v.as_slice()).unwrap_or(&[]) {
                    let pb = [pa[0] + d[0], pa[1] + d[1], pa[2] + d[2]];
                    match pos.get(&b) {
                        Some(q) => {
                            let gap = ((q[0] - pb[0]).powi(2) + (q[1] - pb[1]).powi(2) + (q[2] - pb[2]).powi(2)).sqrt();
                            if gap > 1e-6 {
                                let e = self.elements.iter().find(|e| e.id == id).map(|e| format!("{}–{}", e.from, e.to)).unwrap_or_default();
                                return Err(format!("The loop through element {e} does not close: node {b} would be {:.1} mm from where the other elements put it.", gap * 1000.0));
                            }
                        }
                        None => {
                            pos.insert(b, pb);
                            stack.push(b);
                        }
                    }
                }
            }
        }
        self.nodes = order.iter().map(|n| { let p = pos[n]; Node { num: *n, x: p[0], y: p[1], z: p[2] } }).collect();
        Ok(())
    }

    /// Every rule the model must keep. An edit that breaks one is refused.
    pub fn check(&self) -> Result<(), String> {
        let fin = |v: f64| v.is_finite();
        let mut ids = HashSet::new();
        for id in self.elements.iter().map(|x| x.id).chain(self.pipes.iter().map(|x| x.id)).chain(self.materials.iter().map(|x| x.id)).chain(self.conds.iter().map(|x| x.id)).chain(self.supports.iter().map(|x| x.id)).chain(self.nozzles.iter().map(|x| x.id)).chain(self.loads.iter().map(|x| x.id)).chain(self.cases.iter().map(|x| x.id)) {
            if !ids.insert(id) {
                return Err(format!("Two items share the id {id}."));
            }
        }
        if self.next <= self.max_id() {
            return Err("The id counter is behind the items.".into());
        }
        if self.pipes.is_empty() || self.materials.is_empty() || self.conds.is_empty() {
            return Err("A model needs at least one pipe, one material and one set of line conditions.".into());
        }
        let s = &self.settings;
        if !(s.ambient > 0.0 && s.ambient < 2000.0 && fin(s.g) && s.g >= 0.0 && s.cycles >= 1.0 && fin(s.cycles) && s.step >= 1) {
            return Err("The installed temperature, gravity, cycles and node step must be sensible numbers.".into());
        }
        let horiz = (s.wind_dir[0].powi(2) + s.wind_dir[2].powi(2)).sqrt();
        if !(s.wind_q >= 0.0 && fin(s.wind_q) && s.wind_cd > 0.0 && fin(s.wind_cd) && s.wind_dir.iter().all(|v| fin(*v)) && horiz > 1e-6) {
            return Err("Wind needs a pressure of 0 or more, a shape factor above 0 and a horizontal direction.".into());
        }
        if !s.seis.iter().all(|v| fin(*v) && v.abs() <= 10.0) {
            return Err("The seismic factors must be numbers of g between −10 and 10.".into());
        }
        if !(s.variability > 0.0 && s.variability <= 1.0) || s.springs.iter().any(|x| !(x.k > 0.0 && fin(x.k) && x.min >= 0.0 && x.max > x.min && fin(x.max))) {
            return Err("Each spring needs a rate above 0 and a working range from a load to a larger one; the load variation allowed is from 0 to 100 %.".into());
        }
        for p in &self.pipes {
            if !(p.od > 0.0 && fin(p.od) && p.wall > 0.0 && p.wall < p.od / 2.0) {
                return Err(format!("{}: the wall must be thinner than half the outside diameter.", p.name));
            }
            if !(p.corrosion >= 0.0 && p.corrosion < p.wall && (0.0..0.5).contains(&p.mill)) {
                return Err(format!("{}: the corrosion allowance must be less than the wall, and the mill tolerance under 50 %.", p.name));
            }
            for v in [p.ins_t, p.ins_rho, p.clad_t, p.clad_rho, p.fluid_rho] {
                if !(v >= 0.0 && fin(v)) {
                    return Err(format!("{}: insulation, cladding and contents must be 0 or more.", p.name));
                }
            }
        }
        for m in &self.materials {
            if !(m.rho >= 0.0 && fin(m.rho) && m.nu > -1.0 && m.nu < 0.5) || m.e.is_empty() || m.alpha.is_empty() {
                return Err(format!("{}: density, Poisson's ratio, modulus and expansion must be given.", m.name));
            }
            for t in [&m.e, &m.alpha, &m.allow] {
                if t.iter().any(|r| !(r[0] > 0.0 && fin(r[0]) && fin(r[1]))) || t.windows(2).any(|w| w[1][0] <= w[0][0]) {
                    return Err(format!("{}: each table needs rising temperatures above absolute zero.", m.name));
                }
            }
            if m.e.iter().any(|r| r[1] <= 0.0) || m.allow.iter().any(|r| r[1] <= 0.0) {
                return Err(format!("{}: the modulus and allowable stresses must be above 0.", m.name));
            }
            if !(m.quality > 0.0 && m.quality <= 1.0 && m.weld > 0.0 && m.weld <= 1.0 && (0.0..=1.0).contains(&m.y)) {
                return Err(format!("{}: the quality factor E and weld factor W are between 0 and 1.", m.name));
            }
        }
        for c in &self.conds {
            if c.t.iter().chain(c.p.iter()).chain([&c.hydro]).any(|v| !fin(*v)) || c.t.iter().any(|t| *t <= 0.0) || c.p.iter().chain([&c.hydro]).any(|p| *p < -101_325.0) {
                return Err(format!("{}: temperatures must be above absolute zero and pressures above full vacuum.", c.name));
            }
        }
        let mut pairs = HashSet::new();
        for e in &self.elements {
            if e.from == e.to || e.from == 0 || e.to == 0 {
                return Err(format!("Element {}–{} must join two different nodes (numbers from 1).", e.from, e.to));
            }
            if !pairs.insert((e.from.min(e.to), e.from.max(e.to))) {
                return Err(format!("There are two elements between nodes {} and {}.", e.from, e.to));
            }
            if e.d.iter().any(|v| !fin(*v)) || e.length() < 1e-4 {
                return Err(format!("Element {}–{} needs a length (DX, DY, DZ).", e.from, e.to));
            }
            if self.pipe(e.pipe).is_none() || self.material(e.material).is_none() || self.cond(e.cond).is_none() {
                return Err(format!("Element {}–{} uses a pipe, material or line conditions that do not exist.", e.from, e.to));
            }
            if !(e.extra >= 0.0 && fin(e.extra)) {
                return Err(format!("Element {}–{}: the extra load must be 0 or more.", e.from, e.to));
            }
            match &e.comp {
                Comp::Bend { bend: BendKind::Custom, r } if !(*r > 0.0 && fin(*r)) => return Err(format!("The bend at node {} needs a radius.", e.to)),
                Comp::Tee { pad, .. } if !(*pad >= 0.0 && fin(*pad)) => return Err(format!("The tee at node {}: the pad thickness must be 0 or more.", e.to)),
                Comp::Reducer { to } if self.pipe(*to).is_none() => return Err(format!("The reducer {}–{} needs the pipe at its small end.", e.from, e.to)),
                Comp::Rigid { mass, .. } if !(*mass >= 0.0 && fin(*mass)) => return Err(format!("The rigid part {}–{}: its weight must be 0 or more.", e.from, e.to)),
                _ => {}
            }
        }
        let nodes: HashSet<u32> = self.nodes.iter().map(|n| n.num).collect();
        if nodes.len() != self.nodes.len() || self.nodes.iter().any(|n| !(fin(n.x) && fin(n.y) && fin(n.z))) {
            return Err("The node list is damaged.".into());
        }
        for s in &self.supports {
            if !nodes.contains(&s.node) {
                return Err(format!("A support is on node {}, which is not in the model.", s.node));
            }
            let dir_ok = |d: &[f64; 3]| d.iter().all(|v| fin(*v)) && d.iter().map(|v| v * v).sum::<f64>() > 1e-12;
            let ok = match &s.kind {
                SupKind::Restraint { dir } => dir_ok(dir),
                SupKind::Spring { dir, k } | SupKind::Rotspring { dir, k } => dir_ok(dir) && *k > 0.0 && fin(*k),
                SupKind::Constant { load } => fin(*load) && *load >= 0.0,
                SupKind::Hanger { k, cold } => fin(*k) && *k >= 0.0 && fin(*cold) && *cold >= 0.0,
                _ => true,
            };
            if !(s.gap >= 0.0 && fin(s.gap)) {
                return Err(format!("The support at node {}: the gap must be 0 or more.", s.node));
            }
            if !(s.mu >= 0.0 && s.mu <= 2.0) {
                return Err(format!("The support at node {}: the friction coefficient must be between 0 and 2.", s.node));
            }
            if !ok || !(s.k >= 0.0 && fin(s.k)) || s.disp.iter().flatten().any(|v| !fin(*v)) {
                return Err(format!("The support at node {} needs a direction and a stiffness above 0.", s.node));
            }
        }
        for z in &self.nozzles {
            if !nodes.contains(&z.node) || z.axis.iter().chain(z.allow.iter()).any(|v| !fin(*v)) || z.allow.iter().any(|v| *v < 0.0) {
                return Err(format!("The nozzle {} must be on a node of the model, with allowables of 0 or more.", z.name));
            }
        }
        for l in &self.loads {
            if !nodes.contains(&l.node) || l.f.iter().chain(l.m.iter()).any(|v| !fin(*v)) {
                return Err(format!("A load is on node {}, which is not in the model.", l.node));
            }
        }
        let cases: HashSet<u32> = self.cases.iter().map(|c| c.id).collect();
        for c in &self.cases {
            match &c.recipe {
                Recipe::Sum { t, d, p, .. } if *t > 3 || *d > 3 || *p > 4 => return Err(format!("Load case {}: cases are numbered 1 to 3.", c.name)),
                Recipe::Range { a, b } if !cases.contains(a) || !cases.contains(b) || a == b || *a == c.id || *b == c.id => return Err(format!("Load case {}: a range needs two other load cases.", c.name)),
                _ => {}
            }
        }
        // Ranges may only use cases listed before them, so they can be worked out in order.
        let mut before = HashSet::new();
        for c in &self.cases {
            if let Recipe::Range { a, b } = c.recipe {
                if !before.contains(&a) || !before.contains(&b) {
                    return Err(format!("Load case {}: a range must come after the two cases it uses.", c.name));
                }
            }
            before.insert(c.id);
        }
        Ok(())
    }

    /// The bounding box of the nodes: [min, max].
    pub fn bounds(&self) -> Option<([f64; 3], [f64; 3])> {
        let first = self.nodes.first()?;
        let (mut lo, mut hi) = (first.p(), first.p());
        for n in &self.nodes {
            for i in 0..3 {
                lo[i] = lo[i].min(n.p()[i]);
                hi[i] = hi[i].max(n.p()[i]);
            }
        }
        Some((lo, hi))
    }
}

// ---------------------------------------------------------------- reading and writing

pub fn to_json(m: &Model) -> String {
    serde_json::to_string_pretty(m).unwrap_or_default()
}

/// Older files are brought up to the current format here.
fn migrate(v: Value) -> Result<Value, String> {
    let f = v.get("format").and_then(Value::as_u64).unwrap_or(1);
    if f > FORMAT as u64 {
        return Err(format!("This model was saved by a newer Datum Pipe (format {f}). Update Datum Pipe to open it."));
    }
    Ok(v)
}

/// Reads a `.dpipe` file. A damaged file opens as far as possible: lists are read item by item, bad
/// items are left out, settings of the wrong type fall back to the defaults, and the model is
/// repaired until it passes `check`. Returns the warnings that say what was lost.
pub fn from_json(text: &str) -> Result<(Model, Vec<String>), String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("This is not a Datum Pipe model: {e}"))?;
    if !v.is_object() {
        return Err("This is not a Datum Pipe model.".into());
    }
    let mut v = migrate(v)?;
    let mut warn = vec![];
    let lists = ["nodes", "elements", "pipes", "materials", "conds", "supports", "nozzles", "loads", "cases"];
    let mut keep: Vec<(String, Vec<Value>)> = vec![];
    if let Some(o) = v.as_object_mut() {
        for k in lists {
            match o.remove(k) {
                Some(Value::Array(a)) => keep.push((k.into(), a)),
                Some(Value::Null) | None => {}
                Some(_) => warn.push(format!("The {k} list was damaged and was left out.")),
            }
        }
        for (k, ok) in [("settings", o.get("settings").is_none_or(|x| serde_json::from_value::<Settings>(x.clone()).is_ok())), ("project", o.get("project").is_none_or(|x| serde_json::from_value::<Project>(x.clone()).is_ok()))] {
            if !ok {
                o.remove(k);
                warn.push(format!("The {k} were damaged; the defaults are used."));
            }
        }
        let d = serde_json::to_value(Model::default()).unwrap_or_default();
        for (k, dv) in d.as_object().cloned().unwrap_or_default() {
            if lists.contains(&k.as_str()) || k == "settings" || k == "project" {
                continue;
            }
            if let Some(x) = o.get(&k) {
                let bad = std::mem::discriminant(x) != std::mem::discriminant(&dv) && !(x.is_number() && dv.is_number());
                let neg = k == "next" && !x.is_u64();
                if bad || neg {
                    warn.push(format!("The setting \"{k}\" was damaged; the default is used."));
                    o.remove(&k);
                }
            }
        }
    }
    let mut m: Model = serde_json::from_value(v).map_err(|e| format!("The model file is damaged: {e}"))?;
    fn read<T: serde::de::DeserializeOwned>(a: Vec<Value>, what: &str, warn: &mut Vec<String>) -> Vec<T> {
        let n = a.len();
        let out: Vec<T> = a.into_iter().filter_map(|x| serde_json::from_value(x).ok()).collect();
        if out.len() < n {
            warn.push(format!("{} of the {what} could not be read and were left out.", n - out.len()));
        }
        out
    }
    for (k, a) in keep {
        match k.as_str() {
            "nodes" => m.nodes = read(a, "nodes", &mut warn),
            "elements" => m.elements = read(a, "elements", &mut warn),
            "pipes" => m.pipes = read(a, "pipes", &mut warn),
            "materials" => m.materials = read(a, "materials", &mut warn),
            "conds" => m.conds = read(a, "line conditions", &mut warn),
            "supports" => m.supports = read(a, "supports", &mut warn),
            "nozzles" => m.nozzles = read(a, "nozzles", &mut warn),
            "loads" => m.loads = read(a, "loads", &mut warn),
            "cases" => m.cases = read(a, "load cases", &mut warn),
            _ => {}
        }
    }
    m.format = FORMAT;
    repair(&mut m, &mut warn);
    Ok((m, warn))
}

/// Mends a model read from a damaged file until it passes `check`, saying what changed.
fn repair(m: &mut Model, warn: &mut Vec<String>) {
    let d = Model::default();
    let fin = |v: f64| v.is_finite();
    // Ids: unique and below the counter.
    let mut seen = HashSet::new();
    let mut next = m.next.max(1);
    let mut fresh = |id: &mut u32, seen: &mut HashSet<u32>| {
        if *id == 0 || !seen.insert(*id) {
            next = next.max(*id + 1).max(seen.iter().max().copied().unwrap_or(0) + 1);
            *id = next;
            next += 1;
            seen.insert(*id);
        }
    };
    let mut renumbered = HashMap::<u32, u32>::new();
    for p in &mut m.pipes {
        let old = p.id;
        fresh(&mut p.id, &mut seen);
        renumbered.entry(old).or_insert(p.id);
    }
    for x in &mut m.materials {
        fresh(&mut x.id, &mut seen);
    }
    for x in &mut m.conds {
        fresh(&mut x.id, &mut seen);
    }
    for x in &mut m.elements {
        fresh(&mut x.id, &mut seen);
    }
    for x in &mut m.supports {
        fresh(&mut x.id, &mut seen);
    }
    for x in &mut m.nozzles {
        fresh(&mut x.id, &mut seen);
    }
    for x in &mut m.loads {
        fresh(&mut x.id, &mut seen);
    }
    for x in &mut m.cases {
        fresh(&mut x.id, &mut seen);
    }
    m.next = next.max(m.max_id() + 1);
    // Properties: bad ones are dropped, and at least one of each is kept.
    let n0 = m.pipes.len();
    m.pipes.retain(|p| p.od > 0.0 && fin(p.od) && p.wall > 0.0 && p.wall < p.od / 2.0);
    for p in &mut m.pipes {
        if !(p.corrosion >= 0.0 && p.corrosion < p.wall) {
            p.corrosion = 0.0;
        }
        if !(0.0..0.5).contains(&p.mill) {
            p.mill = 0.125;
        }
        for v in [&mut p.ins_t, &mut p.ins_rho, &mut p.clad_t, &mut p.clad_rho, &mut p.fluid_rho] {
            if !(*v >= 0.0 && fin(*v)) {
                *v = 0.0;
            }
        }
    }
    let n1 = m.materials.len();
    m.materials.retain(|x| x.rho >= 0.0 && fin(x.rho) && x.nu > -1.0 && x.nu < 0.5);
    for x in &mut m.materials {
        let lib = Material::from_lib(0, "cs");
        let good = |t: &Vec<[f64; 2]>, pos: bool| !t.is_empty() && t.iter().all(|r| r[0] > 0.0 && fin(r[0]) && fin(r[1]) && (!pos || r[1] > 0.0)) && t.windows(2).all(|w| w[1][0] > w[0][0]);
        if !good(&x.e, true) {
            x.e = lib.e.clone();
        }
        if !good(&x.alpha, false) {
            x.alpha = lib.alpha.clone();
        }
        if !(x.allow.is_empty() || good(&x.allow, true)) {
            x.allow = lib.allow.clone();
        }
        if !(x.quality > 0.0 && x.quality <= 1.0) {
            x.quality = 1.0;
        }
        if !(x.weld > 0.0 && x.weld <= 1.0) {
            x.weld = 1.0;
        }
        if !(0.0..=1.0).contains(&x.y) {
            x.y = 0.4;
        }
    }
    let n2 = m.conds.len();
    m.conds.retain(|c| c.t.iter().chain(c.p.iter()).chain([&c.hydro]).all(|v| fin(*v)) && c.t.iter().all(|t| *t > 0.0) && c.p.iter().chain([&c.hydro]).all(|p| *p >= -101_325.0));
    if m.pipes.len() < n0 || m.materials.len() < n1 || m.conds.len() < n2 {
        warn.push("Some pipes, materials or line conditions had impossible values and were left out.".into());
    }
    for (list_empty, what) in [(m.pipes.is_empty(), 0), (m.materials.is_empty(), 1), (m.conds.is_empty(), 2)] {
        if list_empty {
            let id = m.next;
            m.next += 1;
            match what {
                0 => m.pipes.push(Pipe { id, ..d.pipes[0].clone() }),
                1 => m.materials.push(Material { id, ..d.materials[0].clone() }),
                _ => m.conds.push(Cond { id, ..d.conds[0].clone() }),
            }
        }
    }
    let s = &mut m.settings;
    if !(s.ambient > 0.0 && s.ambient < 2000.0) {
        s.ambient = d.settings.ambient;
    }
    if !(fin(s.g) && s.g >= 0.0) {
        s.g = d.settings.g;
    }
    if !(s.cycles >= 1.0 && fin(s.cycles)) {
        s.cycles = d.settings.cycles;
    }
    if s.step == 0 {
        s.step = 10;
    }
    if !(s.wind_q >= 0.0 && fin(s.wind_q)) {
        s.wind_q = 0.0;
    }
    if !(s.wind_cd > 0.0 && fin(s.wind_cd)) {
        s.wind_cd = d.settings.wind_cd;
    }
    if !(s.wind_dir.iter().all(|v| fin(*v)) && (s.wind_dir[0].powi(2) + s.wind_dir[2].powi(2)).sqrt() > 1e-6) {
        s.wind_dir = d.settings.wind_dir;
    }
    if !s.seis.iter().all(|v| fin(*v) && v.abs() <= 10.0) {
        s.seis = [0.0; 3];
    }
    s.springs.retain(|x| x.k > 0.0 && x.k.is_finite() && x.min >= 0.0 && x.max > x.min && x.max.is_finite());
    if !(s.variability > 0.0 && s.variability <= 1.0) {
        s.variability = 0.25;
    }
    if m.units != "SI" && m.units != "US" {
        m.units = "SI".into();
    }
    // Elements: missing properties fall back to the first; impossible ones are dropped.
    let (p0, m0, c0) = (m.pipes[0].id, m.materials[0].id, m.conds[0].id);
    let pipes: HashSet<u32> = m.pipes.iter().map(|x| x.id).collect();
    let mats: HashSet<u32> = m.materials.iter().map(|x| x.id).collect();
    let conds: HashSet<u32> = m.conds.iter().map(|x| x.id).collect();
    let n = m.elements.len();
    let mut pairs = HashSet::new();
    m.elements.retain(|e| e.from != e.to && e.from != 0 && e.to != 0 && e.d.iter().all(|v| fin(*v)) && e.length() >= 1e-4 && pairs.insert((e.from.min(e.to), e.from.max(e.to))));
    if m.elements.len() < n {
        warn.push(format!("{} elements without a length, or repeating another, were left out.", n - m.elements.len()));
    }
    for e in &mut m.elements {
        if !pipes.contains(&e.pipe) {
            e.pipe = renumbered.get(&e.pipe).copied().filter(|p| pipes.contains(p)).unwrap_or(p0);
        }
        if !mats.contains(&e.material) {
            e.material = m0;
        }
        if !conds.contains(&e.cond) {
            e.cond = c0;
        }
        if !(e.extra >= 0.0 && fin(e.extra)) {
            e.extra = 0.0;
        }
        let bad = match &e.comp {
            Comp::Bend { bend: BendKind::Custom, r } => !(*r > 0.0 && fin(*r)),
            Comp::Tee { pad, .. } => !(*pad >= 0.0 && fin(*pad)),
            Comp::Reducer { to } => !pipes.contains(to),
            Comp::Rigid { mass, .. } => !(*mass >= 0.0 && fin(*mass)),
            _ => false,
        };
        if bad {
            warn.push(format!("The component on element {}–{} was damaged and became straight pipe.", e.from, e.to));
            e.comp = Comp::Pipe;
        }
    }
    // Positions: keep each piece's first node, drop elements that break a loop.
    m.nodes.retain(|n| fin(n.x) && fin(n.y) && fin(n.z));
    let mut uniq = HashSet::new();
    m.nodes.retain(|n| uniq.insert(n.num));
    while let Err(e) = m.layout() {
        warn.push(format!("{e} The last element of the loop was left out."));
        let Some(bad) = loop_breaker(m) else { break };
        m.elements.retain(|x| x.id != bad);
    }
    let nodes: HashSet<u32> = m.nodes.iter().map(|n| n.num).collect();
    let (a, b, c) = (m.supports.len(), m.nozzles.len(), m.loads.len());
    m.supports.retain(|s| nodes.contains(&s.node));
    m.nozzles.retain(|z| nodes.contains(&z.node) && z.axis.iter().chain(z.allow.iter()).all(|v| fin(*v)) && z.allow.iter().all(|v| *v >= 0.0));
    m.loads.retain(|l| nodes.contains(&l.node) && l.f.iter().chain(l.m.iter()).all(|v| fin(*v)));
    for s in &mut m.supports {
        let unit = |d: &mut [f64; 3]| {
            if !(d.iter().all(|v| fin(*v)) && d.iter().map(|v| v * v).sum::<f64>() > 1e-12) {
                *d = [0.0, 1.0, 0.0];
            }
        };
        match &mut s.kind {
            SupKind::Restraint { dir } => unit(dir),
            SupKind::Spring { dir, k } | SupKind::Rotspring { dir, k } => {
                unit(dir);
                if !(*k > 0.0 && fin(*k)) {
                    *k = 1e6;
                }
            }
            SupKind::Constant { load } if !(fin(*load) && *load >= 0.0) => *load = 0.0,
            SupKind::Hanger { k, cold } if !(fin(*k) && *k >= 0.0 && fin(*cold) && *cold >= 0.0) => {
                *k = 0.0;
                *cold = 0.0;
            }
            _ => {}
        }
        if !(s.k >= 0.0 && fin(s.k)) {
            s.k = 0.0;
        }
        for v in s.disp.iter_mut().flatten() {
            if !fin(*v) {
                *v = 0.0;
            }
        }
    }
    if m.supports.len() + m.nozzles.len() + m.loads.len() < a + b + c {
        warn.push("Supports, nozzles or loads on nodes that are not in the model were left out.".into());
    }
    // Load cases: rebuilt when they cannot be worked out.
    let ok_cases = {
        let mut t2 = m.clone();
        t2.next = t2.next.max(t2.max_id() + 1);
        t2.check().is_ok()
    };
    if !ok_cases {
        if !m.cases.is_empty() {
            warn.push("The load cases were damaged and were built again from the model.".into());
        }
        m.cases.clear();
        crate::cases::rebuild(m);
    }
    m.next = m.next.max(m.max_id() + 1);
    m.fix_ids();
    if let Err(e) = m.check() {
        // Anything still wrong: start the lists that break the rules afresh.
        warn.push(format!("The file still broke a rule ({e}); its supports, loads and cases were rebuilt."));
        m.supports.clear();
        m.nozzles.clear();
        m.loads.clear();
        m.cases.clear();
        crate::cases::rebuild(m);
    }
}

/// An element whose removal helps a loop that does not close: the last element that joins two
/// nodes already reached.
fn loop_breaker(m: &Model) -> Option<u32> {
    let mut reached = HashSet::new();
    let mut last = None;
    for e in &m.elements {
        if reached.contains(&e.from) && reached.contains(&e.to) {
            last = Some(e.id);
        }
        reached.insert(e.from);
        reached.insert(e.to);
    }
    last.or(m.elements.last().map(|e| e.id))
}
