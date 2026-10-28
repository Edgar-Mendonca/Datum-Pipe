//! Datum Pipe engine: the piping model and its edits, the pipe and material library, units, the
//! load cases, and (from 0.1.1) the 3D pipe element solver, code checks, isometrics, the report
//! data and the verification benchmarks. Pure Rust; the window only shows what this computes.

// Numerical code reads best with index loops and explicit NaN-rejecting comparisons.
#![allow(clippy::needless_range_loop, clippy::too_many_arguments, clippy::type_complexity, clippy::neg_cmp_op_on_partial_ord)]

pub mod cases;
pub mod doc;
pub mod examples;
mod hardening;
pub mod library;
pub mod model;
pub mod sparse;
pub mod units;
