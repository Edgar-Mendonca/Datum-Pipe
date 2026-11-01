//! A quick look at every example's results: cargo run -q -p datum-core --example summary
fn main() {
    for e in datum_core::examples::list() {
        let m = datum_core::examples::get(e.id).unwrap();
        for v in &m.vessels {
            match datum_core::vessel::check(v) {
                Ok(r) => {
                    println!("{} vessel {}: MAWP {:.4} MPa, test {:.4}, empty {:.0} kg, operating {:.0}, warnings {:?}", e.name, r.name, r.mawp / 1e6, r.test / 1e6, r.empty, r.operating, r.warnings);
                    for p in &r.parts { println!("  {:12} P {:.4} t_req {:.2} t_min {:.2} nom {:.1} MAWP {:.4} pass {}", p.name, p.p / 1e6, p.t_req * 1e3, p.t_min * 1e3, p.t_nom * 1e3, p.mawp_top / 1e6, p.pass); }
                    for z in &r.nozzles { println!("  {:18} d {:.1} A {:.0} avail {:.0} (A1 {:.0} A2 {:.0} A41 {:.0} A5 {:.0} A42 {:.0}) neck {:.2}/{:.2} pass {}", z.name, z.d * 1e3, z.a_req * 1e6, z.a_avail * 1e6, z.a1 * 1e6, z.a2 * 1e6, z.a41 * 1e6, z.a5 * 1e6, z.a42 * 1e6, z.t_neck * 1e3, z.t_neck_req * 1e3, z.pass); }
                }
                Err(x) => println!("{} vessel ERROR {x}", e.name),
            }
        }
        if m.elements.is_empty() { continue; }
        match datum_core::solve::analyze(&m) {
            Ok(r) => {
                println!("{} ({} warnings: {:?})", e.name, r.warnings.len(), r.warnings);
                for c in &r.cases {
                    let w = c.worst.map(|i| &c.stresses[i]);
                    println!("  {:8} worst {:>6} at {:10} imb {:.1e}  nozzles {:?}", c.name, w.map(|s| format!("{:.3}", s.ratio)).unwrap_or("-".into()), w.map(|s| r.geom.points[s.point].label.clone()).unwrap_or_default(), c.imbalance, c.nozzles.iter().map(|z| (z.ratios.iter().cloned().fold(0.0, f64::max) * 100.0).round() / 100.0).collect::<Vec<_>>());
                }
                for wl in r.walls.iter().filter(|w| !w.pass) { println!("  wall FAIL {:?}", wl.element); }
                for c in &r.cases { for q in &c.equipment { println!("  {:8} equipment {} pass {} worst {:.3}", c.name, q.name, q.pass, q.rows.iter().map(|x| x.ratio).fold(0.0, f64::max)); } }
                for c in &r.cases { if c.name == "OPE1" { for x in &c.reactions { println!("    {} FY {:.0} FX {:.0} FZ {:.0}", x.node, x.f[1], x.f[0], x.f[2]); } } }
            }
            Err(x) => println!("{}: ERROR {x}", e.name),
        }
    }
}
