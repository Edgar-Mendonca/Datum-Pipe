# Datum Pipe 0.1: plan

Written before building, from the brief (`Edgar-Mendonca/Datum` › `briefs/DATUM-PIPE.md`, hub session 16). Every decision there is made; this file says how 0.1 is built and in what order.

## Feedback

The user is a piping and rotating-equipment engineer who has used the commercial pipe stress tools for years and finds them expensive and hard to use. Datum Pipe is the modern, clear, free alternative, built the Datum way from scratch: an element table that draws in 3D as you type, linear supports, the process piping code's equations, support and nozzle loads, an automatic isometric and a stress report, all verified against hand calculations.

## Diagnosis: what 0.1 must get right

1. **Trust.** Pipe stress results go into support and nozzle designs. Every element and solver path needs a test against a known answer, an equilibrium check after every solve, and Help › Verification generated from the same tests.
2. **The way engineers think.** The element table (From, To, DX, DY, DZ, pipe, material, conditions, component) is the model; the 3D view, the isometric and the report follow from it.
3. **Units everywhere.** SI and US, switchable at any time, every value with its unit. The engine owns the units (Calc's table); the file keeps SI.
4. **Public words.** Equations of the process piping code (ASME B31.3) only; allowables are user inputs; never name or compare with commercial software.

## Proposal: how it is built

- **Model** (`crates/core/src/model.rs`): elements reference pipes, materials and line conditions by id; bends and tees sit at an element's To node; supports, nozzles and loads at node numbers. Node positions are laid out from the deltas (the first node of each connected piece keeps its coordinates); a loop that does not close is refused.
- **Edits** (`doc.rs`): Plan's ops on a copy, kept only when the model lays out and passes `check`; whole-model undo.
- **Solver** (`elements.rs`, `solve.rs`, from Mesh's frame solver): every element's stiffness from its exact flexibility (∫ Bᵀ C B ds along the centreline, so straight pipe, bends with their flexibility factor, reducers and rigid parts share one path; Timoshenko shear included), sparse assembly and faer's Cholesky (Mesh's `sparse.rs`), supports as stiff springs along their directions (rigid = 1e12 N/m), imposed displacements through the same springs, one factorisation for all load cases, and the equilibrium check (forces and moments) for every case.
- **Load cases** (`cases.rs`): basic solutions (W, T+D for each case, hydrotest weight) combined linearly: OPE = W + T + D, SUS = W (with pressure for the stresses), EXP = OPE − SUS.
- **Code** (`code.rs`): hoop stress and minimum wall, sustained longitudinal stress with 0.75i, displacement stress range with the in-plane and out-of-plane SIFs, the allowable range with f from the cycles (basic or liberal). Each function cites its equation.
- **Isometric** (`iso.rs`): a 30° projection with dimension placement and sheet splitting; the window draws it as SVG.
- **Window**: copied from Mesh (Plan's skeleton): ribbon, File page, Options, print, updater; the element table at the bottom, the 3D view (three.js) in the centre, the Inspector on the right, Results, Isometric and Report views.

## Size

One overnight run, in seven steps (brief §5), releasing 0.1.0 (shell), 0.1.1 (solver, 3D view), 0.1.2 (components, code checks, results), 0.1.3 (isometric, report, examples), 0.1.4 (polish).

## Questions

None: the brief answers them. Anything unclear is decided simply and recorded in `docs/MEMORY.md` › Decisions.
