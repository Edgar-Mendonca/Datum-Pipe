//! The isometric drawing: the model in the 30° projection piping isometrics use (north up and to
//! the right, east down and to the right, up straight up), with node numbers, a dimension along
//! every element (node to node), and the symbols of bends, tees, reducers, valves, flanges,
//! supports and nozzles. Large models are split across sheets in table order, with the joining
//! nodes marked. The window only scales and draws what this returns.

use crate::geom::{Geom, Shape};
use crate::mat::*;
use crate::model::{Comp, Model, RigidKind, SupKind};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

/// Drawing units are metres along the projection; y grows downwards (as on a page).
pub type P2 = [f64; 2];

const C30: f64 = 0.866_025_403_784_438_6;

/// The isometric projection of a point: x = (X − Z) cos 30°, y = (X + Z) sin 30° − Y.
pub fn project(p: V) -> P2 {
    [(p[0] - p[2]) * C30, (p[0] + p[2]) * 0.5 - p[1]]
}

#[derive(Clone, Debug, Serialize)]
pub struct Line {
    pub element: u32,
    pub pts: Vec<P2>,
    /// pipe, valve, flange, slipon, rigid or reducer.
    pub kind: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct Label {
    pub p: P2,
    pub num: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Dim {
    pub element: u32,
    pub a: P2,
    pub b: P2,
    /// Where the dimension line sits: a unit offset in drawing coordinates, away from the model.
    pub off: P2,
    /// The true length (m) and what it measures: "east" etc.
    pub length: f64,
    pub dir: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct Sym {
    pub p: P2,
    /// The pipe direction on the drawing at the symbol (unit).
    pub along: P2,
    /// anchor, rest, guide, linestop, restraint, spring, rotspring, constant, nozzle, tee.
    pub kind: &'static str,
    pub node: u32,
    pub tag: String,
    pub id: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Sheet {
    pub n: usize,
    pub lines: Vec<Line>,
    pub labels: Vec<Label>,
    pub dims: Vec<Dim>,
    pub syms: Vec<Sym>,
    /// Nodes that continue on another sheet: (node, the other sheet's number).
    pub cont: Vec<(u32, usize)>,
    /// min x, min y, max x, max y.
    pub bbox: [f64; 4],
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Iso {
    pub sheets: Vec<Sheet>,
}

fn dir_name(d: V) -> &'static str {
    let u = unit(d);
    let (i, v) = (0..3).map(|i| (i, u[i])).max_by(|a, b| a.1.abs().total_cmp(&b.1.abs())).unwrap_or((0, 1.0));
    if v.abs() < 0.999 {
        return "skewed";
    }
    match (i, v > 0.0) {
        (0, true) => "east",
        (0, false) => "west",
        (1, true) => "up",
        (1, false) => "down",
        (2, true) => "south",
        _ => "north",
    }
}

/// The isometric, with at most `per_sheet` elements on a sheet (0: everything on one).
pub fn draw(m: &Model, per_sheet: usize) -> Result<Iso, String> {
    let g = Geom::build(m)?;
    if m.elements.is_empty() {
        return Ok(Iso::default());
    }
    let per = if per_sheet == 0 { usize::MAX } else { per_sheet };
    let groups: Vec<&[crate::model::Element]> = m.elements.chunks(per.min(m.elements.len()).max(1)).collect();
    // Which sheet each node is drawn on (all of them, for joins).
    let mut sheets_of: HashMap<u32, Vec<usize>> = HashMap::new();
    for (k, grp) in groups.iter().enumerate() {
        for e in grp.iter() {
            for n in [e.from, e.to] {
                let v = sheets_of.entry(n).or_default();
                if !v.contains(&(k + 1)) {
                    v.push(k + 1);
                }
            }
        }
    }
    let centre = {
        let pts: Vec<P2> = g.points.iter().map(|p| project(p.p)).collect();
        let n = pts.len().max(1) as f64;
        [pts.iter().map(|p| p[0]).sum::<f64>() / n, pts.iter().map(|p| p[1]).sum::<f64>() / n]
    };
    let mut out = Iso::default();
    for (k, grp) in groups.iter().enumerate() {
        let ids: HashSet<u32> = grp.iter().map(|e| e.id).collect();
        let mut sh = Sheet { n: k + 1, lines: vec![], labels: vec![], dims: vec![], syms: vec![], cont: vec![], bbox: [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY] };
        for s in g.segs.iter().filter(|s| ids.contains(&s.element)) {
            let n = if matches!(s.shape, Shape::Arc { .. }) { 8 } else { 1 };
            let pts: Vec<P2> = (0..=n).map(|i| project(g.at(s, i as f64 / n as f64).0)).collect();
            let comp = m.elements.iter().find(|e| e.id == s.element).map(|e| e.comp.clone());
            let kind = match (&s.rigid, comp) {
                (Some((RigidKind::Valve, _)), _) => "valve",
                (Some((RigidKind::Flange, _)), _) => "flange",
                (Some((RigidKind::Slipon, _)), _) => "flange",
                (Some((RigidKind::Rigid, _)), _) => "rigid",
                (None, Some(Comp::Reducer { .. })) => "reducer",
                _ => "pipe",
            };
            sh.lines.push(Line { element: s.element, pts, kind });
        }
        let nodes: Vec<u32> = {
            let mut v = vec![];
            for e in grp.iter() {
                for n in [e.from, e.to] {
                    if !v.contains(&n) {
                        v.push(n);
                    }
                }
            }
            v
        };
        for &n in &nodes {
            let Some(&pi) = g.node_point.get(&n) else { continue };
            sh.labels.push(Label { p: project(g.points[pi].p), num: n });
            if let Some(list) = sheets_of.get(&n) {
                for &o in list {
                    if o != k + 1 {
                        sh.cont.push((n, o));
                    }
                }
            }
        }
        // Dimensions: node to node along each element, offset away from the model's middle.
        for e in grp.iter() {
            let (Some(a), Some(b)) = (m.node(e.from), m.node(e.to)) else { continue };
            let (pa, pb) = (project(a.p()), project(b.p()));
            let d = [pb[0] - pa[0], pb[1] - pa[1]];
            let l = d[0].hypot(d[1]).max(1e-12);
            let mut off = [-d[1] / l, d[0] / l];
            let mid = [(pa[0] + pb[0]) / 2.0, (pa[1] + pb[1]) / 2.0];
            if (mid[0] - centre[0]) * off[0] + (mid[1] - centre[1]) * off[1] < 0.0 {
                off = [-off[0], -off[1]];
            }
            sh.dims.push(Dim { element: e.id, a: pa, b: pb, off, length: e.length(), dir: dir_name(e.d) });
        }
        // Symbols: supports, nozzles, tees.
        let along_at = |pi: usize| -> P2 {
            let s = g.segs.iter().find(|s| s.a == pi || s.b == pi);
            let t = s.map(|s| g.at(s, if s.a == pi { 0.0 } else { 1.0 }).1).unwrap_or([1.0, 0.0, 0.0]);
            let (p0, p1) = (project([0.0; 3]), project(t));
            let d = [p1[0] - p0[0], p1[1] - p0[1]];
            let l = d[0].hypot(d[1]);
            if l < 1e-9 { [1.0, 0.0] } else { [d[0] / l, d[1] / l] }
        };
        for s in m.supports.iter().filter(|s| nodes.contains(&s.node)) {
            let Some(&pi) = g.node_point.get(&s.node) else { continue };
            let kind = match s.kind {
                SupKind::Anchor => "anchor",
                SupKind::Rest => "rest",
                SupKind::Guide => "guide",
                SupKind::Linestop => "linestop",
                SupKind::Restraint { .. } => "restraint",
                SupKind::Spring { .. } => "spring",
                SupKind::Rotspring { .. } => "rotspring",
                SupKind::Constant { .. } => "constant",
                SupKind::Hanger { .. } => "spring",
            };
            sh.syms.push(Sym { p: project(g.points[pi].p), along: along_at(pi), kind, node: s.node, tag: s.tag.clone(), id: s.id });
        }
        for z in m.nozzles.iter().filter(|z| nodes.contains(&z.node)) {
            let Some(&pi) = g.node_point.get(&z.node) else { continue };
            sh.syms.push(Sym { p: project(g.points[pi].p), along: along_at(pi), kind: "nozzle", node: z.node, tag: z.name.clone(), id: z.id });
        }
        for t in g.tees.iter().filter(|t| nodes.contains(&t.node)) {
            sh.syms.push(Sym { p: project(g.points[t.point].p), along: along_at(t.point), kind: "tee", node: t.node, tag: String::new(), id: 0 });
        }
        for p in sh.lines.iter().flat_map(|l| l.pts.iter()).chain(sh.labels.iter().map(|l| &l.p)) {
            sh.bbox = [sh.bbox[0].min(p[0]), sh.bbox[1].min(p[1]), sh.bbox[2].max(p[0]), sh.bbox[3].max(p[1])];
        }
        out.sheets.push(sh);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_projection_and_sheets() {
        // East goes down to the right at 30°, north up to the right, up straight up.
        let e = project([1.0, 0.0, 0.0]);
        assert!((e[0] - C30).abs() < 1e-12 && (e[1] - 0.5).abs() < 1e-12);
        let n = project([0.0, 0.0, -1.0]);
        assert!((n[0] - C30).abs() < 1e-12 && (n[1] + 0.5).abs() < 1e-12);
        assert_eq!(project([0.0, 2.0, 0.0]), [0.0, -2.0]);
        let m = crate::examples::get("pump").unwrap();
        let iso = draw(&m, 0).unwrap();
        assert_eq!(iso.sheets.len(), 1);
        let s = &iso.sheets[0];
        assert_eq!(s.dims.len(), m.elements.len());
        assert!(s.syms.iter().any(|x| x.kind == "nozzle") && s.lines.iter().any(|l| l.kind == "valve"));
        assert_eq!(s.dims[0].dir, "up");
        // Split in sheets of 4 elements: three sheets, joined at the shared nodes.
        let iso = draw(&m, 4).unwrap();
        assert_eq!(iso.sheets.len(), 3);
        assert!(iso.sheets[0].cont.iter().any(|(_, o)| *o == 2) && iso.sheets[1].cont.iter().any(|(_, o)| *o == 1));
        assert!(iso.sheets.iter().map(|s| s.dims.len()).sum::<usize>() == m.elements.len());
    }
}
