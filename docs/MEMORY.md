# Datum Pipe: project memory

> **Paste this file at the start of each new session**, or just open the repo: `CLAUDE.md` tells Claude to read it.
> It records what exists, why, and what comes next. Update it at the end of every session.

| | |
| --- | --- |
| **Product** | Datum Pipe: pipe stress analysis on the desktop (element table, 3D view, linear supports, the process piping code's equations, support and nozzle loads, isometric, stress report) |
| **Current version** | 0.1.4 (2026-10-28) |
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
| `crates/core/src/import.rs` | Import of plain-text piping component files (.pcf) into a new model (`pcf`), with its warnings |
| `crates/core/src/iso.rs` | The isometric: the 30° projection (x = (X − Z) cos 30°, y = (X + Z) sin 30° − Y), lines per segment (arcs sampled), node labels, a dimension per element offset away from the middle, symbols for supports, nozzles and tees, sheets of at most n elements in table order with their joins |
| `crates/core/src/worked.rs` | Hand-worked cases for every component and support kind (spring, settlement, guide, line stop, rest, skewed restraint, valve weight, constant support, reducer taper and SIF, tee SIF, nozzle axes, liberal allowable and f) |
| `tests_ui/screens.cjs` | The four README and site screenshots (1440×900) from the built window with the engine's real results |
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
| 0.2.1 | The sparse solver marks a mechanism with the free unknown ("[dof n]": the zero pivot, the largest probe answer, or a 1e-9-stiffened copy's largest answer); `locate_free` turns it into the node and direction; the window selects that node | The most common question a new user has: where is it free? |
| 0.2.1 | Analyze automatically is a per-user option (on by default): 400 ms after an edit, 300 ms after opening; quiet (no tab switch, toasts only when something is wrong and changed) | Results that follow the typing; fast enough on every example |
| 0.2.1 | Typed units: the engine sends every unit symbol with its factor, offset and dimension (`units::typed`) and each display quantity's dimension; `parseQ` in `ui/src/units.ts` adds number-unit pairs (' and " for feet and inches), refuses a wrong dimension | The engine stays the only source of factors |
| 0.2.1 | 3D view: right-click menus; a node within 14 px of the pointer wins over the pipe under it, for left and right clicks | Nodes are small targets |
| plan | Stage 0.3 split into 0.2.1–0.2.6 with 0.3.0 as the milestone; the owner asked for ease of use and efficiency too, so those come first (0.2.1, 0.2.2) | Small releases, as the owner asked; usability before new checks |
| 0.2.0 | Touching one-way or gapped supports held exactly on one unknown share its reaction equally (`CondSolve.share`), with the shared-support warning | Found by the review; the same rule as linear supports |
| 0.2.0 | A custom bend radius ≤ D/2 is refused by `check` (and repaired on open); `bend_factor` returns an infinite intrados factor there so a wall check fails | Found by the review: the factor turned negative |
| 0.2.0 | Hanger travel = the pipe's movement minus the hanger's own imposed movement in case 1 | Found by the review: C must give H in operation |
| 0.2.0 | The .pcf import splits a piece at any support or olet point inside it, even if another piece already starts there | Found by the review: olet branches were unconnected |
| 0.1.11 | The regulator's piping benchmark problems (NUREG/CR-1677, Brookhaven 1980) are response-spectrum problems with no static results, so they move to 0.4 | A benchmark is only worth adding when this version can solve it |
| 0.1.11 | `.pcf` import in `crates/core/src/import.rs`: a new unsaved model; nodes merged within 1 mm and numbered 10, 20, … in order of first appearance; X east, Y north, Z up → (x, z, −y); elbows at the corner with R = tangent / tan(θ/2); tees and olets as tees at the centre (set-on and olets fabricated); supports split the pipe they sit on; rests one-way, ANC/GUI/STOP in the name → anchor/guide/line stop; nominal bore → library NPS, Std wall | The common content of these files, with everything the user must check named in the notes and the warnings |
| 0.1.11 | Public words name the format by its extension only (.pcf, "piping component file"), not any tool; the header line naming a tool is not needed and is matched generically | FAMILY §13 |
| 0.1.10 | `SupKind::Hanger { k, cold }` (0 in the file); `analyze` sizes hangers first (`solve_model` is the plain solve): weight alone with hangers as rigid Y restraints → H; OPE1 recipe with hangers as constant H → Δ; then `choose_spring` | The usual hanger design runs, kept inside the engine so the window never calculates |
| 0.1.10 | Choice: the first table row (in the user's order) with H and C = H + kΔ inside [min, max] and |kΔ|/H ≤ variability (25 %); none → constant-force of H with a warning; H ≤ 0 → constant 0 with a warning | Simple, predictable, and the user controls preference by ordering the table |
| 0.1.10 | The final solve models a hanger as a Y spring of rate k plus its cold load as an upward force in every case with weight (like a constant support), reported together on the hanger | Then OPE1 gives C − kΔ = H, which benchmark 18 checks |
| 0.1.10 | `Settings.springs` (default `typical_springs()`: series A/B/C with 25/50/100 mm travel, 20 sizes, max from 400 N ×1.5 each, min = max/1.8, k = (max − min)/travel) and `Settings.variability`; generic, not a catalogue; the library command returns the generic table for "Back to the generic table" | No maker's data; the user replaces it |
| 0.1.9 | Wind and seismic are model settings (`wind_q`, `wind_cd`, `wind_dir`, `seis`): one uniform design pressure (no height profile, no shielding) and one seismic vector in g; both user inputs, no standard's tables | Simple, honest inputs; the site standards stay with the user |
| 0.1.9 | Wind per metre = q Cd D (w − (w·t) t) with D over insulation and cladding and t the segment's chord (arcs use their chord); seismic per metre = the operating weight × the factors | Drag on a cylinder across the flow; chord keeps one uniform load per segment |
| 0.1.9 | Recommended: OPE1+WIN / OPE1+SEIS (Sum with `wind`/`seis`) and OCC WIN / OCC SEIS as ranges from OPE1, with case 1 only | The occasional effect from the operating state (supports as in operation), the usual practice |
| 0.1.9 | An occasional range's stress = the largest SUS stress at the point + the sustained expression of the range's loads without pressure, against 1.33 Sh (Sh at the case's hot temperature); an occasional Sum (a user's own) is checked as before | B31.3 para. 302.3.6 used as an equation; the scalar sum is on the safe side |
| 0.1.9 | `Recipe::Sum.wind`/`seis` and the settings have serde defaults, so older files open unchanged; a `wpress` display quantity (kPa, psf) | Compatibility |
| 0.1.8 | `Support.mu` on rests, guides, line stops and restraints; friction acts across the support's own directions (a rest: horizontal; a guide on a horizontal pipe: along it and vertical; a vertical pipe's guide: along it), with the load pressing it = the size of the support's reaction | Isotropic Coulomb friction, the usual model |
| 0.1.8 | Sticking holds the pipe across (exactly along a global axis, else a 1e12 N/m spring) unless something else already holds that direction; it slides when the force needed exceeds μ N | Exact like the other holds, so the benchmark matches to rounding |
| 0.1.8 | A sliding force is solved exactly for each support from its own flexibility (unit loads in the same solve): direct in one direction, bisection on the movement's size in two; supports are updated one after another with the displacement corrected through their flexibility (Gauss–Seidel) | Re-pointing the force each solve oscillated on flexible lines and between supports at one node |
| 0.1.8 | N is taken from the previous solve (lagged); friction is in every Sum case, including SUS (where it usually sticks with no force) | Simple, and converges with the states |
| 0.1.8 | When a case does not settle, the loads returned are those of the last solve's states, so it still balances (with the warning) | Found by the hardening tests: the reported states ran one step ahead |
| 0.1.8 | The 1500-element speed gate runs without friction; a 300-element random model with friction has its own gate | A 3 km random chain held at one end trips the solver's mechanism check once rests slide; it times the solver, not friction |
| 0.1.7 | `Support.one_way` (rests) and `Support.gap` (rests, guides, line stops, restraints); new rests from the window and the examples are one-way, files without the field stay two-way | Lift-off is the realistic default, and old files keep their results |
| 0.1.7 | Each Sum case is solved by iteration with its own support states (start: touching for a one-way rest without a gap, open for a gap; change every wrong state each solve, at most 100); ranges stay OPE − SUS of the settled results | The usual practice; ranges as differences keep the code's displacement range meaning |
| 0.1.7 | A touching rigid support along a global axis is held exactly at its gap (prescribed value); flexible or skewed ones are springs with an offset | Same exactness as linear supports, so the benchmarks match to rounding |
| 0.1.7 | When open gaps leave a mechanism, the open supports get 1 N/mm and the case is solved again, with a warning; the small loads are reported | A straight run held sideways only by gapped guides is common and must solve |
| 0.1.7 | Linear-only models keep the one-factorisation path, bit for bit | No change to results or speed for models without one-way or gapped supports |
| 0.1.6 | The Windows publisher is "Datum" (`bundle.publisher` in `src-tauri/tauri.conf.json`); the copyright stays "© 2026 Edgar Mendonca" | A family-wide change (FAMILY §9 in the hub); nothing else in the repo names the publisher |
| 0.1.5 | Public words about the code: "calculates using the published equations of ASME B31.3", "contains none of the code's text, tables or figures", "not affiliated with or endorsed by ASME", ASME and B31.3 named as trademarks; Help › Sources (`SOURCES` in `ui/src/help.tsx`) lists each equation with an open-literature reference, and says plainly where an equation is the code's own (reducer SIF, the 1.25/0.25 range factors, Ts) | The owner asked to be safe from any use of copyrighted material: equations and physical facts are not protected, the code's wording, tables and figures are, and none is in the app. Citations were checked against library and journal records before use; Lorenz (1910) was left out as unconfirmed, the bend factor is derived from the torus instead |
| 0.1.5 | Bend wall: an extra check per bend at its intrados with I = (4R/D − 1)/(4R/D − 2); the extrados (I < 1) is not listed, the straight check covers it | The intrados governs; one line per bend keeps the table short |
| 0.1.5 | t ≥ D/6 fails the wall check, with a warning, rather than switching to a thick-wall equation | The code asks for special consideration there; the app says so instead of guessing |
| 0.1.5 | Ze = π r₂² Ts at a tee's branch end (setting `branch_ze`, on), for both sustained (corroded walls) and expansion (nominal) | The earlier editions' rule; editions that use the branch pipe's own Z turn it off |
| 0.1.5 | Pressure elongation (setting `pressure_elongation`, off) is added to the thermal strain of straight, non-rigid, non-reducer segments in cases 1–3, so it is in OPE and in EXP = OPE − SUS, never in SUS; bends' Bourdon opening is not modelled | Off keeps every earlier model's results unchanged; the strain-in-OPE convention is the usual one |
| 0.1.5 | Two rigid supports holding one direction at a node share its reaction equally, with a warning | The split between rigid supports is statically indeterminate; equal shares keep each support's load plausible and the sum right |
| plan | The 0.2 stage is split into the small releases 0.1.5–0.1.11; 0.2.0 is a milestone release with no new features | The owner asked for many small steps instead of one big 0.2 release |
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
| 0.1.4 | An independent review of the engine (a separate agent, told to report only) before the last release; every confirmed finding fixed with a regression test in worked.rs | Two real bugs no test had caught: an expansion range between cases took Sh at the higher-numbered case, not the hotter one (unsafe), and a line stop or guide at a tee averaged the branch into its direction |
| 0.1.4 | A guide or line stop takes the run through its node (two opposite directions), or the only segment at an end; at a corner it warns and uses the first element | There is no single axis at a corner; saying so is better than guessing silently |
| 0.1.4 | Tees take their SIF from the run's pipe and their plane from the run and branch (from the first two legs, with a warning, when there is no straight run) | A tee marked on its branch element used the branch size (unsafe for a small branch) |
| 0.1.4 | Bend SIF pressure correction with the lowest operating pressure, flexibility with the highest | The SIF correction lowers stresses, so the lowest pressure is safe for every case; one stiffness keeps one factorisation |
| 0.1.4 | A flexible anchor's stiffness is translational; its rotations stay rigid | One number in N/m cannot also be a rotational stiffness |
| 0.1.4 | The equilibrium help says what the check proves (loads applied, equations solved) and what it does not (stiffness, SIFs, axes) | The review pointed out that a self-equilibrated element always balances; the benchmarks are what test the physics |
| 0.1.3 | The element table keys its rows by position (Solid's `Index`), not by item | Every edit returns a new model, so keyed rows were all rebuilt and the cursor was lost after each cell (found by a UI test that types across a row) |
| 0.1.3 | The loop wizard puts the loop in the middle of a straight element, sideways (horizontal plane) or upwards, with long-radius bends, and refuses a loop that does not fit its bends | The common case in one dialog; anything else is typed in the table |
| 0.1.3 | Copy Section joins the copy's first node to a chosen node and numbers the rest after the largest node; Mirror flips one axis of every delta (and of skewed restraints) | Simple to explain; covers repeated runs and symmetric layouts |
| 0.1.2 | Iso tags: one per element, on its longest line; dimensions node to node (centre to centre), offset away from the drawing's middle | Bends have two arc segments and drew two tags each |

## 3. Roadmap

Each stage is reached through small 0.x.y steps; the next minor version (0.2.0, 0.3.0, …) only marks a finished stage. If a step grows, it becomes two steps (0.1.11, …) rather than one big release. Later stages are split the same way before they start.

✅ done · 🔜 next · ⬜ later

| Version | Item | Status |
| --- | --- | --- |
| plan | Read the hub, Plan, Mesh, Calc and Atlas; PLAN-0.1, MEMORY, CLAUDE, icon | ✅ |
| 0.1.0 | Shell, updater, File page, Options; model, edits, library, units, cases, hardening tests; element table, Inspector, property dialogs, print of the input; CI and release | ✅ |
| 0.1.1 | Engine: 3D pipe element, bends, supports, solve, equilibrium, code checks; benchmarks 1–8 (21 checks); 3D view; results tables; examples L-bend and Z-bend; Help › Verification | ✅ |
| 0.1.2 | Components checked by hand (worked.rs), deformed shape and animation, nozzle checks, the four other examples, the isometric (view, print, SVG), the results pages of the report | ✅ |
| 0.1.3 | Loop wizard, copy and mirror, table typing fixes, report and isometric polish, hydrotest and occasional checks | ✅ |
| 0.1.4 | Polish and the review fixes; screenshots, README, site; every example analyzes, draws and prints in a UI test | ✅ |
| 0.1.5 | Code check fixes: t < D/6 limit and bend thickness factor, reducers at both ends, tee effective branch section modulus, pressure elongation, each support's own share of a shared reaction; Help › Sources and the legal wording | ✅ |
| 0.1.6 | Windows publisher "Datum" (family-wide) | ✅ |
| 0.1.7 | One-way rests and gaps (iterative solve), lift-off shown in the results, benchmarks | ✅ |
| 0.1.8 | Friction at rests and guides, equilibrium with friction, a friction benchmark | ✅ |
| 0.1.9 | Wind and static seismic loads, occasional cases and the occasional stress check | ✅ |
| 0.1.10 | Spring hanger selection (spring data entered by the user) | ✅ |
| 0.1.11 | Import of plain-text piping component files (the regulator benchmarks moved to 0.4: they are dynamic) | ✅ |
| 0.2.0 | Milestone: 0.1.5–0.1.11 reviewed, verification record and examples updated; no new features | ✅ |
| 0.2.1 | Ease of use: where a model is free, analyze automatically, values typed with units, right-click menus in the 3D view | ✅ |
| 0.2.2 | Speed on large models (table, 3D view, analysis), with speed checks; the grey blocks the 3D view draws for the compressor example under the test display | 🔜 |
| 0.2.3 | Pump, compressor and steam turbine nozzle checks (published equations; allowable tables are inputs) | ⬜ |
| 0.2.4 | Flange leakage (equivalent pressure method) | ⬜ |
| 0.2.5 | Relief valve thrust (open discharge reaction, occasional) | ⬜ |
| 0.2.6 | Vessel nozzle flexibility and local stresses, only from published equations | ⬜ |
| 0.3.0 | Milestone: 0.2.1–0.2.6 reviewed; no new features | ⬜ |
| 0.4 | Dynamics: modes, harmonic, response spectrum; the regulator's public piping benchmark problems (NUREG/CR-1677) | ⬜ |
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

### 2026-10-28: Session 1, continued (0.1.4: polish, review, the build to start with)
- Released 0.1.3 (Release run 4; CI green).
- Polish: the isometric fills the window, the report's title block (a stray CSS rule pushed its labels to the foot of page 1, found by inspecting the printed page), stresses in table order, the steam example's nozzle axis, a hydrotest case in the pump example, screenshots (`tests_ui/screens.cjs`), README and site.json.
- An independent review of the engine found two bugs and four smaller issues (see Decisions 0.1.4); all fixed, each with a regression test. Not changed: Sp added outside |Fa/Ap| (conservative), no t < D/6 check or bend thickness factor in the minimum wall, the reaction of two supports holding the same direction at one node is reported on the first.
- Checked: clippy, 33 engine tests (benchmarks 25 checks, worked cases, hardening), 48 UI checks, cargo deny, the real app under Xvfb (WebGL 3D view, analysis, isometric, print preview, updater).

### Summary of the overnight run
- **Released:** 0.1.0 (shell, table, model file), 0.1.1 (solver, 3D view, results, verification), 0.1.2 (components checked, four more examples, isometric, full report), 0.1.3 (loop wizard, copy and mirror, table typing), 0.1.4 (polish and review fixes).
- **Works:** the whole brief §2 except what is listed below as cut: element table, 3D view, components, pipe library, materials, supports, three operating cases and hydrotest, automatic load cases, the 3D pipe element solver with bends' flexibility, the B31.3 equations (hoop and minimum wall, sustained, expansion range, basic and liberal allowables), results on the model and in tables, support and nozzle loads, the isometric (print and SVG, multi-sheet), the stress report, six worked examples, Help › Verification (benchmarks 1–8, 25 checks).
- **Cut or partial:** occasional loads (the case kind is there, no loads yet, as planned); PDF export is through the print window's Save as PDF; isometric sheets split by table order, not by drawing area; the deformed shape of arcs is interpolated along the chord; no one-way rests (0.2, as planned).
- **Try first:** File › Examples › Pump discharge with an expansion loop; press F5; Results › Nozzle Loads; the Isometric view; File › Print. Then type your own line in the table.

### 2026-10-29: Session 2 (roadmap in small steps)
- The owner asked for small releases: the 0.2 stage became 0.1.5–0.1.10, with 0.2.0 as the milestone. Updated the roadmap here, `docs/site.json` (File › Roadmap) and the "planned for" labels in the ribbon, File page, help and inspector.

### 2026-10-29: Session 2, continued (0.1.5: code check fixes and sources)
- Engine: bend factor at the intrados, the D/6 limit, reducers at both ends, Ze at tee branches (geom `Tee.branch`), pressure elongation, shared rigid reactions; two new settings. Benchmarks 9–11 (29 checks) and six worked cases.
- Legal: README "About the code checks", Help › Sources, NOTICE, report wording; every comment that mentions the code was read again: they give equations as mathematics and point to paragraph numbers only.
- UI: the wall table shows where each check is (pipe, bend, large or small end) with I and the D/6 flag; settings in the Inspector; Sources on the Help tab; four new UI checks.

### 2026-10-29: Session 2, continued (0.1.6: publisher)
- The Windows publisher became "Datum" across the family; released as 0.1.6. The roadmap's one-way rests and gaps move to the next release number.

### 2026-10-29: Session 2, continued (0.1.7: one-way rests and gaps)
- Engine: `Support.one_way`, `Support.gap`, `Support::conditional`; `solve_conditional` (per-case iteration, exact holds, springs with offsets, 1 N/mm fallback); `CaseResult.contacts` and `iterations`. Benchmarks 12–13 (34 checks), four worked cases, random hardening models with one-way rests and gaps.
- Examples: rests one-way, steam guides with 3 mm gaps (all settle with no warnings; results otherwise unchanged).
- UI: Inspector one-way switch and gap; Support Loads state column and solve count; help; ribbon's Nonlinear group now only plans friction. Three UI checks; the mock bridge adds supports.

### 2026-10-30: Session 2, continued (0.1.8: friction)
- Engine: `Support.mu`, `Support::has_friction`; `Fric`, `across`, `FState`, `slide_force`, `Linear`; friction in `solve_conditional` (stick holds, exact sliding force per support, Gauss–Seidel, light stiffness fallback for sliding supports too); `CaseResult.friction`. Benchmark 14 (38 checks), three worked cases, random friction in hardening, a friction speed gate.
- Examples: steam rack rests and guides μ = 0.3 (settles in 10 solves, balanced, no warnings).
- UI: friction coefficient in the Inspector, friction state and force in Support Loads, help; the ribbon's Nonlinear group is gone (nothing left planned there).
- Disk: the session's allowance filled with `target/` (17 GB); `target/release` and `target/debug/incremental` were removed.

### 2026-10-30: Session 2, continued (0.1.9: wind and static seismic)
- Engine: settings `wind_q`, `wind_cd`, `wind_dir`, `seis` (checked and repaired); `Recipe::Sum { wind, seis }`; `q_occ`, `occ_rhs`, `rb_occ` in `solve.rs` with the case's distributed load from `q_of`; the occasional range check; `cases::recommend` adds OPE1+WIN, OCC WIN, OPE1+SEIS, OCC SEIS. Benchmarks 15–17 (43 checks), three worked cases, wind and seismic in the random hardening models.
- Example: the steam rack has wind 0.8 kPa from the south and 0.15 g sideways (OCC WIN 0.49, OCC SEIS 0.39, balanced).
- UI: Occasional loads in the model Inspector, Loads › Wind and Static Seismic go there, an Occasional column in Load Cases, help topic, Sources rows, report settings line; one UI check.

### 2026-10-30: Session 2, continued (0.1.10: spring hangers)
- Engine: `SupKind::Hanger`, `SpringSize`, `typical_springs`, settings `springs`/`variability` (checked and repaired); `analyze` → design runs + `choose_spring` + `solve_model`; `Results.hangers`. Benchmark 18 (48 checks), three worked cases. The tank example's SH-40 is now a sized hanger (A-03).
- UI: Supports › Spring Hanger and Spring Table (dialog with the variation limit and "Back to the generic table"), Results › Hangers and its report block, hanger in the Inspector, 3D and isometric (spring symbol), help and Sources. Two UI checks.

### 2026-10-30: Session 2, continued (0.1.11: import)
- Engine: `import::pcf` with three tests (a sample through to analysis, units and unknown parts, every truncation and line damage of the sample never panicking). Shell: `import_pcf` (file dialog, new unsaved model, the warnings shown on open). UI: the File › Export tile, Tell Me entry, help topic; one UI check. Sample: `docs/examples/6-P-101.pcf`.
- Checked in the real app through the file dialog: the sample imports and analyses.
- The regulator benchmarks are dynamic: moved to 0.4 (site, MEMORY, the Verification dialog's note).

### 2026-10-30: Session 2, continued (0.2.0: review and milestone)
- An independent reviewer (a separate agent, given the diff and MEMORY, told to demonstrate every finding with numbers) read cec9e76..0.1.11. Four bugs, all confirmed and fixed with regression tests: shared reactions of touching conditional supports, bends with R ≤ D/2, hanger travel with a moving attachment, olet branches in the import. It found the rest correct (bend factor, Ze, pressure strain, reactions at held unknowns, contact and friction states, sliding force, wind, seismic, occasional range, hanger sizing, .pcf mapping).
- README and site describe everything to 0.2.0; screenshots, verification record (48 checks) refreshed.

### Summary of the second stage (0.1.5–0.2.0)
- **Released:** 0.1.5 (code check fixes, Help › Sources, legal wording), 0.1.6 (publisher "Datum"), 0.1.7 (one-way rests, gaps), 0.1.8 (friction), 0.1.9 (wind, static seismic, occasional check), 0.1.10 (spring hangers), 0.1.11 (.pcf import), 0.2.0 (review).
- **Works:** every support is linear or settles by iteration (contact, gaps, friction) with balanced loads; occasional loads; hangers sized from the user's spring table; import from design tools; 18 benchmarks (48 checks) and 35 hand-worked cases.
- **Moved:** the regulator's public benchmarks to 0.4 (they are dynamic).
- **Next:** stage 0.3 (equipment and flanges), to be split into small 0.2.x releases first.

### 2026-10-31: Session 2, continued (0.2.1: ease of use)
- Owner asked to start the third stage and to make the app more efficient and easier: the stage was split into 0.2.1–0.2.6 with ease of use (0.2.1) and speed (0.2.2) first.
- Engine: mechanism location in `sparse.rs` (`mechanism(Option<dof>)`, `locate`) and `solve.rs` (`locate_free`); `units::typed` and `Shown.dim`; one worked case. UI: auto-analyze option and quiet solves (`rstate.ts`), `parseQ` and `numInQ` in number fields, table and V3 inputs, 3D right-click menus and node picking; four UI checks. Checked in the real app (node and element menus).

## 5. Known limits (0.2.0)

- One-way supports are rests only (+Y); a guide or restraint with a gap acts both ways.
- Friction is the same in every direction across a support (no separate coefficients), and is path-independent: each case slides from the installed position.
- Each case finds its own support states; there is no option to use the operating states for the sustained case.
- No pressure thrust. Wind is one uniform pressure (no height profile or shielding) and acts on a bend's chord; seismic is one static vector (no ± or per-axis cases built for you); occasional loads go with operating case 1 only.
- The deformed shape moves arcs' vertices by the chord-interpolated displacement (fine to 45° per arc segment; bends are split in two).
- Bends' pressure opening (Bourdon effect of curved pipe) is not modelled; pressure elongation is for straight pipe only and off by default.
- Minimum wall: t ≥ D/6 fails rather than using a thick-wall equation; bends are checked at the intrados only.
- Two rigid supports holding the same direction at one node share the reaction equally (the true split is indeterminate).
- Spring hangers are sized from operating case 1 only, hold vertically only, and use one spring each (no double springs or load-to-travel charts).
- The .pcf import reads geometry, sizes, in-line items and supports; schedules, materials, temperatures and pressures in the file are not read (set them in Datum Pipe).
- The equilibrium check cannot find stiffness, SIF or axis mistakes (each element is self-equilibrated); the benchmarks and worked cases test those.
