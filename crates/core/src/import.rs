//! Import of plain-text piping component files (.pcf): the component list design tools write for
//! a pipeline, each component with its end points and bore. Coordinates in the file have X east,
//! Y north and Z up; Datum Pipe has Y up and Z south, so (x, y, z) becomes (x, z, −y).
//!
//! What is read: pipes; elbows and bends (their corner point and the radius from the tangent
//! length); tees and olets (a tee at the centre point); reducers (concentric or eccentric, as
//! concentric); valves, flanges and other in-line items (rigid, with their weight); supports (a
//! rest unless the name says anchor, guide or stop). Anything else with end points is read as
//! pipe and listed in the warnings. Pipes are nominal sizes of the library at Std wall; materials,
//! conditions and supports are for the user to check.

use crate::geom::bend_radius;
use crate::model::{BendKind, Comp, Element, Model, Node, Pipe, RigidKind, SupKind, Support, TeeKind};
use std::collections::HashMap;

type P3 = [f64; 3];

/// Nominal bore in millimetres → the library's nominal size.
const MM_NPS: &[(f64, &str)] = &[
    (15.0, "1/2"), (20.0, "3/4"), (25.0, "1"), (32.0, "1-1/4"), (40.0, "1-1/2"), (50.0, "2"), (65.0, "2-1/2"), (80.0, "3"), (90.0, "3-1/2"),
    (100.0, "4"), (125.0, "5"), (150.0, "6"), (200.0, "8"), (250.0, "10"), (300.0, "12"), (350.0, "14"), (400.0, "16"), (450.0, "18"), (500.0, "20"), (600.0, "24"),
];

struct Component {
    kind: String,
    attrs: Vec<(String, Vec<String>)>,
}

impl Component {
    fn all(&self, key: &str) -> Vec<&Vec<String>> {
        self.attrs.iter().filter(|a| a.0 == key).map(|a| &a.1).collect()
    }
    fn first(&self, key: &str) -> Option<&Vec<String>> {
        self.attrs.iter().find(|a| a.0 == key).map(|a| &a.1)
    }
    fn text(&self, key: &str) -> String {
        self.first(key).map(|v| v.join(" ")).unwrap_or_default()
    }
}

/// What the import made of the file, besides the model: everything it could not read as given.
pub struct Imported {
    pub model: Model,
    pub warnings: Vec<String>,
    pub components: usize,
}

fn num(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok().filter(|v| v.is_finite())
}

