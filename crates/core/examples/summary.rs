//! A quick look at every example's results: cargo run -q -p datum-core --example summary
fn main() {
    for e in datum_core::examples::list() {
        let m = datum_core::examples::get(e.id).unwrap();
        for v in &m.vessels {
            match datum_core::vessel::check_in(v, &m.settings) {
                Ok(r) => {
                    println!("{} vessel {}: MAWP {:.4} MPa, test {:.4}, empty {:.0} kg, operating {:.0}, warnings {:?}", e.name, r.name, r.mawp / 1e6, r.test / 1e6, r.empty, r.operating, r.warnings);
                    for p in &r.parts { println!("  {:12} P {:.4} t_req {:.2} t_min {:.2} nom {:.1} MAWP {:.4} pass {}", p.name, p.p / 1e6, p.t_req * 1e3, p.t_min * 1e3, p.t_nom * 1e3, p.mawp_top / 1e6, p.pass); }
                    if let Some(sd) = &r.saddles { println!("  saddles Q {:.0} N M1 {:.0} M2 {:.0} K {:?}", sd.q, sd.m1, sd.m2, sd.k); for x in &sd.rows { println!("    {:55} {:9.2} MPa / {:7.2} ratio {:.3} {}", x.what, x.stress / 1e6, x.allow / 1e6, x.ratio, x.pass); } }
                    if let Some(su) = &r.support { println!("  support: wind V {:.0} M {:.0}; seis V {:.0} M {:.0}; bolts {:.0} N {:.1}/{:.1} MPa; bearing {:.2}/{:.2} MPa; ring {:.1}/{:.1} mm pass {}", su.shear_wind, su.moment_wind, su.shear_seis, su.moment_seis, su.bolt_load, su.bolt_stress / 1e6, su.bolt_allow / 1e6, su.bearing / 1e6, su.bearing_allow / 1e6, su.ring_t_req * 1e3, su.ring_t * 1e3, su.pass); for x in &su.sections { println!("    {:22} z {:6.2} Mw {:8.0} Ms {:8.0} W {:8.0} t {:7.2} c {:7.2} allow {:7.2}/{:7.2} {}", x.name, x.z, x.m_wind, x.m_seis, x.weight, x.tension / 1e6, x.compression / 1e6, x.allow_t / 1e6, x.allow_c / 1e6, x.pass); } }
                    for f in &r.flanges { println!("  flange {} G {:.1} Wm1 {:.0} Wm2 {:.0} Am {:.0} Ab {:.0} op {:.4} seat {:.4} allow {:.4}/{:.4} pass {}", f.name, f.g * 1e3, f.wm1, f.wm2, f.am * 1e6, f.ab * 1e6, f.op, f.seat, f.op_allow, f.seat_allow, f.pass); }
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
