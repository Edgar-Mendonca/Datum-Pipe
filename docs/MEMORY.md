# Datum Pipe: project memory

> **Paste this file at the start of each new session**, or just open the repo: `CLAUDE.md` tells Claude to read it.
> It records what exists, why, and what comes next. Update it at the end of every session.

| | |
| --- | --- |
| **Product** | Datum Pipe: pipe stress analysis on the desktop (element table, 3D view, linear supports, the process piping code's equations, support and nozzle loads, isometric, stress report) |
| **Current version** | 0.1.0 (2026-10-28) |
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
| `crates/core/src/doc.rs` | Edit operations (Add, Set, Delete with cascade, SetStart, Split, Renumber, SetSettings, SetMeta, SetCases, RecommendCases, Batch) on a copy, kept only if layout and `check` pass; snapshot undo/redo |
| `crates/core/src/library.rs` | Pipe sizes ½"–24" and schedules (ASME B36.10M/B36.19M dimensions); five materials (carbon, 1¼Cr-½Mo, 304, 316, duplex 2205) with E and α against temperature and typical allowables min(Su/3, 2Sy/3), with sources; `interp` |
| `crates/core/src/units.rs` | Calc's unit table (NIST SP 811) with unit expressions and the display table for SI and US |
| `crates/core/src/cases.rs` | Which operating cases are used; the recommended load cases (OPE1–3, SUS, EXP1–3, EXP ranges between cases, HYD); `rebuild` keeps ids by name |
| `crates/core/src/sparse.rs` | Mesh's sparse solver: triplets, faer Cholesky, probe solve for mechanisms, iterative refinement, residual |
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
| `ui/src/view3d.tsx`, `ui/src/results.tsx`, `ui/src/iso.tsx` | The 3D view, results and isometric (placeholders in 0.1.0) |
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

## 3. Roadmap

✅ done · 🔜 next · ⬜ later

| Version | Item | Status |
| --- | --- | --- |
| plan | Read the hub, Plan, Mesh, Calc and Atlas; PLAN-0.1, MEMORY, CLAUDE, icon | ✅ |
| 0.1.0 | Shell, updater, File page, Options; model, edits, library, units, cases, hardening tests; element table, Inspector, property dialogs, print of the input; CI and release | ✅ |
| 0.1.1 | Engine: 3D pipe element, supports, solve, equilibrium; benchmarks 1, 2, 8; 3D view; examples L-bend and Z-bend; Help › Verification | 🔜 |
| 0.1.2 | Bends, tees, reducers, rigid elements; thermal and pressure cases; benchmarks 3–7; code checks; results colouring, deformed shape, tables, support and nozzle loads | ⬜ |
| 0.1.3 | Isometric (view, print, SVG); stress report; the other examples | ⬜ |
| 0.1.4 | Polish: expansion loop wizard, copy/mirror, renumber, keyboard, empty states, help cards; screenshots, README | ⬜ |
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

## 5. Known limits (0.1.0)

- No solver or 3D view yet (0.1.1).
- Rests are two-way (one-way in 0.2).
