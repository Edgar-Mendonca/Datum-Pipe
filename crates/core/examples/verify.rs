//! The verification record as Markdown, from the benchmarks this version solves:
//! cargo run -q -p datum-core --example verify > docs/verification/<version>.md
fn main() {
    let rows = datum_core::bench::run();
    println!("# Datum Pipe {}: verification record\n", env!("CARGO_PKG_VERSION"));
    println!("Every row was solved by Datum Pipe's engine and compared with a reference worked out independently. The same checks run as tests (`cargo test`, `bench.rs`) and in the app (Help › Verification). SI base units.\n");
    println!("| # | Benchmark | Quantity | Datum Pipe | Reference | Error | Source | |");
    println!("| --- | --- | --- | --- | --- | --- | --- | --- |");
    for r in &rows {
        println!("| {} | {} | {} | {:.6e} | {:.6e} | {:.1e} | {} | {} |", r.bench, r.name, r.quantity, r.result, r.reference, r.error, r.source, if r.pass { "pass" } else { "FAIL" });
    }
    println!("\n{} of {} checks pass.", rows.iter().filter(|r| r.pass).count(), rows.len());
}