fn sub(a: P3, b: P3) -> P3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn len(a: P3) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
fn dot(a: P3, b: P3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Read a piping component file into a new model.
pub fn pcf(text: &str) -> Result<Imported, String> {
    // ---------------------------------------------------------------- the lines
    let mut header: Vec<(String, Vec<String>)> = vec![];
    let mut comps: Vec<Component> = vec![];
    for raw in text.lines() {
        if raw.trim().is_empty() {
            continue;
        }
        let indented = raw.starts_with(' ') || raw.starts_with('\t');
        let mut words = raw.split_whitespace().map(|w| w.to_string());
        let key = words.next().unwrap_or_default().to_uppercase();
        let rest: Vec<String> = words.collect();
        if indented {
            match comps.last_mut() {
                Some(c) => c.attrs.push((key, rest)),
                None => header.push((key, rest)),
            }
        } else if key.contains("UNITS") || key.ends_with("-FILES") || key == "PIPELINE-REFERENCE" || key.starts_with("PROJECT") || key.starts_with("AREA") || key.starts_with("DATE") || key.starts_with("REVISION") {
            header.push((key, rest));
        } else {
            comps.push(Component { kind: key, attrs: vec![] });
        }
    }
    let head = |k: &str| header.iter().find(|h| h.0 == k).map(|h| h.1.join(" ").to_uppercase());
    let inch = |v: Option<String>| v.is_some_and(|s| s.starts_with("INCH"));
    let coord = if inch(head("UNITS-CO-ORDS")) { 0.0254 } else { 0.001 };
    let bore_in = inch(head("UNITS-BORE"));
    let lbs = head("UNITS-WEIGHT").is_some_and(|s| s.starts_with("LB"));
    let name = header.iter().find(|h| h.0 == "PIPELINE-REFERENCE").map(|h| h.1.join(" ")).filter(|s| !s.is_empty()).unwrap_or_else(|| "Imported piping".into());
    if !comps.iter().any(|c| c.first("END-POINT").is_some() || c.first("CENTRE-POINT").is_some() || c.first("CO-ORDS").is_some()) {
        return Err("This file has no piping components with coordinates: is it a piping component file (.pcf)?".into());
    }

    // ---------------------------------------------------------------- points, bores and nodes
    let mut warnings: Vec<String> = vec![];
    let point = |v: &[String]| -> Option<(P3, f64)> {
        let (x, y, z) = (num(v.first()?)?, num(v.get(1)?)?, num(v.get(2)?)?);
        let bore = v.get(3).and_then(|b| num(b)).unwrap_or(0.0);
        Some(([x * coord, z * coord, -y * coord], bore))
    };
    let mut nodes: Vec<P3> = vec![];
    let node_at = |p: P3, nodes: &mut Vec<P3>| -> usize {
        if let Some(i) = nodes.iter().position(|q| len(sub(*q, p)) < 1e-3) {
            return i;
        }
        nodes.push(p);
        nodes.len() - 1
    };
    let nps_of = |bore: f64| -> Option<&'static str> {
        if bore <= 0.0 {
            return None;
        }
        let mm = if bore_in { bore * 25.4 } else { bore };
        MM_NPS.iter().min_by(|a, b| (a.0 - mm).abs().total_cmp(&(b.0 - mm).abs())).filter(|x| (x.0 - mm).abs() <= 0.15 * x.0).map(|x| x.1)
    };

    // A straight piece between two nodes, with what sits on it.
    struct Piece {
        a: usize,
        b: usize,
        nps: Option<&'static str>,
        to_nps: Option<&'static str>,
        rigid: Option<(RigidKind, f64)>,
    }
    let mut pieces: Vec<Piece> = vec![];
    let mut bends: Vec<(usize, f64)> = vec![];
    let mut tees: Vec<(usize, TeeKind)> = vec![];
    let mut sups: Vec<(usize, SupKind, String)> = vec![];
    let mut unknown: HashMap<String, usize> = HashMap::new();
    let mut last_nps: Option<&'static str> = None;
    for c in &comps {
        let ends: Vec<(P3, f64)> = c.all("END-POINT").iter().filter_map(|v| point(v)).collect();
        let centre = c.first("CENTRE-POINT").and_then(|v| point(v));
        let branch = c.first("BRANCH1-POINT").and_then(|v| point(v));
        let weight = c.first("WEIGHT").and_then(|v| v.first()).and_then(|w| num(w)).map(|w| if lbs { w * 0.453_592_37 } else { w }).unwrap_or(0.0);
        let mut nps = |b: f64| { let n = nps_of(b).or(last_nps); if n.is_some() { last_nps = n; } n };
        let k = c.kind.as_str();
        match k {
            _ if k == "SUPPORT" || k.starts_with("SUPPORT") => {
                let Some((p, _)) = c.first("CO-ORDS").and_then(|v| point(v)) else { continue };
                let label = format!("{} {}", c.text("SKEY"), c.text("NAME")).to_uppercase();
                let kind = if label.contains("ANC") { SupKind::Anchor } else if label.contains("GUI") { SupKind::Guide } else if label.contains("STOP") || label.contains("LST") { SupKind::Linestop } else { SupKind::Rest };
                let n = node_at(p, &mut nodes);
                sups.push((n, kind, c.text("NAME")));
            }
            "ELBOW" | "BEND" | "PULLED-BEND" if ends.len() == 2 && centre.is_some() => {
                let corner = centre.unwrap().0;
                let (u, v) = (sub(corner, ends[0].0), sub(ends[1].0, corner));
                let (lu, lv) = (len(u), len(v));
                let cos = if lu * lv > 0.0 { dot(u, v) / (lu * lv) } else { 1.0 };
                let n0 = nps(ends[0].1);
                let (a, m, b) = (node_at(ends[0].0, &mut nodes), node_at(corner, &mut nodes), node_at(ends[1].0, &mut nodes));
                if cos > 0.9999 || lu < 1e-6 || lv < 1e-6 {
                    pieces.push(Piece { a, b, nps: n0, to_nps: None, rigid: None });
                } else {
                    let half = 0.5 * cos.clamp(-1.0, 1.0).acos();
                    bends.push((m, 0.5 * (lu + lv) / half.tan()));
                    pieces.push(Piece { a, b: m, nps: n0, to_nps: None, rigid: None });
                    pieces.push(Piece { a: m, b, nps: n0, to_nps: None, rigid: None });
                }
            }
            _ if (k == "TEE" || k.starts_with("TEE-") || k == "CROSS") && ends.len() == 2 && centre.is_some() => {
                let ctr = centre.unwrap().0;
                let n0 = nps(ends[0].1);
                let m = node_at(ctr, &mut nodes);
                for e in &ends {
                    let a = node_at(e.0, &mut nodes);
                    pieces.push(Piece { a, b: m, nps: n0, to_nps: None, rigid: None });
                }
                if let Some((bp, bb)) = branch {
                    let b = node_at(bp, &mut nodes);
                    pieces.push(Piece { a: m, b, nps: nps_of(bb).or(n0), to_nps: None, rigid: None });
                }
                tees.push((m, if k == "TEE-SET-ON" { TeeKind::Fabricated } else { TeeKind::Welding }));
            }
            "OLET" | "WELDOLET" | "SOCKOLET" | "THREDOLET" if centre.is_some() && branch.is_some() => {
                let (ctr, (bp, bb)) = (centre.unwrap().0, branch.unwrap());
                let (m, b) = (node_at(ctr, &mut nodes), node_at(bp, &mut nodes));
                pieces.push(Piece { a: m, b, nps: nps_of(bb).or(last_nps), to_nps: None, rigid: None });
                tees.push((m, TeeKind::Fabricated));
            }
            _ if k.starts_with("REDUCER") && ends.len() == 2 => {
                let (a, b) = (node_at(ends[0].0, &mut nodes), node_at(ends[1].0, &mut nodes));
                let n0 = nps(ends[0].1);
                let n1 = nps_of(ends[1].1);
                if n1.is_some() {
                    last_nps = n1;
                }
                pieces.push(Piece { a, b, nps: n0, to_nps: n1.filter(|x| Some(*x) != n0), rigid: None });
            }
            _ if ends.len() == 2 => {
                let (a, b) = (node_at(ends[0].0, &mut nodes), node_at(ends[1].0, &mut nodes));
                let n0 = nps(ends[0].1);
                let rigid = if k.starts_with("VALVE") {
                    Some((RigidKind::Valve, weight))
                } else if k.starts_with("FLANGE") {
                    Some((RigidKind::Flange, weight))
                } else if k.starts_with("PIPE") || matches!(k, "COUPLING" | "UNION" | "GASKET" | "WELD" | "BOLT") {
                    None
                } else if k.starts_with("INSTRUMENT") || k.starts_with("FILTER") || k.starts_with("TRAP") || k.starts_with("MISC") {
                    Some((RigidKind::Rigid, weight))
                } else {
                    *unknown.entry(c.kind.clone()).or_default() += 1;
                    None
                };
                pieces.push(Piece { a, b, nps: n0, to_nps: None, rigid });
            }
            _ if !ends.is_empty() || centre.is_some() => {
                *unknown.entry(c.kind.clone()).or_default() += 1;
            }
            _ => {}
        }
    }
    for (k, n) in &unknown {
        warnings.push(format!("{n} × {k}: read as plain pipe where it has two ends, otherwise left out."));
    }
    // Zero-length pieces (a gasket at one point) are dropped; supports and olets on a piece split it.
    pieces.retain(|p| p.a != p.b && len(sub(nodes[p.b], nodes[p.a])) >= 1e-4);
    let splits: Vec<usize> = sups.iter().map(|s| s.0).chain(tees.iter().map(|t| t.0)).collect();
    for &n in &splits {
        if pieces.iter().any(|p| p.a == n || p.b == n) {
            continue;
        }
        let q = nodes[n];
        if let Some(i) = pieces.iter().position(|p| {
            let (a, b) = (nodes[p.a], nodes[p.b]);
            let d = sub(b, a);
            let t = dot(sub(q, a), d) / dot(d, d);
            t > 1e-6 && t < 1.0 - 1e-6 && len(sub(q, [a[0] + d[0] * t, a[1] + d[1] * t, a[2] + d[2] * t])) < 1e-3 && p.to_nps.is_none()
        }) {
            let p = pieces.remove(i);
            pieces.push(Piece { a: p.a, b: n, nps: p.nps, to_nps: None, rigid: p.rigid });
            pieces.push(Piece { a: n, b: p.b, nps: p.nps, to_nps: None, rigid: p.rigid.map(|r| (r.0, 0.0)) });
        } else {
            warnings.push(format!("A support or branch at ({:.0}, {:.0}, {:.0}) mm is not on the piping: left out.", q[0] * 1e3, q[1] * 1e3, q[2] * 1e3));
        }
    }
    if pieces.is_empty() {
        return Err("No piping could be read from this file.".into());
    }

    // ---------------------------------------------------------------- the model
    let mut m = Model::new();
    m.name = name.clone();
    m.notes = format!("Imported from a piping component file: {} components. Check the pipe schedules (Std wall), the material, the line conditions and the supports.", comps.len());
    m.elements.clear();
    m.nodes.clear();
    m.supports.clear();
    let mut next = m.next.max(m.max_id() + 1);
    let mut id = || { next += 1; next - 1 };
    // One pipe per nominal size, at Std wall.
    let base = m.pipes[0].clone();
    m.pipes.clear();
    let mut pipe_of: HashMap<&'static str, u32> = HashMap::new();
    let mut get_pipe = |nps: Option<&'static str>, m: &mut Model, id: &mut dyn FnMut() -> u32| -> u32 {
        let key = nps.unwrap_or("6");
        if let Some(&p) = pipe_of.get(key) {
            return p;
        }
        let pid = id();
        let mut p = Pipe::std(pid, key, "STD");
        p.corrosion = base.corrosion;
        p.mill = base.mill;
        p.fluid_rho = base.fluid_rho;
        m.pipes.push(p);
        pipe_of.insert(key, pid);
        pid
    };
    if pieces.iter().any(|p| p.nps.is_none()) {
        warnings.push("Some components had no bore that matches a library size: they use 6\" Std.".into());
    }
    let num_of = |i: usize| 10 * (i as u32 + 1);
    let (mat, cond) = (m.materials[0].id, m.conds[0].id);
    for p in &pieces {
        let pipe = get_pipe(p.nps, &mut m, &mut id);
        let comp = match (p.to_nps, p.rigid) {
            (Some(t), _) => Comp::Reducer { to: get_pipe(Some(t), &mut m, &mut id) },
            (None, Some((rigid, mass))) => Comp::Rigid { rigid, mass },
            _ => Comp::Pipe,
        };
        let d = sub(nodes[p.b], nodes[p.a]);
        m.elements.push(Element { id: id(), from: num_of(p.a), to: num_of(p.b), d, pipe, material: mat, cond, comp, ..Default::default() });
    }
    // Bends and tees go on an element ending at their node (turned round if need be).
    let mark = |n: usize, comp: Comp, m: &mut Model| -> bool {
        let num = num_of(n);
        let i = m.elements.iter().position(|e| e.to == num && e.comp == Comp::Pipe).or_else(|| m.elements.iter().position(|e| e.from == num && e.comp == Comp::Pipe));
        let Some(i) = i else { return false };
        let e = &mut m.elements[i];
        if e.from == num {
            std::mem::swap(&mut e.from, &mut e.to);
            e.d = [-e.d[0], -e.d[1], -e.d[2]];
        }
        e.comp = comp;
        true
    };
    for &(n, r) in &bends {
        if !mark(n, Comp::Bend { bend: BendKind::Custom, r }, &mut m) {
            warnings.push(format!("The bend at node {} could not be placed.", num_of(n)));
        }
    }
    for &(n, kind) in &tees {
        if !mark(n, Comp::Tee { tee: kind, pad: 0.0 }, &mut m) {
            warnings.push(format!("The tee at node {} could not be placed.", num_of(n)));
        }
    }
    let used: std::collections::HashSet<u32> = m.elements.iter().flat_map(|e| [e.from, e.to]).collect();
    for (i, p) in nodes.iter().enumerate() {
        if used.contains(&num_of(i)) {
            m.nodes.push(Node { num: num_of(i), x: p[0], y: p[1], z: p[2] });
        }
    }
    for (n, kind, tag) in &sups {
        if used.contains(&num_of(*n)) {
            let one_way = *kind == SupKind::Rest;
            m.supports.push(Support { id: id(), node: num_of(*n), kind: kind.clone(), tag: tag.clone(), one_way, ..Default::default() });
        }
    }
    m.next = next;
    // Every bend needs room on both of its elements: report the ones that do not fit.
    for e in &m.elements {
        if let Comp::Bend { bend, r } = &e.comp {
            let r = bend_radius(&m, *bend, *r, e.pipe);
            if r <= 0.0 || !r.is_finite() {
                warnings.push(format!("The bend at node {} has no usable radius.", e.to));
            }
        }
    }
    crate::cases::rebuild(&mut m);
    m.fix_ids();
    m.layout()?;
    m.check().map_err(|e| format!("The imported piping is not a valid model: {e}"))?;
    if m.supports.is_empty() {
        warnings.push("The file has no supports: add anchors and supports before analysing.".into());
    }
    Ok(Imported { model: m, warnings, components: comps.len() })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small file as design tools write it: 6" line, east 3 m, an elbow turning north, a tee with
    /// a 4" branch up, a reducer to 4", a valve, and supports.
    const SAMPLE: &str = "UNITS-BORE MM
