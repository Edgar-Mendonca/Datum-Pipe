# Datum Pipe: project memory

> **Paste this file at the start of each new session**, or just open the repo: `CLAUDE.md` tells Claude to read it.
> It records what exists, why, and what comes next. Update it at the end of every session.

| | |
| --- | --- |
| **Product** | Datum Pipe: pipe stress analysis on the desktop (element table, 3D view, linear supports, the process piping code's equations, support and nozzle loads, isometric, stress report) |
| **Current version** | 0.1.3 (2026-10-28) |
| **Repo** | `Edgar-Mendonca/Datum-Pipe` · branch `claude/datum-pipe-setup-1oa8wk` (the first branch pushed, so the default branch) |
| **Stack** | Rust engine (`crates/core`, own solver with faer), Tauri 2, SolidJS + strict TypeScript, three.js for the 3D view; no Python |
| **Identifier** | `com.edgarmendonca.datumpipe` · files `.dpipe` (JSON, `format` 1) · installer `DatumPipe-x.y.z-setup.exe` + `SHA256SUMS.txt` |
| **Accent** | Moss `#4D7C0F` (`--blue2` `#3F6212`, `--sky` `#BEDC8F`, `--pale` `#F1F7E6`, `--sel` `#E4F0CF`, `--sel2` `#CBE2A4`; dark `#A3D65C`; dark File sidebar `#2A420A`) |
| **Brief** | `Edgar-Mendonca/Datum` › `briefs/DATUM-PIPE.md` (hub session 16): every decision made up front |
| **Reference repos** | `Edgar-Mendonca/Datum` (FAMILY, MEMORY, brain), `Edgar-Mendonca/Datum-Plan` (reference code), `Edgar-Mendonca/Datum-Mesh` (frame solver, sparse solve, verification, the skeleton copied), `Edgar-Mendonca/Datum-Calc` (units engine), `Edgar-Mendonca/Datum-Atlas` (materials data and sources) |

---

## 1. Architecture

`crates/core` is the engine (pure Rust, no Tauri): the model and its edits, the library, units, load cases, and (from 0.1.1) the elements, the solver, code checks, the isometric, the report data and the benchmarks. `src-tauri` is the shell: commands, files, options, recovery and the updater. `ui` is the window: it draws what the engine returns and sends edits as operations.

Units: the engine and the file use SI base units (m, kg, N, Pa, K). The engine's display table (`units::shown`) gives every quantity's unit in SI (mm, N, N·m, MPa, bar, °C) and US (ft for lengths along the pipe, in for sizes and displacements, lbf, lbf·ft, psi, °F), built from Calc's NIST unit table; the window only applies it.

### Files

| File | What it does |
| --- | --- |
| `crates/core/src/model.rs` | The model: nodes (number and position), elements (From, To, DX/DY/DZ, pipe, material, line conditions, component, extra load), pipes, materials (tables against temperature), line conditions (T1–T3, P1–P3, hydrotest), supports, nozzles, loads, load cases, title block, settings; `layout` (positions from the deltas, loops must close); `check` (every rule); `.dpipe` read item by item with migration and `repair` (damaged files open as far as possible, with warnings), and write |
| `crates/core/src/doc.rs` | Edit operations (Add, Set, Delete with cascade, SetStart, Split, Renumber, Loop (expansion loop wizard), Duplicate (copy or mirror a section with its supports), SetSettings, SetMeta, SetCases, RecommendCases, Batch) on a copy, kept only if layout and `check` pass; snapshot undo/redo |
| `crates/core/src/library.rs` | Pipe sizes ½"–24" and schedules (ASME B36.10M/B36.19M dimensions); five materials (carbon, 1¼Cr-½Mo, 304, 316, duplex 2205) with E and α against temperature and typical allowables min(Su/3, 2Sy/3), with sources; `interp` |
| `crates/core/src/units.rs` | Calc's unit table (NIST SP 811) with unit expressions and the display table for SI and US |
| `crates/core/src/cases.rs` | Which operating cases are used; the recommended load cases (OPE1–3, SUS, EXP1–3, EXP ranges between cases, HYD); `rebuild` keeps ids by name |
| `crates/core/src/sparse.rs` | Mesh's sparse solver: triplets, faer Cholesky, probe solve for mechanisms, iterative refinement, residual; `solve_many` (every load case with one factorisation, prescribed values per case) |
| `crates/core/src/mat.rs` | 3-vectors, 6×6 matrices (inverse, move a force to another point), local axes of a pipe, 4-point Gauss |
| `crates/core/src/geom.rs` | The analysis geometry: bends trimmed by R·tan(θ/2) into arcs (near, middle = the bend's node, far points), shared points, tees with their plane; bend radii from NPS; sent to the window for the 3D view |
| `crates/core/src/elements.rs` | The 3D pipe element from its exact flexibility ∫ Bᵀ C B ds (axial, shear with κ = 0.5, torsion, bending with the bend's k), inverted and moved to 12×12; uniform loads and thermal growth as equivalent nodal loads; end forces and section resultants |
| `crates/core/src/code.rs` | The process piping code's equations: pressure design thickness and hoop stress, minimum wall with corrosion and mill tolerance, bend and tee SIFs and flexibility (h, pressure correction), reducer SIF, sustained stress, displacement stress range, allowable range (basic and liberal), f from cycles |
| `crates/core/src/solve.rs` | The analysis: parts (sections, weights, strains, SIFs), assembly, supports (rigid axis-aligned ones held exactly with imposed displacements as prescribed values; skewed or flexible ones as springs), basic loads, every case solved together, ranges as differences, forces in local axes, reactions, code stresses per segment end, nozzle loads in nozzle axes, minimum wall, the equilibrium check |
| `crates/core/src/iso.rs` | The isometric: the 30° projection (x = (X − Z) cos 30°, y = (X + Z) sin 30° − Y), lines per segment (arcs sampled), node labels, a dimension per element offset away from the middle, symbols for supports, nozzles and tees, sheets of at most n elements in table order with their joins |
| `crates/core/src/worked.rs` | Hand-worked cases for every component and support kind (spring, settlement, guide, line stop, rest, skewed restraint, valve weight, constant support, reducer taper and SIF, tee SIF, nozzle axes, liberal allowable and f) |
| `crates/core/examples/summary.rs` | Every example's worst ratios, imbalances, nozzle ratios and reactions, for tuning examples by hand |
| `crates/core/src/bench.rs` | The verification benchmarks 1–8 (`run()`), with an independent plane-frame flexibility method by hand for the thermal cases; a test fails on any row over its tolerance |
| `crates/core/examples/verify.rs` | Writes the verification record (`docs/verification/x.y.z.md`) |
| `crates/core/src/examples.rs` | The worked examples, built in code; `table()` builds a model from rows |
| `crates/core/src/hardening.rs` | FAMILY §11: failed edits change nothing, random models keep their rules and round-trip, damaged files, speed gate |
| `crates/core/examples/dump.rs` | Writes `tests_ui/fixture.json` (the engine's real models, unit table and library) for the UI tests |
| `src-tauri/src/main.rs` | Commands: state, apply, history, examples, new model, library, unit table, open/save, save file, options, recovery, recent, startup file, open_url, version, updates |
| `src-tauri/src/update.rs` | The updater (Plan's, `REPO` = Datum-Pipe) |
| `ui/src/main.tsx` | Window layout (3D view, element table, Inspector; Results, Isometric, Report views), sidebar, status bar, keys, start-up and recovery |
| `ui/src/store.ts` | Window state: the model (`out().m`), the selection (element, node, support, nozzle, load), views, menus, dialogs |
| `ui/src/api.ts` | Shell calls and the model's shapes in TypeScript |
| `ui/src/units.ts` | Applies the engine's display table |
| `ui/src/table.tsx` | The element table: editable rows, the new row that adds an element, keyboard moves, component choices |
| `ui/src/panels.tsx` | The Inspector: element, node (supports, nozzle, loads), support (direction, stiffness, imposed displacements), nozzle (axis, allowables), load, model (title block, analysis settings) |
| `ui/src/props.tsx` | Pipes, materials (tables), line conditions and load cases dialogs |
| `ui/src/ribbon.tsx` | Title bar, quick access, Tell Me, the ribbon (Home, Model, Loads, Supports, Analyze, Results, Isometric, View, Help, contextual Component); planned tools dimmed with their release |
| `ui/src/cmds.ts` | Every command (ribbon, menus, keys, Tell Me) |
| `ui/src/dialogs.tsx` | Save changes, close, recovery, shortcuts, getting started, glossary |
| `ui/src/backstage.tsx` | File page: examples, recent, export, options, updates |
| `ui/src/print.tsx`, `ui/src/report.tsx` | Print (Plan's page: paper, header and footer codes, preview, the report flowed over pages); the stress report's blocks and the Report view |
| `ui/src/view3d.tsx` | The 3D view (three.js): pipes at true diameter from the engine's geometry, arcs, insulation, components, support and nozzle symbols, node labels, picking, standard views, fit, stress-ratio colours and legend, the deformed shape (vertices moved by the interpolated displacements, animated) |
| `ui/src/rstate.ts`, `ui/src/results.tsx` | Results state (last analysis, out of date, case shown, colour and deformation options) and Analyze; the Results tab and view (summary, code stresses, displacements, forces, support loads, nozzle loads, minimum wall, equilibrium) |
| `ui/src/iso.tsx` | The isometric view: each sheet as SVG (symbols, dimensions in mm or ft-in, north arrow, continuation notes, stress ratio and support load tags), sheet tabs, Save as SVG, and the sheets in the printed report |
| `ui/src/tools.tsx` | The expansion loop wizard (with a picture to scale) and copy or mirror a section of table rows |
| `ui/src/help.tsx` | The `?` cards |
| `ui/src/roadmap.tsx` | File › Roadmap from `docs/site.json` |
| `ui/src/styles.css`, `ui/src/ui.tsx`, `ui/index.html` | Plan's styles (moss accent) and components; the icon sprite |
| `tests_ui/run.cjs` | UI checks of the built window with a mocked bridge |
| `tests_ui/realapp.sh` | Screenshots of the real app under Xvfb |
| `docs/PLAN-0.1.md`, `docs/site.json`, `CHANGELOG.md` | The plan, the public summary and roadmap, the change log |

### Release flow

`release.yml` (windows-latest): checks a pushed tag matches the version, takes the notes from CHANGELOG, runs the engine tests, builds the NSIS installer, writes `SHA256SUMS.txt` and publishes the GitHub Release. Run it from the Actions tab (tag pushes are blocked from Claude sessions).

### How we work

1. The user reviews a release and sends screenshots.
2. Reproduce first.
3. Big asks get a plan file and questions; build after the answers.
4. Engine first (with a test), then the UI.
5. Check every change in the real app (Xvfb screenshots, `tests_ui/realapp.sh`).
6. Add a UI test for every bug.
7. Run all checks: clippy `-D warnings`, `cargo test` (with the hardening tests and benchmarks), the UI type-check and UI tests, `cargo deny check`.
8. Release.
9. Report in plain language: what changed, what was checked, what is not done.

## 2. Decisions

| Version | Decision | Why |
| --- | --- | --- |
| plan | Everything in the brief §1 (name, moss accent, identifier, `.dpipe`, stack, own Rust solver, three.js, units, ribbon tabs, logo, codes as published equations, public words) | Made with the user in the hub, session 16 |
| plan | The shell and window are copied from Datum Mesh (Plan's skeleton by way of Sim, already adapted to an engineering-analysis app) | The brief: copy, don't reinvent; Mesh is the closest app |
| 0.1.0 | Y is up; X east, Z south (right-handed). The first node of each connected piece starts at the origin unless moved | The usual piping convention; one rule for the table, the 3D view and the isometric |
| 0.1.0 | Bends and tees are components on the element that ends at their node (a bend turns into the element leaving the node) | One column in the table; matches how a stress sketch is marked up |
| 0.1.0 | Line conditions are named sets (T1–T3, P1–P3, hydrotest) that elements reference, rather than six columns per row | A table that fits the screen; one place to change a line's conditions |
| 0.1.0 | Temperatures are kelvin in the file (SI base), shown in °C or °F | SI base units in the file, as every Datum app |
| 0.1.0 | Typical allowables are worked out from typical strengths as min(Su/3, 2Sy/3) at temperature, up to the family's usual limit (creep range not covered) | The brief forbids copying code tables; this is the published basis of the process piping code's allowables below the creep range, and gives the familiar 138 MPa (20 ksi) for carbon steel |
| 0.1.0 | Modulus and expansion against temperature: Atlas's room values with trends typical for each family, marked with their sources | Atlas has room-temperature values only |
| 0.1.0 | The display table comes from the engine (Calc's units), not from factors in the window | One owner for conversions; the brief says copy Calc's units engine |
| 0.1.0 | SI lengths in mm everywhere; US lengths along the pipe in ft, sizes and displacements in in; pressure in bar (SI) or psi | What piping engineers use in each system |
| 0.1.0 | Load cases are rebuilt after every edit while "automatic" is on (ids kept by name); editing a case turns it off | Always right for a new model; the user stays in control |
| 0.1.1 | Every element (straight, arc, reducer, rigid) from its exact flexibility by quadrature along the centreline, then inverted and moved by equilibrium | One path for every component, exact for straight pipe and for arcs (16 × 4 Gauss points); reducers taper naturally; matches the hand flexibility method to 1e-14 |
| 0.1.1 | Shear deformation with κ = 0.5 (thin tube); the beam-theory benchmarks include the shear term | Short spans near nozzles are common in piping; the references say so |
| 0.1.1 | Rigid supports along global axes hold their degree of freedom exactly (imposed displacements as prescribed values); skewed or flexible ones are springs, a skewed rigid one 1e12 N/m | Exact anchors and rests keep the benchmarks at 1e-14; a penalty only where it is needed |
| 0.1.1 | The modulus at the installed temperature for every case; thermal strain from mean coefficients from 21 °C: α(T)(T − 21) − α(T₀)(T₀ − 21) | The process piping code's practice for displacement stresses; the library's tables are mean coefficients |
| 0.1.1 | Bend flexibility and SIF pressure corrections use the highest operating pressure of the line | One stiffness for every case keeps one factorisation; conservative for the SIF in lower-pressure cases (k is lower) |
| 0.1.1 | Rigid elements are 1000 × the pipe's stiffness; their weight replaces the pipe's metal, the contents and insulation stay | Stiff enough to be rigid, soft enough for a well-conditioned solve |
| 0.1.1 | Tee SIFs apply at every segment end on the tee's point, with each leg's own section modulus | Simple and on the safe side; the branch's effective section modulus can come later |
| 0.1.1 | The equilibrium check is judged against the larger of the loads, the reactions, the model's weight and 1 N | The hardening test found a thermal range on a model with one anchor (no reactions at all) reported as 79 % "imbalance" of rounding noise |
| 0.1.1 | The window gets the analysis geometry (arcs, points) with every model, so the 3D view never works out bends itself | The engine is the one owner of geometry and results |
| 0.1.1 | Releases follow the brief's steps, but 0.1.1 already shows code stresses and the results tables (the engine had them) | Cheap once the solver was there; 0.1.2 is then about the remaining components, results polish and nozzle checks |
| 0.1.2 | The isometric and the full report (brief step 6) came in 0.1.2, with the four other examples; 0.1.3 is their polish and more checks, 0.1.4 the brief's step 7 | They were built while 0.1.1 was releasing; releasing them at once gives the user more to try |
| 0.1.2 | Example supports chosen like a designer would: a line stop instead of a second anchor on the rack, a spring next to the settling tank | The first drafts gave a 3 MN axial load on the rack and 4.7 × the tank nozzle's allowable: correct numbers for poor layouts, but examples should show good practice |
| 0.1.3 | The element table keys its rows by position (Solid's `Index`), not by item | Every edit returns a new model, so keyed rows were all rebuilt and the cursor was lost after each cell (found by a UI test that types across a row) |
| 0.1.3 | The loop wizard puts the loop in the middle of a straight element, sideways (horizontal plane) or upwards, with long-radius bends, and refuses a loop that does not fit its bends | The common case in one dialog; anything else is typed in the table |
| 0.1.3 | Copy Section joins the copy's first node to a chosen node and numbers the rest after the largest node; Mirror flips one axis of every delta (and of skewed restraints) | Simple to explain; covers repeated runs and symmetric layouts |
| 0.1.2 | Iso tags: one per element, on its longest line; dimensions node to node (centre to centre), offset away from the drawing's middle | Bends have two arc segments and drew two tags each |

## 3. Roadmap

✅ done · 🔜 next · ⬜ later

| Version | Item | Status |
| --- | --- | --- |
| plan | Read the hub, Plan, Mesh, Calc and Atlas; PLAN-0.1, MEMORY, CLAUDE, icon | ✅ |
| 0.1.0 | Shell, updater, File page, Options; model, edits, library, units, cases, hardening tests; element table, Inspector, property dialogs, print of the input; CI and release | ✅ |
| 0.1.1 | Engine: 3D pipe element, bends, supports, solve, equilibrium, code checks; benchmarks 1–8 (21 checks); 3D view; results tables; examples L-bend and Z-bend; Help › Verification | ✅ |
| 0.1.2 | Components checked by hand (worked.rs), deformed shape and animation, nozzle checks, the four other examples, the isometric (view, print, SVG), the results pages of the report | ✅ |
| 0.1.3 | Loop wizard, copy and mirror, table typing fixes, report and isometric polish, hydrotest and occasional checks | ✅ |
| 0.1.4 | Polish: keyboard, empty states, help cards; screenshots, README, site; every example solves and prints | 🔜 |
| 0.2 | Nonlinear supports (one-way rests, gaps, friction), spring hanger selection, import of plain-text piping component files, wind and seismic, the public regulator benchmarks | ⬜ |
| 0.3 | Pump, compressor and turbine nozzle checks, flange leakage, local stresses at vessel nozzles, relief valve thrust | ⬜ |
| 0.4 | Dynamics: modes, harmonic, response spectrum | ⬜ |
| 0.5 | Power piping and European piping code equations, buried pipe, cold spring | ⬜ |
| 1.0 | Validation report, user guide, signed installer | ⬜ |

## 4. Session log

### 2026-10-28: Session 1 (overnight run, 0.1.0)
- Read the brief, the hub's CLAUDE, MEMORY, FAMILY, brand and brain; Mesh's code (the skeleton, frame and sparse solver, verification), Calc's units, Atlas's metals (room-temperature values with sources; no temperature tables).
- Wrote PLAN-0.1, CLAUDE.md, this memory, `docs/site.json`, README, CHANGELOG, NOTICE; drew the icon (a pipe run with an expansion loop and an anchor, amber diamond).
- Engine: model, layout, check, repair, edits with undo, library, units, cases; tests including hardening (failed edits, random models round-trip, damaged files, speed gate).
- Window: ribbon, File page, Options, element table, Inspector, property dialogs, load cases, print with the input echo.

- Released 0.1.0 (Release run 1; CI run 1 green on Linux, Windows and macOS).

### 2026-10-28: Session 1, continued (0.1.1: the solver and the 3D view)
- Engine: mat, geom (bends as arcs), elements (exact flexibility), code (the equations), solve (assembly, supports, cases, reactions, stresses, nozzles, walls, equilibrium), bench (8 benchmarks, 21 checks, all within 1e-6; most at 1e-14 against the hand flexibility method).
- Found by the hardening tests: the equilibrium check reported rounding noise as a 79 % imbalance in a thermal range with one anchor (fixed: judged against the model's loads).
- Window: three.js 3D view, Analyze (F5), results tables, legend, Help › Verification. Checked in the real app under Xvfb (WebKitGTK renders the WebGL view; the updater found the published 0.1.0).

### 2026-10-28: Session 1, continued (0.1.2: components, examples, isometric, report)
- Released 0.1.1 (Release run 2).
- Engine: worked.rs (8 hand-worked cases; found my own sign error for a settling anchor's reaction, the engine was right), the four remaining examples (tuned with `examples/summary.rs`), iso.rs.
- Window: the Isometric view and its print pages, the report's results pages, deformed shape checked by screenshot. 33 UI checks.

### 2026-10-28: Session 1, continued (0.1.3: editing tools and polish)
- Released 0.1.2 (Release run 3). CI runs 1–2 green on Linux, Windows and macOS.
- Found by a new UI test: the table lost the cursor after every edit (rows rebuilt); fixed with position-keyed rows. Checked typing rows with the keyboard in the real app.
- Engine: Loop and Duplicate ops with a test; hydrotest and occasional hand-worked cases. Window: loop wizard, copy and mirror dialogs, report page fit, plain report headings, one iso tag per element.

## 5. Known limits (0.1.3)

- Rests are two-way (one-way in 0.2); no friction or gaps.
- No pressure elongation (Bourdon effect) or pressure thrust; no occasional loads yet (the OCC case kind is ready).
- The deformed shape moves arcs' vertices by the chord-interpolated displacement (fine to 45° per arc segment; bends are split in two).
- Tee SIFs use each leg's own section modulus (no effective branch section modulus).
