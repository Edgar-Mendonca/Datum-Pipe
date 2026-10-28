//! Writes the engine's real output for the UI tests (tests_ui/fixture.json):
//! cargo run -q -p datum-core --example dump > tests_ui/fixture.json
use datum_core::{examples, library, units};
use serde_json::json;

fn main() {
    let out = |m: &datum_core::model::Model| json!({ "m": m, "path": null, "dirty": false, "undo": false, "redo": false, "created": [], "warnings": [] });
    let mut models = serde_json::Map::new();
    for e in examples::list() {
        models.insert(e.id.to_string(), out(&examples::get(e.id).expect("example")));
    }
    let fix = json!({
        "examples_list": examples::list(),
        "models": models,
        "unit_table": (units::shown(false), units::shown(true)),
        "library": { "sizes": library::sizes(), "materials": library::materials(), "source": library::SIZE_SOURCE },
    });
    println!("{}", serde_json::to_string(&fix).unwrap());
}