UNITS-CO-ORDS MM
UNITS-WEIGHT KGS
PIPELINE-REFERENCE 6-P-101
PIPE
    END-POINT 0.0 0.0 1000.0 150
    END-POINT 3000.0 0.0 1000.0 150
ELBOW
    END-POINT 3000.0 0.0 1000.0 150
    END-POINT 3228.6 228.6 1000.0 150
    CENTRE-POINT 3228.6 0.0 1000.0
    SKEY ELBW
PIPE
    END-POINT 3228.6 228.6 1000.0 150
    END-POINT 3228.6 2000.0 1000.0 150
TEE
    END-POINT 3228.6 2000.0 1000.0 150
    END-POINT 3228.6 2300.0 1000.0 150
    CENTRE-POINT 3228.6 2150.0 1000.0
    BRANCH1-POINT 3228.6 2150.0 1150.0 100
PIPE
    END-POINT 3228.6 2150.0 1150.0 100
    END-POINT 3228.6 2150.0 3000.0 100
REDUCER-CONCENTRIC
    END-POINT 3228.6 2300.0 1000.0 150
    END-POINT 3228.6 2450.0 1000.0 100
VALVE
    END-POINT 3228.6 2450.0 1000.0 100
    END-POINT 3228.6 2750.0 1000.0 100
    WEIGHT 45.0
