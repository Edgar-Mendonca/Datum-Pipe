//! A quick look at every example's results: cargo run -q -p datum-core --example summary
fn main() {
    for e in datum_core::examples::list() {
        let m = datum_core::examples::get(e.id).unwrap();
        if m.elements.is_empty() { continue; }
        match datum_core::solve::analyze(&m) {
            Ok(r) => {
                println!("{} ({} warnings: {:?})", e.name, r.warnings.len(), r.warnings);
                for c in &r.cases {
                    let w = c.worst.map(|i| &c.stresses[i]);
                    println!("  {:8} worst {:>6} at {:10} imb {:.1e}  nozzles {:?}", c.name, w.map(|s| format!("{:.3}", s.ratio)).unwrap_or("-".into()), w.map(|s| r.geom.points[s.point].label.clone()).unwrap_or_default(), c.imbalance, c.nozzles.iter().map(|z| (z.ratios.iter().cloned().fold(0.0, f64::max) * 100.0).round() / 100.0).collect::<Vec<_>>());
                }
                for wl in r.walls.iter().filter(|w| !w.pass) { println!("  wall FAIL {:?}", wl.element); }
                for c in &r.cases { if c.name == "OPE1" { for x in &c.reactions { println!("    {} FY {:.0} FX {:.0} FZ {:.0}", x.node, x.f[1], x.f[0], x.f[2]); } } }
            }
            Err(x) => println!("{}: ERROR {x}", e.name),
        }
    }
}
