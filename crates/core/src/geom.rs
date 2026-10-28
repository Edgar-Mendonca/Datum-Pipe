//! The analysis geometry: the element table turned into points and segments as the solver, the 3D
//! view and the isometric see them. A bend at a node trims the two straight elements meeting there
//! by its tangent length R·tan(θ/2) and puts in an arc from its near point to its far point, with a
//! point in its middle (which stands for the bend's node: supports and loads there act at the
//! middle of the bend).

use crate::library;
use crate::mat::*;
use crate::model::{BendKind, Comp, Model, RigidKind, TeeKind};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize)]
pub struct Point {
    pub p: V,
    /// What the tables call it: "30", or "30 near" / "30 far" on a bend.
    pub label: String,
    /// The model node this point stands for.
    pub node: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Shape {
    Straight,
    /// An arc about `centre` through `angle` (rad), in the plane with normal `normal`
    /// (start × end direction); radius `r`.
    Arc { centre: V, normal: V, r: f64, angle: f64 },
}

#[derive(Clone, Debug, Serialize)]
pub struct Seg {
    pub a: usize,
    pub b: usize,
    pub shape: Shape,
    /// The element this segment is part of.
    pub element: u32,
    /// The pipe at its start and end (the same except on a reducer).
    pub pipe: u32,
    pub pipe_b: u32,
    pub material: u32,
    pub cond: u32,
    /// Rigid (valve, flange, rigid part), with its weight (kg) spread along it.
    pub rigid: Option<(RigidKind, f64)>,
    /// The bend's flexibility characteristic is for this element's bend (node).
    pub bend: Option<u32>,
    pub extra: f64,
    pub length: f64,
}

/// A tee at a node: where its stress intensification applies.
#[derive(Clone, Debug, Serialize)]
pub struct Tee {
    pub node: u32,
    pub point: usize,
    pub kind: TeeKind,
    pub pad: f64,
    /// The pipe of the run (for the flexibility characteristic).
    pub pipe: u32,
    /// The plane of the tee (normal), for in-plane and out-of-plane moments.
    pub normal: V,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Geom {
    pub points: Vec<Point>,
    pub segs: Vec<Seg>,
    pub tees: Vec<Tee>,
    /// Each model node's point.
    pub node_point: HashMap<u32, usize>,
    pub warnings: Vec<String>,
}

/// A bend's radius: 1.5 × NPS (long radius), 1.0 × NPS (short), or the given one; your own sizes
/// use the outside diameter for NPS.
pub fn bend_radius(m: &Model, kind: BendKind, r: f64, pipe: u32) -> f64 {
    let p = m.pipe(pipe);
    let nps = p.and_then(|p| library::nps_inches(&p.nps)).map(|n| n * 0.0254).unwrap_or_else(|| p.map_or(0.15, |p| p.od));
    match kind {
        BendKind::Long => 1.5 * nps,
        BendKind::Short => nps,
        BendKind::Custom => r,
    }
}

impl Geom {
    pub fn build(m: &Model) -> Result<Geom, String> {
        let mut g = Geom::default();
        let pos: HashMap<u32, V> = m.nodes.iter().map(|n| (n.num, n.p())).collect();
        // Incident elements at each node: (element index, the direction leaving the node).
        let mut at: HashMap<u32, Vec<(usize, V)>> = HashMap::new();
        for (i, e) in m.elements.iter().enumerate() {
            let u = unit(e.d);
            at.entry(e.from).or_default().push((i, u));
            at.entry(e.to).or_default().push((i, scale(u, -1.0)));
        }
        // Bends: trims at each end of each element, and the arcs to add.
        let mut trim = vec![[0.0f64; 2]; m.elements.len()];
        struct Arc {
            node: u32,
            e: usize,
            n: V,
            f: V,
            c: V,
            mid: V,
            normal: V,
            r: f64,
            angle: f64,
        }
        let mut arcs: Vec<Arc> = vec![];
        for (i, e) in m.elements.iter().enumerate() {
            let Comp::Bend { bend, r } = e.comp else { continue };
            let b = e.to;
            let others: Vec<&(usize, V)> = at.get(&b).map(|v| v.iter().filter(|x| x.0 != i).collect()).unwrap_or_default();
            if others.len() != 1 {
                g.warnings.push(if others.is_empty() { format!("The bend at node {b} has no element after it yet; it is left out.") } else { format!("The bend at node {b} has {} elements after it; a bend joins exactly two. It is left out.", others.len()) });
                continue;
            }
            let (j, u2) = *others[0];
            let u1 = unit(e.d);
            let cos = dot(u1, u2).clamp(-1.0, 1.0);
            let theta = cos.acos();
            if theta < 1e-4 {
                g.warnings.push(format!("The bend at node {b} is in a straight line; it is left out."));
                continue;
            }
            if theta > std::f64::consts::PI - 1e-3 {
                return Err(format!("The bend at node {b} turns the pipe back on itself."));
            }
            let rad = bend_radius(m, bend, r, e.pipe);
            let lt = rad * (theta / 2.0).tan();
            trim[i][1] += lt;
            // The next element is trimmed at the end that touches node b.
            if m.elements[j].from == b { trim[j][0] += lt } else { trim[j][1] += lt }
            let pb = pos[&b];
            let n = sub(pb, scale(u1, lt));
            let f = add(pb, scale(u2, lt));
            let inward = unit(sub(u2, scale(u1, dot(u2, u1))));
            let c = add(n, scale(inward, rad));
            let mid = add(c, scale(unit(sub(pb, c)), rad));
            arcs.push(Arc { node: b, e: i, n, f, c, mid, normal: unit(cross(u1, u2)), r: rad, angle: theta });
        }
        // Points, shared where they coincide.
        let mut key: HashMap<[i64; 3], usize> = HashMap::new();
        let mut point = |g: &mut Geom, p: V, label: String, node: Option<u32>| -> usize {
            let k = [(p[0] * 1e7).round() as i64, (p[1] * 1e7).round() as i64, (p[2] * 1e7).round() as i64];
            if let Some(&i) = key.get(&k) {
                if node.is_some() && g.points[i].node.is_none() {
                    g.points[i].node = node;
                    g.points[i].label = label;
                }
                return i;
            }
            g.points.push(Point { p, label, node });
            key.insert(k, g.points.len() - 1);
            g.points.len() - 1
        };
        let bent: HashMap<u32, usize> = arcs.iter().enumerate().map(|(k, a)| (a.node, k)).collect();
        let mut mids = vec![0usize; arcs.len()];
        for (k, a) in arcs.iter().enumerate() {
            mids[k] = point(&mut g, a.mid, a.node.to_string(), Some(a.node));
            g.node_point.insert(a.node, mids[k]);
        }
        for n in &m.nodes {
            if !bent.contains_key(&n.num) {
                let i = point(&mut g, n.p(), n.num.to_string(), Some(n.num));
                g.node_point.insert(n.num, i);
            }
        }
        for (i, e) in m.elements.iter().enumerate() {
            let l = e.length();
            let u = unit(e.d);
            let [t0, t1] = trim[i];
            if t0 + t1 > l + 1e-9 {
                return Err(format!("Element {}–{} is {:.0} mm long, but its bends need {:.0} mm of it. Make it longer or use a smaller bend radius.", e.from, e.to, l * 1000.0, (t0 + t1) * 1000.0));
            }
            let pa = add(pos[&e.from], scale(u, t0));
            let pb = sub(pos[&e.to], scale(u, t1));
            let la = if t0 > 0.0 { format!("{} far", e.from) } else { e.from.to_string() };
            let lb = if t1 > 0.0 { format!("{} near", e.to) } else { e.to.to_string() };
            let a = if t0 > 0.0 { point(&mut g, pa, la, None) } else { g.node_point[&e.from] };
            let b = if t1 > 0.0 { point(&mut g, pb, lb, None) } else { g.node_point[&e.to] };
            let len = l - t0 - t1;
            if len > 1e-9 && a != b {
                let (pipe_b, rigid) = match &e.comp {
                    Comp::Reducer { to } => (*to, None),
                    Comp::Rigid { rigid, mass } => (e.pipe, Some((*rigid, *mass))),
                    _ => (e.pipe, None),
                };
                g.segs.push(Seg { a, b, shape: Shape::Straight, element: e.id, pipe: e.pipe, pipe_b, material: e.material, cond: e.cond, rigid, bend: None, extra: e.extra, length: len });
            }
        }
        // Arcs: near → middle → far, on the element whose To node the bend is at.
        for (k, a) in arcs.iter().enumerate() {
            let e = &m.elements[a.e];
            let lab = a.node;
            let pn = point(&mut g, a.n, format!("{lab} near"), None);
            let pf = point(&mut g, a.f, format!("{lab} far"), None);
            for (x, y) in [(pn, mids[k]), (mids[k], pf)] {
                g.segs.push(Seg { a: x, b: y, shape: Shape::Arc { centre: a.c, normal: a.normal, r: a.r, angle: a.angle / 2.0 }, element: e.id, pipe: e.pipe, pipe_b: e.pipe, material: e.material, cond: e.cond, rigid: None, bend: Some(lab), extra: e.extra, length: a.r * a.angle / 2.0 });
            }
        }
        // Tees: at the To node of an element marked with one; three or more elements meet there.
        for e in &m.elements {
            let Comp::Tee { tee, pad } = e.comp else { continue };
            let list = at.get(&e.to).cloned().unwrap_or_default();
            if list.len() < 3 {
                g.warnings.push(format!("The tee at node {} has {} elements meeting there; it needs three.", e.to, list.len()));
            }
            // The run: the two directions most nearly opposite; the plane holds the branch too.
            let mut normal = [0.0, 1.0, 0.0];
            'find: for (x, ux) in &list {
                for (y, uy) in &list {
                    if x < y && dot(*ux, *uy) < -0.99 {
                        if let Some((_, ub)) = list.iter().find(|(z, _)| z != x && z != y) {
                            let n = cross(*ux, *ub);
                            if norm(n) > 1e-6 {
                                normal = unit(n);
                            }
                        }
                        break 'find;
                    }
                }
            }
            let point = g.node_point[&e.to];
            g.tees.push(Tee { node: e.to, point, kind: tee, pad, pipe: e.pipe, normal });
        }
        Ok(g)
    }

