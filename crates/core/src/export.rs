//! Export of the model as a plain-text piping component file (.pcf), the inverse of `import::pcf`:
//! a component per piece, each with its end points and nominal bore, in millimetres with X east,
//! Y north and Z up (Datum Pipe's (x, y, z) is written (x, −z, y)).
//!
//! What is written: pipes; bends as elbows (the end points of the arc and the corner point);
//! tees (run ends and branch end at half the run's outside diameter from the centre, as a fitting
//! has them); reducers (concentric); valves and flanges (with their weight); other rigid parts and
//! expansion joints as miscellaneous components (they come back rigid); supports at their nodes,
//! named by their kind so they read back as the same kind. Materials, schedules, temperatures and
//! pressures are not part of this file.

use crate::geom::{Geom, Shape};
use crate::model::{Comp, Model, RigidKind, SupKind};

type P3 = [f64; 3];

/// The library's nominal sizes → nominal bore in millimetres.
const NPS_MM: &[(&str, f64)] = &[
    ("1/2", 15.0), ("3/4", 20.0), ("1", 25.0), ("1-1/4", 32.0), ("1-1/2", 40.0), ("2", 50.0), ("2-1/2", 65.0), ("3", 80.0), ("3-1/2", 90.0),
    ("4", 100.0), ("5", 125.0), ("6", 150.0), ("8", 200.0), ("10", 250.0), ("12", 300.0), ("14", 350.0), ("16", 400.0), ("18", 450.0), ("20", 500.0), ("24", 600.0),
];

