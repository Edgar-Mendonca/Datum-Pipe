//! Writes the engine's real output for the UI tests (tests_ui/fixture.json):
//! cargo run -q -p datum-core --example dump > tests_ui/fixture.json
use datum_core::{bench, examples, geom::Geom, library, solve, units};
use serde_json::json;

fn main() {
    let out = |m: &datum_core::model::Model| {
        let (g, e) = match Geom::build(m) { Ok(g) => (Some(g), None), Err(e) => (None, Some(e)) };
        json!({ "m": m, "path": null, "dirty": false, "undo": false, "redo": false, "created": [], "warnings": [], "geom": g, "geom_error": e })
    };
    let mut results = serde_json::Map::new();
    let mut models = serde_json::Map::new();
    for e in examples::list() {
        let m = examples::get(e.id).expect("example");
        if let Ok(r) = solve::analyze(&m) {
            results.insert(e.id.to_string(), serde_json::to_value(r).unwrap());
        }
        models.insert(e.id.to_string(), out(&m));
    }
    let fix = json!({
        "examples_list": examples::list(),
        "models": models,
        "results": results,
        "verification": bench::run(),
        "unit_table": (units::shown(false), units::shown(true)),
        "library": { "sizes": library::sizes(), "materials": library::materials(), "source": library::SIZE_SOURCE },
    });
    println!("{}", serde_json::to_string(&fix).unwrap());
}