    /// The position along a segment at fraction `s` (0 at a, 1 at b), and the direction there.
    pub fn at(&self, seg: &Seg, s: f64) -> (V, V) {
        let (pa, pb) = (self.points[seg.a].p, self.points[seg.b].p);
        match seg.shape {
            Shape::Straight => (add(pa, scale(sub(pb, pa), s)), unit(sub(pb, pa))),
            Shape::Arc { centre, normal, r, angle } => {
                let ra = unit(sub(pa, centre));
                let tan0 = cross(normal, ra);
                let phi = angle * s;
                let rp = add(scale(ra, phi.cos()), scale(tan0, phi.sin()));
                (add(centre, scale(rp, r)), unit(cross(normal, rp)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Comp;

    #[test]
    fn a_bend_becomes_an_arc() {
        // An L: 10 → 20 along X 3 m, 20 → 30 along Y 2 m, a long-radius bend at 20 (6": R = 228.6 mm).
        let mut d = crate::examples::table("L", &[(10, 20, 3.0, 0.0, 0.0), (20, 30, 0.0, 2.0, 0.0)]);
        let mut e = d.m.elements[0].clone();
        e.comp = Comp::Bend { bend: BendKind::Long, r: 0.0 };
        d.apply(crate::doc::Op::Set { item: crate::doc::Item::Element(e) }).unwrap();
        let g = Geom::build(&d.m).unwrap();
        let r = 0.2286;
        assert_eq!(g.segs.len(), 4);
        let near = g.points.iter().find(|p| p.label == "20 near").unwrap();
        assert!((near.p[0] - (3.0 - r)).abs() < 1e-12);
        let mid = &g.points[g.node_point[&20]];
        // The middle of a 90° arc is r(1 − 1/√2) from the corner along both legs.
        let off = r * (1.0 - 0.5f64.sqrt());
        assert!((mid.p[0] - (3.0 - off)).abs() < 1e-12 && (mid.p[1] - off).abs() < 1e-12, "{:?}", mid.p);
        let arc = g.segs.iter().find(|s| matches!(s.shape, Shape::Arc { .. })).unwrap();
        let (p1, t1) = g.at(arc, 1.0);
        assert!(dist(p1, mid.p) < 1e-12 && (dot(t1, unit([1.0, 1.0, 0.0])) - 1.0).abs() < 1e-12);
        let total: f64 = g.segs.iter().map(|s| s.length).sum();
        assert!((total - (5.0 - 2.0 * r + r * std::f64::consts::FRAC_PI_2)).abs() < 1e-12);
    }
}