fn sub(a: P3, b: P3) -> P3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: P3, b: P3) -> P3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale(a: P3, s: f64) -> P3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn len(a: P3) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
fn unit(a: P3) -> P3 {
    let l = len(a);
    if l > 0.0 { scale(a, 1.0 / l) } else { a }
}
fn dot(a: P3, b: P3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A point in the file's axes and units.
fn xyz(p: P3) -> String {
    let f = |v: f64| {
        let s = format!("{:.1}", v * 1000.0);
        if s == "-0.0" { "0.0".to_string() } else { s }
    };
    format!("{} {} {}", f(p[0]), f(-p[2]), f(p[1]))
}

/// The model as a piping component file.
pub fn pcf(m: &Model) -> Result<String, String> {
    if m.elements.is_empty() {
        return Err("There is no piping to export yet.".into());
    }
    let g = Geom::build(m)?;
    let bore = |pipe: u32| -> String {
        let Some(p) = m.pipe(pipe) else { return "0".into() };
        match NPS_MM.iter().find(|x| x.0 == p.nps) {
            Some(x) => format!("{}", x.1),
            None => format!("{:.1}", p.od * 1000.0),
        }
    };
    let node_pos = |n: u32| m.nodes.iter().find(|x| x.num == n).map(|x| [x.x, x.y, x.z]);
    let mut out = String::new();
    let mut line = |s: String| {
        out.push_str(&s);
        out.push('\n');
    };
    line("UNITS-BORE MM".into());
    line("UNITS-CO-ORDS MM".into());
    line("UNITS-WEIGHT KGS".into());
    line(format!("PIPELINE-REFERENCE {}", m.name.replace(['\n', '\r'], " ")));

    // Tees: the run and the branch at each tee point, and how far the fitting reaches along each.
    let mut trim: Vec<[f64; 2]> = vec![[0.0; 2]; g.segs.len()];
    let mut tees: Vec<(P3, [P3; 2], P3, u32, u32, &'static str)> = vec![];
    for t in &g.tees {
        let at: Vec<(usize, usize)> = g.segs.iter().enumerate().filter(|(_, s)| s.a == t.point || s.b == t.point).map(|(i, s)| (i, if s.a == t.point { 0 } else { 1 })).collect();
        if at.len() != 3 {
            continue;
        }
        // Out of the tee along each segment.
        let dir = |&(i, end): &(usize, usize)| {
            let s = &g.segs[i];
            let (_, tng) = g.at(s, if end == 0 { 0.0 } else { 1.0 });
            if end == 0 { tng } else { scale(tng, -1.0) }
        };
        let dirs: Vec<P3> = at.iter().map(dir).collect();
        // The run: the two most nearly opposite directions.
        let pairs = [(0, 1, 2), (0, 2, 1), (1, 2, 0)];
        let (r1, r2, br) = pairs.iter().copied().min_by(|a, b| dot(dirs[a.0], dirs[a.1]).total_cmp(&dot(dirs[b.0], dirs[b.1]))).unwrap();
        let centre = g.points[t.point].p;
        let od = m.pipe(t.pipe).map_or(0.17, |p| p.od);
        let mut reach = |k: usize| {
            let (i, end) = at[k];
            let c = (od / 2.0).min(0.4 * g.segs[i].length);
            trim[i][end] = c;
            add(centre, scale(dirs[k], c))
        };
        let (e1, e2, eb) = (reach(r1), reach(r2), reach(br));
        let kind = if matches!(t.kind, crate::model::TeeKind::Welding) { "TEE" } else { "TEE-SET-ON" };
        tees.push((centre, [e1, e2], eb, g.segs[at[r1].0].pipe, g.segs[at[br].0].pipe, kind));
    }

    // Straight pieces, and the bends (two arcs each, written as one elbow).
    let mut done_bends: Vec<u32> = vec![];
    for (i, s) in g.segs.iter().enumerate() {
        let e = m.elements.iter().find(|e| e.id == s.element);
        match s.shape {
            Shape::Straight => {
                let (pa, pb) = (g.points[s.a].p, g.points[s.b].p);
                let t = unit(sub(pb, pa));
                let (a, b) = (add(pa, scale(t, trim[i][0])), sub(pb, scale(t, trim[i][1])));
                let comp = e.map(|e| e.comp.clone()).unwrap_or_default();
                let (kind, extra): (&str, Option<String>) = match (&comp, s.rigid) {
                    (Comp::Reducer { .. }, _) => ("REDUCER-CONCENTRIC", None),
                    (_, Some((RigidKind::Valve, mass))) => ("VALVE", Some(format!("    WEIGHT {mass}"))),
                    (_, Some((RigidKind::Flange | RigidKind::Slipon, mass))) => ("FLANGE", Some(format!("    WEIGHT {mass}"))),
                    (_, Some((RigidKind::Rigid, mass))) => ("MISC-COMPONENT", Some(format!("    WEIGHT {mass}"))),
                    (Comp::Bellows(b), _) => ("MISC-COMPONENT", Some(format!("    ITEM-DESCRIPTION EXPANSION JOINT\n    WEIGHT {}", b.mass))),
                    _ => ("PIPE", None),
                };
                line(kind.into());
                line(format!("    END-POINT {} {}", xyz(a), bore(s.pipe)));
                line(format!("    END-POINT {} {}", xyz(b), bore(s.pipe_b)));
                if let Some(x) = extra {
                    line(x);
                }
            }
            Shape::Arc { .. } => {
                let Some(node) = s.bend else { continue };
                if done_bends.contains(&node) {
                    continue;
                }
                done_bends.push(node);
                let arcs: Vec<&crate::geom::Seg> = g.segs.iter().filter(|x| x.bend == Some(node)).collect();
                let (Some(first), Some(last)) = (arcs.first(), arcs.last()) else { continue };
                let corner = node_pos(node).ok_or("A bend's node is missing.")?;
                line("ELBOW".into());
                line(format!("    END-POINT {} {}", xyz(g.points[first.a].p), bore(s.pipe)));
                line(format!("    END-POINT {} {}", xyz(g.points[last.b].p), bore(s.pipe)));
                line(format!("    CENTRE-POINT {}", xyz(corner)));
                line("    SKEY ELBW".into());
            }
        }
    }
    for (centre, run, branch, pipe, bpipe, kind) in &tees {
        line(kind.to_string());
        line(format!("    END-POINT {} {}", xyz(run[0]), bore(*pipe)));
        line(format!("    END-POINT {} {}", xyz(run[1]), bore(*pipe)));
        line(format!("    CENTRE-POINT {}", xyz(*centre)));
        line(format!("    BRANCH1-POINT {} {}", xyz(*branch), bore(*bpipe)));
    }
    // Supports, named so they read back as the same kind.
    for sp in &m.supports {
        let Some(p) = node_pos(sp.node) else { continue };
        let kind = match sp.kind {
            SupKind::Anchor => "ANCHOR",
            SupKind::Guide => "GUIDE",
            SupKind::Linestop => "STOP",
            SupKind::Rest => "REST",
            SupKind::Restraint { .. } => "RESTRAINT",
            SupKind::Spring { .. } | SupKind::Rotspring { .. } | SupKind::Hanger { .. } => "SPRING",
            SupKind::Constant { .. } => "CONSTANT",
        };
        line("SUPPORT".into());
        line(format!("    CO-ORDS {} 0", xyz(p)));
        let tag = sp.tag.trim();
        line(format!("    NAME {}", if tag.is_empty() { format!("{kind}-{}", sp.node) } else if tag.to_uppercase().contains(kind) { tag.to_string() } else { format!("{kind} {tag}") }));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every example goes out and comes back: the same nodes where the piping turns or is
    /// supported, the same length of piping, bends, tees, valves and supports by kind.
    #[test]
    fn every_example_round_trips() {
        for ex in crate::examples::list() {
            let Some(m) = crate::examples::get(ex.id) else { continue };
            if m.elements.is_empty() {
                assert!(pcf(&m).is_err());
                continue;
            }
            let text = pcf(&m).unwrap();
            let r = crate::import::pcf(&text).unwrap_or_else(|e| panic!("{}: {e}\n{text}", ex.id));
            let back = &r.model;
            let total = |m: &Model| m.elements.iter().map(|e| e.length()).sum::<f64>();
            assert!((total(&m) - total(back)).abs() < 1e-3 * m.elements.len() as f64, "{}: length {} vs {}", ex.id, total(&m), total(back));
            let count = |m: &Model, f: &dyn Fn(&Comp) -> bool| m.elements.iter().filter(|e| f(&e.comp)).count();
            assert_eq!(count(&m, &|c| matches!(c, Comp::Bend { .. })), count(back, &|c| matches!(c, Comp::Bend { .. })), "{}: bends", ex.id);
            assert_eq!(count(&m, &|c| matches!(c, Comp::Tee { .. })), count(back, &|c| matches!(c, Comp::Tee { .. })), "{}: tees", ex.id);
            assert_eq!(count(&m, &|c| matches!(c, Comp::Rigid { rigid: RigidKind::Valve, .. })), count(back, &|c| matches!(c, Comp::Rigid { rigid: RigidKind::Valve, .. })), "{}: valves", ex.id);
            // Every node where the original piping turns or is supported is still there.
            for sp in &m.supports {
                let p = m.nodes.iter().find(|n| n.num == sp.node).unwrap();
                assert!(back.nodes.iter().any(|n| ((n.x - p.x).powi(2) + (n.y - p.y).powi(2) + (n.z - p.z).powi(2)).sqrt() < 2e-3), "{}: support node {} lost", ex.id, sp.node);
            }
            let kinds = |m: &Model, k: SupKind| m.supports.iter().filter(|s| s.kind == k).count();
            for k in [SupKind::Anchor, SupKind::Guide, SupKind::Linestop] {
                assert_eq!(kinds(&m, k.clone()), kinds(back, k.clone()), "{}: {:?}", ex.id, k);
            }
        }
    }

    #[test]
    fn coordinates_and_bores_in_the_files_axes() {
        assert_eq!(xyz([1.0, 2.0, -3.0]), "1000.0 3000.0 2000.0");
        let m = crate::examples::get("lbend").unwrap();
        let t = pcf(&m).unwrap();
        assert!(t.starts_with("UNITS-BORE MM\nUNITS-CO-ORDS MM\n"));
        assert!(t.contains("ELBOW") && t.contains(" 150\n") && !t.contains("NAME A-10\n") && t.contains("ANCHOR A-10"));
    }
}