SUPPORT
    CO-ORDS 0.0 0.0 1000.0 0
    NAME ANCHOR-1
SUPPORT
    CO-ORDS 1500.0 0.0 1000.0 0
    NAME PS-1
SUPPORT
    CO-ORDS 3228.6 2150.0 3000.0 0
    NAME ANC-2
SUPPORT
    CO-ORDS 3228.6 2750.0 1000.0 0
    NAME ANC-3
";

    #[test]
    fn a_small_file_reads_into_a_model_that_analyses() {
        let r = pcf(SAMPLE).unwrap();
        let m = &r.model;
        assert_eq!(m.name, "6-P-101");
        // Pipes: 6" and 4" Std.
        let mut sizes: Vec<&str> = m.pipes.iter().map(|p| p.nps.as_str()).collect();
        sizes.sort();
        assert_eq!(sizes, ["4", "6"]);
        // The elbow: R = tangent / tan(45°) = 228.6 mm, at the corner node, north is −Z.
        let bend = m.elements.iter().find(|e| matches!(e.comp, Comp::Bend { .. })).unwrap();
        assert!(matches!(bend.comp, Comp::Bend { bend: BendKind::Custom, r } if (r - 0.2286).abs() < 1e-9));
        let corner = m.nodes.iter().find(|n| n.num == bend.to).unwrap();
        assert!((corner.x - 3.2286).abs() < 1e-9 && (corner.y - 1.0).abs() < 1e-9 && corner.z.abs() < 1e-9);
        assert!(m.elements.iter().any(|e| matches!(e.comp, Comp::Tee { .. })));
        assert!(m.elements.iter().any(|e| matches!(e.comp, Comp::Reducer { .. })));
        assert!(m.elements.iter().any(|e| matches!(e.comp, Comp::Rigid { rigid: RigidKind::Valve, mass } if mass == 45.0)));
        // The rest splits the first pipe; anchors at the three ends.
        assert_eq!(m.supports.iter().filter(|s| s.kind == SupKind::Anchor).count(), 3);
        assert!(m.supports.iter().any(|s| s.kind == SupKind::Rest && s.tag == "PS-1" && s.one_way));
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
        let res = crate::solve::analyze(m).unwrap();
        assert!(res.cases.iter().all(|c| c.imbalance < 1e-9));
    }

    #[test]
    fn damaged_files_never_crash_the_import() {
        // Every prefix of the sample, and the sample with each line damaged in turn: the import
        // gives a model or a reason, never a panic, and any model it gives passes the checks.
        let lines: Vec<&str> = SAMPLE.lines().collect();
        let mut tries: Vec<String> = (0..=lines.len()).map(|n| lines[..n].join("\n")).collect();
        for i in 0..lines.len() {
            for bad in ["", "    END-POINT nan 0 0 150", "    END-POINT 1e308 1e308 1e308 -5", "PIPE", "    CENTRE-POINT 0 0", "\u{feff}???"] {
                let mut l = lines.clone();
                l[i] = bad;
                tries.push(l.join("\n"));
            }
        }
        for t in &tries {
            if let Ok(r) = pcf(t) {
                assert!(r.model.check().is_ok());
            }
        }
    }

    #[test]
    fn inches_unknown_parts_and_bad_files() {
        let f = "UNITS-CO-ORDS INCH\nUNITS-BORE INCH\nPIPE\n    END-POINT 0 0 0 6\n    END-POINT 120 0 0 6\nWIDGET\n    END-POINT 120 0 0 6\n    END-POINT 130 0 0 6\n";
        let r = pcf(f).unwrap();
        assert!((r.model.elements[0].d[0] - 3.048).abs() < 1e-12);
        assert_eq!(r.model.pipes[0].nps, "6");
        assert!(r.warnings.iter().any(|w| w.contains("WIDGET")));
        assert!(r.warnings.iter().any(|w| w.contains("no supports")));
        assert!(pcf("hello\nworld\n").is_err());
        assert!(pcf("").is_err());
    }
}
