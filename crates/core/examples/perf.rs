//! Times what one edit costs on a large model: the edit, the geometry, the data sent to the window,
//! and the analysis. cargo run --release -p datum-core --example perf [bays]
use datum_core::doc::{Doc, Item, Op};
use datum_core::{examples, geom::Geom, solve};
use std::time::Instant;

fn main() {
    let bays: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(160);
    let t = Instant::now();
    let m = examples::big(bays);
    println!("model: {} elements, {} supports, built in {:.0} ms", m.elements.len(), m.supports.len(), t.elapsed().as_secs_f64() * 1e3);
    let mut d = Doc::new(m.clone());
    let mut e = d.m.elements[d.m.elements.len() / 2].clone();
    e.d[0] += 0.5;
    let t = Instant::now();
    d.apply(Op::Set { item: Item::Element(e) }).unwrap();
    println!("one edit (apply): {:.1} ms", t.elapsed().as_secs_f64() * 1e3);
    let t = Instant::now();
    let g = Geom::build(&d.m).unwrap();
    println!("geometry: {:.1} ms ({} segments)", t.elapsed().as_secs_f64() * 1e3, g.segs.len());
    let t = Instant::now();
    let js = serde_json::to_string(&(&d.m, &g)).unwrap();
    println!("model + geometry as JSON: {:.1} ms, {:.0} kB", t.elapsed().as_secs_f64() * 1e3, js.len() as f64 / 1e3);
    let t = Instant::now();
    let r = solve::analyze(&d.m).unwrap();
    println!("analysis: {:.0} ms ({} cases, {} equations, most solves in a case {})", t.elapsed().as_secs_f64() * 1e3, r.cases.len(), r.stats.equations, r.cases.iter().map(|c| c.iterations).max().unwrap_or(0));
    let t = Instant::now();
    let rj = serde_json::to_string(&r).unwrap();
    println!("results as JSON: {:.1} ms, {:.0} kB", t.elapsed().as_secs_f64() * 1e3, rj.len() as f64 / 1e3);
    println!("warnings: {}", r.warnings.len());
}
