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
| `crates/core/src/relief.rs` | Relief valve thrust at an open discharge (ideal gas, sonic exit, dynamic load factor); `ReliefResult` in `Results.reliefs` |
| `crates/core/src/flange.rs` | Flange leakage by the equivalent pressure |
| `crates/core/src/vessel.rs` | Pressure vessels: parts, thin-shell pressure equations, MAWP, MAP, test, weights, outline |
| `ui/src/vessel.tsx` | The Vessels view: design data, parts table, drawing, results, report pages |
| `crates/core/src/export.rs` | Export of the model as a piping component file (.pcf), round-trip tested against the importer |
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
| 0.4.3 | `Settings.spectrum` [(Hz, g)], `rs_dir`, `damping` (0.05); in `solve_model` after the cases: per mode and direction u = φ Γ Sa / ω² (Sa linear in f, held beyond the ends), section forces from `end_force` with no loads, reactions from `dyn_sys` (K with touching conds) at held dofs, springs and conds; CQC (Der Kiureghian) across modes, SRSS across directions, per component; a synthetic `LoadCase` OCC SPEC (Occ, Range{OPE1, OPE1} so it takes OPE1's temperature) in a local `cases` list for the stress loop; `CaseResult.spectrum` = true, imbalance not computed | Reuses the occasional check unchanged; no missing-mass correction yet |
| 0.4.4 | `Settings.harmonic: Option<Harmonic{node, f, m, freq}>`; after the spectrum: u = K⁻¹F (on `dyn_sys`) + Σ φ(φᵀF)[1/(ω²−Ω²+2iζωΩ) − 1/ω²] (mode acceleration, so the static part is exact with few modes); section forces and reactions of the real and imaginary parts, combined as amplitudes; synthetic case HARM (Ope, so not stress-checked; `spectrum` = true). A warning if it is set with no modes; `doc::finish` drops it when its node goes | Force excitation only (no imposed displacement at a frequency), one load at one frequency (no sweep) |
| 0.5.1 | `Element.cold` (m, straight pipe only, < L/10) as an initial strain −c/L on its segments (`Part.cs`, `cs_rhs`); `Recipe::Sum.cs`; with any cold spring the cases get COLD (Ope: W + P1 + CS), OPE/OCC/HYD have CS, SUS does not, EXP = OPE − COLD | The range is unchanged (no credit, as the codes say); full cold spring in hot reactions (no 2/3 factor: the analysis gives the real loads); the hanger sizing runs leave it out |
| 0.5.2 | `Settings.code: PipingCode` (serde "b31.3"/"b31.1", default process) and `Settings.occ_k` (1–2, default 1.33; choosing a code in the UI fills its usual value: 1.33 process, 1.2 power). Power: `sustained_power` = |Fa|/A + P Do/4t + max(0.75 i, 1) M/Z, `expansion_power` = i Mc/Z, M with torsion, i = max(ii, io); same wall, hoop, SA and liberal forms | Classic published form; |Fa|/A added (conservative); one i per point as the code's appendix gives (in-plane for bends, out-of-plane for tees); sustained on the corroded wall (conservative, like the process code) |
| 0.5.3 | `PipingCode::European` ("en13480"): σ1, σ2, σ3 by the power-code functions (same forms: single i, 0.75 i ≥ 1, resultant M with torsion); `wall_y` = 0.5 (e = p Do/(2fz + p)); `range_allow`: fa = U(1.25 fc + 0.25 fh), liberal = fa + (fh − σ1); U = the same 6 N^−0.2 form as f | The standard's text could not be opened from the session (the preview host is blocked; a government comparison report and other software's help were not used). The forms are the long-published ones; every factor (k, f, U) is the user's input or marked to check against the edition. Owner should know this |
| 0.5.4 | `soil.rs`: `Soil` (depth to centre, γ, φ, c, f, α, yield movements, own ultimates) with resistances from first principles: Tu friction of K0 (Jaky) pressure, Pu Rankine Kp − Ka on the face, Qu soil block + side shear, Qd bearing (Prandtl–Reissner Nq, Vesić Nγ). `Model.soils`, `Element.soil` (0 = above ground), `Op::SetSoils` (drops element references). In `solve`: `SoilSp` at both ends of each buried segment for half its length (axial, horizontal lateral, vertical up/down with separate k and fu; vertical pipe gets two laterals); in `solve_conditional` a state per spring (±1 elastic side, ±2 yielded) with `classify`; yielded = constant force (K_OPEN in soft mode). `CaseResult.soil` (per point, element); `reacted` adds `soil_sum`; ranges subtract; modes add max(k); wind skipped on buried segments. Unit `uweight` (kN/m³, lbf/ft³) | No ALA/government guideline used (owner's rule); the textbook formulas from first principles instead, every value overridable. Rankine lateral is lower than depth-dependent bearing charts: the user can give their own |
| 0.5.5 | `vessel::Rules` ("default"/"en13445") per vessel; `pressure_by` uses e = P Di/(2fz − P), P Di/(4fz − P), P Dk/((2fz − P) cos α) for cylinder, hemisphere, cone (MAWP inverted), else `pressure`; test = max(1.25 MAWP ratio, 1.43 MAWP); test limit sy/1.05; warning for dished heads | The torispherical β function (and ellipsoidal equivalence) could not be checked against an open source from the session (standard previews and the background document are on blocked hosts; a pirated copy was not used), so dished heads keep the default rules with a warning. To add: the owner can give the clause or open the network |
| 0.4.2 | `rstate.modeNo` (cleared on load); the 3D view takes the mode's shape in place of the case's displacements, 8 % of the model's size, animated sin(t) about still; stress colours and legend off while a mode shows | A mode's amplitude is arbitrary |
| 0.4.1 | `Settings.modes` (0 = off); `System::modes` subspace iteration (q = min(2p, p + 8) vectors, K factorised once, M-orthonormal Gram–Schmidt applied to X̄ and K X̄ together, Jacobi on the projected Kr), `jacobi()`; lumped translational masses from `Part.mass_len` (weight arithmetic with g = 1, so g = 0 models still have mass); one-way/gap supports as touching (k or RIGID), friction out; `Results.modes` (freq, period, mass-normalised shapes, Γ, Γ²/M) and `Results.mass` (moving mass, unheld points only) | No new dependency; the static solve untouched; modes off by default |
| 0.3.8 | `Vessel.pe`, `ext_fs` (3), `ring_spacing`, `ring_i`; `cylinder_collapse` (von Mises short tube in closed form, floored by Bryan's long tube), L = tangent length + a third of each head's depth or the ring spacing; heads as spheres R = 0.9 D (2:1), crown (torispherical), D/2 (hemi), 0.1875 E (t/R)²; each min with hoop yield 2 Sy t/D, ÷ factor; ring I ≥ F pe R³ Ls / (3E) | Classical theory only, per the owner's rule: no material charts |
| 0.3.7 | `VNozzle.link` = a piping `Nozzle` id; `vessel::link_checks(m, cases)` in `analyze` → `Results.vessel_links`: neck (pipe at 87.5 % less corrosion) P d/(4tn) (no pressure in EXP/OCC) + |Fa|/A + √(Mb² + Mc²)/Z, with torsion by max shear, ≤ 1.5 S; the window shows them on the Vessels page from the piping results | The piping engine gives the loads; the vessel side checks what it can without copyrighted local-stress methods |
| 0.3.7 | Fixed: the 3D view's cleanup now calls `forceContextLoss()` and disposes the scene's geometry | Leaking a WebGL context per view switch crashed the UI suite (and could crash a long session) |
| 0.3.6 | `Vessel.lifting` (`Lifting`: lugs and trunnions, impact, Sy) and `Vessel.legs` (`Legs`: n, len, A, Z, r, circle, Sy) as `CheckRow`s; lug load = impact × empty × g / n / cos φ; allowables 0.6/0.4/0.9/0.66 Sy; legs P = W/n + 4M/(n D), Euler K = 2 / factor 2 capped 0.6 Sy, bending V/n × L, interaction; legs take the base at −len for the moments; no skirt with legs | Statics and Euler only, with the usual working-stress fractions |
| 0.3.5 | `Vessel.flanges` (`BodyFlange`): ring (loose, Y Mo/(t² B), Y = (0.66845 + 5.71690 K² log K/(K² − 1))/(K − 1)) and blind (G √(0.3 P/S + 1.9 W hG/(S G³))); b = b0 up to 6.35 mm else 2.52 √b0 (mm), G = gasket OD − 2b (or the mean); Wm1, Wm2, Am, W = (Am + Ab) Sa/2. Weld-neck (hub factors F, V, f) not done: their closed forms could not be verified from here | Only what could be checked against known values (Y at K = 1.5 = 4.96) |
| 0.3.5 | Fixed: the nozzle editor multiplied the library's sizes (already in metres) by 0.0254; the schedule boxes now show their value (drawn after the library loads) | Found by a new UI check written for the display bug |
| 0.3.4 | `Vessel.skirt` (`Skirt`), `wind_cd` (0.7), `wind_dfac` (1.2), `seis`, `emod` (195 GPa), `base_el`; `check_in(v, settings)` uses the model's wind (`check` = no wind); `supports()` integrates wind and seismic over 400 slices from the base (outline radius + t; bottom head inside the skirt), masses spread over each part's span, extra over the tangent length; larger moment of wind or seismic; compressive allowable min(S, 0.0625 E t / R) (no material charts) | Statics and classical buckling only; the site's wind is shared with the piping |
| 0.3.3 | `Vessel.saddles` (`Saddles`: a, θ, b, welded, rings); `zick_k` from the closed forms (Δ = π/6 + 5θ/12; α = 0.95(π − θ/2); β = π − θ/2; K6 quarter value at A/R ≤ 0.5, linear to the full at 1), unit-tested against the published 120° table; M1, M2 by the beam formulas; stiffened when A ≤ R/2 or rings (K3 shear in shell and head) else K2 shear and K1/K1* sections; horn by L ≥ 8R or not; bottom K5 with 0.1 when welded; weight = max(operating, full of water) with the design pressure | The free articles were blocked by the network; the closed forms reproduced every published 120° constant, which is the check |
| 0.3.3 | Small openings exempt from the area calculation (finished d ≤ 60.3 mm, or ≤ 88.9 mm in walls ≤ 10 mm); the neck is still checked | The long-standing small-opening rule |
| 0.3.2 | `Vessel.nozzles` (`VNozzle`: part, pos, angle, nps, od, t, pipe, leg, pad); `NozzleResult` by area replacement with tr at E = 1 and the part's design pressure (with static head), A1 = max(d, 2(t + tn))(t − tr), A2 = 5(tn − trn) min(t, tn), A41/A42 = leg², A5 = (min(Dp, 2d) − d − 2tn) te; pipe necks at 87.5 %; neck minimum max(trn + c, min(tr + c, 0.875 t_std + c)); heads use their own tr with E = 1 (conservative); no inward projection, fr factors 1 (same material) | The classic method, simply and on the safe side; plate necks have no standard-wall minimum |
| 0.3.1 | `vessel.rs`: `Vessel` (design data, parts bottom→top), `Part` (kind, D, len, t, E), `check` → `VesselResult` (per part: static head at its lowest point, t_req, t_min, MAWP, MAWP at top, MAP, test stress vs 0.9 Sy; vessel MAWP = min at top, MAP, test 1.3 MAWP S_cold/S, volume and weights from profiles turned about the axis, outline for the drawing). `Model.vessels` via `Op::SetVessels`; shell `vessel_check`; the window refreshes on every edit and load | Vessels are independent of the piping solve: instant, and they live in the same file |
| 0.3.1 | Heads only first or last; cones up to 30° half angle; torispherical crown ≤ D, knuckle ≥ 6 % D; head depth from the geometry (0.16934 D for L = D, r = 0.06 D) | Where these equations apply |
| 0.3.1 | The window's edit and load hooks are lists (every hook runs) | A single slot let the second results hook replace the first: results were not marked stale with Analyze automatically off |
| 0.3.0 | **Owner, later on 2026-11-01:** the vessel formulas may be taken from what is known of ASME VIII-1, **without naming it anywhere public**: vessel text describes the equations themselves (thin-shell membrane theory, the area replacement rule, Zick's saddle method, the bolted flange method). Allowables and code values stay inputs. Owner gave the go-ahead for 0.3.0 → 0.4.0 in one run and wants a worked vessel example (like a vessel design program's) first | The owner's words |
| 0.3.0 | Source audit: piping equations from published papers (von Kármán 1911, Clark–Reissner 1951, Markl 1952, Rodabaugh–George 1957), physics (Barlow/Lamé, beam theory, Hooke, drag, momentum), dimensions (facts), the flange leakage equation now with its statics derivation; turbine criteria as published equations with every number an input; no government publication in the app (the regulator's dynamic benchmarks were only planned and are dropped) | Nothing found that has to be removed |
| 0.3.0 | **Owner's rules, 2026-11-01.** (1) One app: everything (piping, vessels, and what follows) lives in Datum Pipe; no separate Datum Vessel. (2) Only what is open source or free to the public is used: no proprietary sources and no government publications. A code's equations are taken from free public sources or derived from first principles and checked to agree, never copied from a paid standard; code tables stay user inputs. (3) Vessels come before dynamics; vessels follow ASME VIII-1 first (EN 13445 later). (4) External pressure only by a method that is legal and free (classical shell-buckling theory). | The owner's words; clean and neat, nothing violated |
| 0.2.6 | **First rule (owner, 2026-11-01): never compare Datum Pipe with commercial software, or name any, in anything public** (README, CHANGELOG, site.json, help, commits, release notes, examples). Comparisons for the owner stay in the conversation only | The owner's most important rule, on top of FAMILY §13 |
| 0.2.6 | Vessel nozzle local stresses are not implemented: the methods in use are copyrighted bulletins (charts and curve fits) and no free closed form was found. Flexibility is the user's stiffness values; a calculated option waits on reading the public ORNL report (ORNL/Sub/2913-6, 1979), whose host is blocked by this environment's network | Never from memory, never from copyrighted charts |
| 0.2.6 | `Nozzle.stiff` [axial, shear, torsion, longitudinal bending, circumferential bending] and `Nozzle.vessel` (serde default, 0 = rigid / vertical); an anchor at a flexible nozzle's node uses `nozzle_restraints` (axis a, circumferential c = a × v, longitudinal l = c × a) instead of its global directions | Reuses the support machinery: imposed growth, reactions and nozzle loads unchanged |
| 0.2.7 | `Comp::Bellows(Bellows)` (axial, lateral, angular, torsion, area, tied, mass); the segment's stiffness overridden in `Props.over` by `bellows_beam` (EA = ka L, GJ = kt L, EI = kθ L, κGA from the lateral rate; 0 = the pipe's value × the rigid factor; tied = axially rigid, no thrust) | A beam with the maker's rates reuses the exact-flexibility element, its loads and its end forces |
| 0.2.7 | Untied thrust P·A at both ends (cases with pressure 1–3 and the hydrotest), in the independent load sum; no code stress on a joint segment; area 0 = π/4 OD² | The anchors must see the thrust; the bellows is not pipe |
| 0.2.8 | `Settings.wind_profile` [(Y, Pa)] interpolated and held beyond its ends, averaged along each segment (9 points); `Settings.has_wind()`, `wind_at()` | A profile from the site's standard as input; no standard's tables |
| 0.2.8 | `Settings.occ_both` (default false, so old files keep their cases) and `Recipe::Sum.neg` (serde default): wind and seismic reversed in OPE1−WIN / OCC −WIN and −SEIS; relief thrust is never reversed | Nonlinear supports make the reversed case differ |
| 0.2.9 | `export::pcf` writes from the analysis geometry (trimmed straights, one elbow per bend with the model node as corner, tees with ends at OD/2 of the run from the centre); support names carry their kind; the shell's `export_pcf` returns the text and the window saves it with `save_file` | The inverse of the importer, so a round trip is testable; no new file dialog code |
| 0.2.4 | Flange leakage by Kellogg's equivalent pressure (`flange.rs`): `Model.flanges` (element, G, rating) via `Op::SetFlanges`; per Sum case the largest P_eq over the element's segment ends from |Fa| and the bending resultant; ratio (P + P_eq)/rating | A public method with every flange value the user's input; |Fa| is on the safe side |
| 0.2.5 | Relief thrust from first principles (`relief.rs`): momentum and pressure thrust of an ideal gas at a sonic exit, F = DLF (ṁV + (P − Pa)A); matches the published API RP 520 Part II form (unit test in US units, < 0.5 %) | Physics anyone can derive, so nothing copyrighted; the standard is named by number only |
| 0.2.5 | `Model.reliefs` via `Op::SetReliefs`; a point force at the open end in cases with `Recipe::Sum.relief` (serde default); cases OPE1+RV and OCC RV like wind; direction along the pipe out of the open end unless given; a non-open end without a direction is an analysis error | The same pattern as the other occasional loads; no silent guess at a direction |
| 0.2.5 | Unchoked exit: pressure term 0, sound speed kept | Overstates the thrust (safe side); said in the help and the results |
| 0.2.5 | New units `mflow`, `area`, `speed` in the display table | Inputs and results of the relief valves |
| 0.2.4 | No flange check in the hydrotest (and none in ranges) | A hydrotest is judged against a test rating; the example's valve failed only there |
| 0.2.3 | `Model.equipment` (`equip.rs`), edited whole with `Op::SetEquipment` like the load cases; nozzles that disappear are dropped from it in `finish` and on repair | No new item kind in every op; equipment is a few entries |
| 0.2.3 | Turbines and compressors: the criteria as equations with every number a pre-filled input (`Coef`: 3, 1, 500; De cap 8 in, (16 + D)/3 above; 2, 1, 250; Dc cap 9 in, (18 + Dc)/3 above; component limits 50, 125, 100, 250, 125, 125 × Dc; compressor multiplier 1.85). The single-nozzle form, the combined form, the resolution at the exhaust and Dc from the combined area were confirmed against published descriptions; the over-cap rules and the component limits could not be (the sources were not reachable from the session), so they are inputs to check against the edition | The owner's rule: equations only, never a standard's tables; anything unconfirmed is an input |
| 0.2.3 | Pumps: components and resultants against the user's table values in the pump's axes (x shaft, y up, z = x × y), with a multiplier; the pump standard's combined interaction equations are not implemented | The table is copyrighted; the interaction rules could not be confirmed in the session |
| 0.2.3 | Equipment is checked in Sum cases only (not ranges); a case with a failing equipment check no longer shows PASS | Ranges are not loads on the machine |
| 0.2.2 | Over 64 supports with friction (`EXACT_FRICS`), sliding forces use the node's own stiffness block as the flexibility estimate; after 40 solves (`FAST_ITER`) the exact unit-load columns take over | The exact columns cost one solve per sliding direction (1276 columns, 2 s per solve on 1441 elements); the estimate only changes how fast it settles, not the answer |
| 0.2.2 | The element table draws only the rows in view (25 px rows, 15 rows of margin) with spacer rows; a selection elsewhere scrolls its row into view | 1441 rows of inputs and selects took seconds of layout |
| 0.2.2 | The 3D view merges every still mesh sharing a material into one buffer (positions, normals, indices; triangle ranges map back to the tag for picking); straight pipe is a cylinder; models over 600 segments use 10 radial facets and 6 per arc | Thousands of draw calls and curve sampling dominated |
| 0.2.2 | The grey blocks in the 3D view under the test display (WebKitGTK on Xvfb, software GL) do not appear in Chromium: left as a quirk of that display | Not reproducible elsewhere |
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
| 0.2.2 | Speed on large models (table, 3D view, analysis), with speed checks | ✅ |
| 0.2.3 | Pump, compressor and steam turbine nozzle checks (published equations; allowable tables are inputs) | ✅ |
| 0.2.4 | Flange leakage (equivalent pressure method) | ✅ |
| 0.2.5 | Relief valve thrust (open discharge reaction, occasional) | ✅ |
| 0.2.6 | Vessel nozzle flexibility from the user's stiffness values (local stresses not done: copyrighted bulletins) | ✅ |
| 0.2.7 | Expansion joints (bellows: axial, lateral, angular, torsional stiffness; pressure thrust) | ✅ |
| 0.2.8 | Wind pressure by height (user table); static seismic in both directions | ✅ |
| 0.2.9 | .pcf export | ✅ |
| 0.3.0 | Milestone: 0.2.1–0.2.9 reviewed; no new features; **source audit**: every equation, value and benchmark checked against the open-sources rule (2026-11-01), anything that fails replaced by a first-principles derivation or a free public source, or removed | ✅ |
| 0.3.1 | Vessels: a Vessel workspace in Datum Pipe; shells and heads under internal pressure (cylinder, sphere, 2:1 ellipsoidal, torispherical, hemispherical, cone), required thickness, MAWP, test pressure, report; equations of ASME VIII-1 taken from free public sources | ✅ |
| 0.3.2 | Vessels: nozzle reinforcement (area replacement) and nozzle neck thickness | ✅ |
| 0.3.3 | Vessels: horizontal vessels on two saddles (Zick's published method) | ✅ |
| 0.3.4 | Vessels: vertical vessels: weight, wind, seismic, skirt, base ring, anchor bolts | ✅ |
| 0.3.5 | Vessels: body flanges (the published Taylor-Forge method) | ✅ ring and blind; weld-neck later |
| 0.3.6 | Vessels: lifting lugs, trunnions, leg supports | ✅ |
| 0.3.7 | Vessels and piping linked: the piping model's nozzle loads checked on the vessel's nozzle | ✅ |
| 0.3.8 | Vessels: external pressure and stiffening rings, from classical shell-buckling theory in free public sources (no code charts) | ✅ |
| 0.4.0 | Milestone: vessels reviewed | ✅ |
| 0.4.1 | Dynamics: natural frequencies and mode shapes (subspace iteration), mass fractions | ✅ |
| 0.4.2 | Dynamics: modes drawn and animated in the 3D view | ✅ |
| 0.4.3 | Dynamics: response spectrum (user spectrum, mode combination, occasional check) | ✅ |
| 0.4.4 | Dynamics: harmonic response (force and moment at a node at a frequency, damping) | ✅ |
| 0.5.0 | Milestone: dynamics reviewed; benchmarks by closed-form theory (not the regulator's problem set: government publication) | ✅ |
| 0.5.1 | More codes: cold spring | ✅ |
| 0.5.2 | More codes: power piping code (B31.1) equations, code chosen per model | ✅ |
| 0.5.3 | More codes: European piping standard (EN 13480-3) equations | ✅ |
| 0.5.4 | More codes: buried pipe (soil springs from first-principles soil mechanics) | ✅ |
| 0.5.5 | More codes: European vessel standard (EN 13445-3) cylinders, spheres, cones, test pressure (dished heads: default rules) | ✅ |
| 0.6.0 | Milestone: more codes reviewed | ✅ |
| 0.7 | 🔜 Load combinations, sustained case with operating support states, more operating cases, other pipe size standards, jacketed pipe | ⬜ |
| 1.0 | Validation report, user guide, signed installer | ⬜ |

**Backlog: owner said "we should not miss out anything"** (each becomes small steps when its stage starts; every item from free public sources only):
- Hydraulics: pressure drop (Darcy–Weisbach, Colebrook), pumps and their curves, pipe networks; line sizing (velocity and pressure-drop limits).
- Relief valve sizing (gas, steam, liquid: nozzle-flow physics); control valve sizing (Cv/Kv, choked flow).
- Storage tanks: shell courses by the one-foot method, wind girders, tank wind and seismic.
- Heat exchanger tubesheets; fitness for service of corroded vessels (only if free public methods exist).
- Vessel nozzle local stresses by our own shell finite element model (Datum Mesh's solver), instead of the copyrighted bulletins.
- Water hammer and surge (after dynamics); thermowells; orifice plates.
- Dynamics, later: missing-mass correction for the spectrum, a frequency sweep and imposed motion for harmonic loads, time history (relief valve opening, water hammer forces), separate spectra per direction, rotary inertia.
- Not planned: thermal design of heat exchangers (proprietary correlations); plant 3D layout (we exchange .pcf files instead).

**Proposed 2026-11-08, awaiting the owner: the window, from Datum Mesh (0.10.2)** (owner: "dig out things from Datum Mesh… improve the UI, the interfaces, the usability… small steps, 3-digit versions"). Mesh's lesson: build each screen against an approved mock-up and compare side by side before release. Steps proposed: mock-ups first (scratchpad, not the repo); 0.6.1 Messages panel and log (every edit, solve, warning kept) and a title bar that says what the file is; 0.6.2 model tree with counts and a filter, resizable panes; 0.6.3 bottom-panel tables (nodes with coordinates, supports, loads) with paste from a spreadsheet; 0.6.4 bar above the 3D view (case, colour by any result or property, render mode, views) and a dark results legend card; 0.6.5 results tree with largest values, click-to-query in 3D, step through overstresses; 0.6.6 player bar (deformed shape, modes, harmonic); 0.6.7 graphs along the line and a material manager with curves; 0.6.8 Home and examples in folders with search, model checker list (click to locate); 0.6.9 output processor (cases × reports to Excel/CSV, report builder); 0.7.0 usability milestone. The old 0.7 (load combinations…) would move to 0.8. Engine items to reuse from Mesh: progress events and Stop (`progress.rs`), results packing (`wire.rs`), Newmark time history (`engine/dynamics.rs`) for later water hammer/relief opening; Mesh's shells for vessel nozzle local stresses later (a link, not copied code).

**Held for later: the family plant-design workflow** (owner, 2026-11-02: "keep this in memory; put it in the roadmap later; finish piping and vessels first"). Datum Schema (P&IDs, line list, specs, DEXPI) → a separate 3D layout app (working name Datum Layout: routing, equipment, its own library, construction isometrics) → Datum Pipe (stress, stress isometrics) → results back. Datum Pipe does no 3D modelling. Files only, never linked code. What Datum Pipe would need:
1. Source IDs and tags kept on elements, supports and nozzles; a line number per element (now one per model). Small.
2. Plant coordinates: a model origin, East/North/Elevation shown in tables and the iso. Small.
3. Richer .pcf import: line number, spec, schedule, material, insulation, temperatures and pressures, support tags and types from the attributes (today geometry only). Medium.
4. A published open JSON interchange file for the family: several lines per system, three operating conditions and hydrotest, density, equipment nozzles with growth, support types, IDs. Medium.
5. Re-import with merge: new geometry from Layout keeps the stress engineer's supports, springs, cases and settings; shows what changed upstream. Large, the most important.
6. Systems of several lines joined at shared points. Medium.
7. Results out (CSV/Excel and the interchange file): support loads and movements by tag, spring data, nozzle loads by equipment nozzle, support changes, stress status per line for Schema's line list. Medium.
8. Read Schema's pipe-spec file (spec → material, schedule, corrosion). 9. Equipment nozzle positions and growth from Layout, allowables from Schema's datasheets.
Order when it starts: 1, 2, 3, 7 first (useful with outside layout tools too); 4, 5, 6 with Layout's first export (write the Layout brief in the hub with the interchange format as the contract); 8, 9 later. Schema already has `links.rs::to_pipe` (a straight run between anchors, for an early check); keep it.

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

### 2026-10-31: Session 2, continued (0.2.2: speed)
- Measured (`examples/perf.rs`, `tests_ui/perf.cjs` on `examples::big(160)`): analysis 16.6 s → 0.18 s, window open 7.2 s → 1.8 s, edit 2.4 s → 0.6–0.75 s, code stresses table 1.2 s. Engine speed gate on the large rack; `npm run perf:ui` for the window. A Coulomb's-law test on a 30-bay rack guards the fast friction path.
- Disk: the build cache filled the allowance again; `target/release` and incremental removed.

### 2026-10-31: Session 2, continued (0.2.3: rotating equipment)
- Engine: `equip.rs` (Equipment, Coef, PumpAllow, check, size_at), `Model.equipment`, `Op::SetEquipment`, check/repair; nozzle loads collected per case into `CaseResult.equipment`. Benchmark 19 (52 checks), unit tests by hand. Examples: K-201 (compressor, passes), T-301 (turbine, fails by 6.5: a lesson in the example's notes).
- UI: Supports › Equipment Checks dialog (machines, nozzles, shaft, multiplier, coefficients, pump table), Results › Equipment and its report block, case badges and the analysis message count equipment; help, Sources; two UI checks.
- Disk: `cargo clean` (19.8 GB) and CARGO_INCREMENTAL=0 for the session.

### 2026-10-31: Session 2, continued (0.2.4: flange leakage)
- Engine: `flange.rs`, `Model.flanges`, `Op::SetFlanges`, check/repair/finish pruning; `CaseResult.flanges`. Benchmark 20 (54 checks). Example: the pump's valve (0.84 in OPE1).
- UI: Loads › Flange Leakage dialog, Results › Flanges and its report block, case badges and the analysis message count flanges; help and Sources; two UI checks.

### 2026-10-31: Session 2, continued (0.2.5: relief valve thrust)
- Engine: `relief.rs`, `Model.reliefs`, `Op::SetReliefs`, check/repair/finish pruning, `Recipe::Sum.relief`, cases OPE1+RV and OCC RV, the thrust as a point load (and in the independent load sum), `Results.reliefs`. Benchmark 21 and the new example's equilibrium (58 checks), two worked tests, the US-units unit test. Example: Safety valve vent stack (3.61 kN, OCC RV 0.139). Units mflow, area, speed.
- UI: Loads › Relief Valves dialog, Results › Relief and its report block, the case editor's Relief thrust option, the example tile; help and Sources; five UI checks. Checked in the real app (the dialog was too wide at first: narrowed).

### 2026-11-01: Session 2, continued (0.2.6: vessel nozzle flexibility; roadmap to 0.2.9)
- Owner reviewed the state of the app; gave the first rule (no comparisons with commercial software in public), chose option 1 for 0.2.6 and allowed releases up to 0.2.9, with gaps added to the roadmap.
- Engine: `Nozzle.stiff`/`vessel`, `nozzle_restraints`, check/repair; benchmark 22 (61 checks), worked test. Example: the tank outlet's shell flexibility.
- UI: Wall flexibility in the nozzle Inspector, help topic; one UI check; the relief dialog test now closes with its Close button. Checked in the real app (labels shortened).

### 2026-11-01: Session 2, continued (0.2.7: expansion joints)
- Engine: `Comp::Bellows`, `Seg.bellows`, `Props.over`, `bellows_beam`, thrust per pressure case, no stress points on joints, iso kind "bellows"; benchmark 23 and the example's equilibrium (65 checks), worked test. Example: Expansion joint between anchors (60 kN on each main anchor).
- UI: component in the table and the ribbon, Inspector rates and Tied, 3D convolutions and tie rods, isometric zigzag, example tile, help and Sources; three UI checks. Checked in the real app.

### 2026-11-01: Session 2, continued (0.2.8: wind by height, both directions)
- Engine: `wind_profile`, `occ_both`, `Recipe::Sum.neg`, check/repair, the sign through the right-hand sides and the section loads; benchmark 24 (68 checks), worked test. The steam rack example uses both directions.
- UI: Wind by height editor and the both-directions box in the Inspector, reversed options in the case editor, the report's settings line, help; four UI checks. Checked in the real app.

### 2026-11-01: Session 2, continued (0.2.9: .pcf export)
- Engine: `export.rs` (`pcf`), round-trip tests on every example and on random models without overlapping pipe (the importer refuses pipe lying on pipe). Shell: `export_pcf`.
- UI: File › Export tile, command search entry, help; one UI check. Checked in the real app.
- Stage 0.3 features are all in (0.2.1–0.2.9); next is the 0.3.0 review.

### 2026-11-01: Session 2, continued (planning: vessels and the whole plant)
- Owner asked for vessel calculations (as in vessel design software) and a plan to cover more of plant design in one app. Research by web search (the Perplexity connector here cannot search). Owner decided: one app (Datum Pipe), ASME VIII-1 first, vessels before dynamics, external pressure only by a legal free method, and only open or free public sources: no proprietary or government ones.
- Roadmap rewritten: 0.3.0 review with a source audit, 0.3.1–0.3.8 vessels, 0.4.0 milestone, dynamics to 0.5, codes 0.6, options 0.7; the backlog lists every other area so nothing is lost. Example problems for 0.3.1 worked by hand: shell 1.5 MPa, R 750 mm, SE 117.3 MPa → t 9.665 mm; MAWP corroded (t 11, R 753) 1.699 MPa; 2:1 head 9.603 mm, hemisphere 4.801 mm, torispherical (L = D) 17.0 mm.

### 2026-11-01: Session 2, continued (0.3.0: review and source audit)
- Full checks green; sources audited (decision above); Help › Sources gains the flange leakage derivation. Owner: go from 0.3.0 to the vessel milestone 0.4.0 without stopping; vessel formulas as known, not named in public text.

### 2026-11-01: Session 2, continued (0.3.1: vessels, shells and heads)
- Engine: `vessel.rs` with unit tests by hand (shell 9.665 mm, 2:1 head 9.603, hemisphere 4.801, torispherical, cone, MAWP 1.699 MPa; head volumes; static heads), `Model.vessels`, `Op::SetVessels`, check/repair; benchmark 25 (73 checks). Example: Vertical separator V-101.
- UI: the Vessels view (left bar), drawing, results, report pages, help and Sources; the edit hook fix; six UI checks. Checked in the real app (layout reworked: form and drawing side by side, results full width).

### 2026-11-01: Session 2, continued (0.3.2: vessel nozzles)
- Engine: `VNozzle`, `nozzles()` in `vessel.rs`, unit test of the rules; benchmark 26 (76 checks). V-101 gains N1–N3 and M1 with pads (first try failed as a real check would: heads need pads, Sch 80 4" neck too thin, a STD 20" manway neck too thin → Sch 160, plate neck).
- UI: nozzle editor (pipe size and schedule or plate), nozzles on the drawing, Nozzles results table; four UI checks. Checked in the real app.

### 2026-11-01: Session 2, continued (0.3.3: saddles)
- Engine: `Saddles`, `zick_k`, `saddle_check`; small openings; unit test of the 120° constants; benchmark 27 (79 checks; first try had the long-vessel horn form, but D-201 is short: L 9 m < 8R 9.64 m, the engine was right). Example: Horizontal drum D-201.
- UI: saddle editor, saddles drawn, Saddles table, wider drawing for horizontal vessels; three UI checks. Checked in the real app.

### 2026-11-01: Session 2, continued (0.3.4: vertical vessels)
- Engine: `Skirt`, `supports()`, `compressive_allow`, `check_in`; benchmark 28 (83 checks: wind base shear from the projected outline matched to the newton, seismic within 1e-3). V-101 gains its skirt, bolts, 1 kPa wind and 0.15 g.
- UI: the Wind, seismic and skirt editor, skirt and base ring drawn, the section table; two UI checks. Checked in the real app.

### 2026-11-01: Session 2, continued (0.3.5: bolted flanges and covers)
- Engine: `BodyFlange`, `flange_check`, `ring_y`; unit test and benchmark 29 (86 checks). V-101's manway cover.
- UI: flanges editor and results table; the nozzle-size unit bug and the schedule box display fixed with UI checks. Checked in the real app.

### 2026-11-01: Session 2, continued (0.3.6: lifting and legs)
- Engine: `Lifting`, `Legs`, `lifting_check`, `legs_check`, `CheckRow`; unit test of the legs, benchmark 30 (88 checks). V-101's lugs.
- UI: lifting and legs editors, the Lifting and legs table; a UI check. Checked in the real app. (An insertion left a stray `fn cd0` line; the compiler caught it.)

### 2026-11-01: Session 2, continued (0.3.7: vessels and piping together)
- Engine: `VNozzle.link`, `LinkCheck`, `link_checks`, `Results.vessel_links`; V-101 gains its 8" inlet line (piping + vessel in one model); benchmark 31 and the separator's equilibrium (90 checks).
- UI: Piping column in the nozzles table, Piping loads table on the Vessels page, failures counted in the analysis message; the WebGL context leak found by the UI suite crashing once the separator gained piping, fixed with a 30-switch UI check. Disk filled (18 GB target); `cargo clean` freed it.

### 2026-11-01: Session 2, continued (0.3.8: external pressure)
- Engine: `ExternalRow`, `cylinder_collapse`, `external()`; unit test, benchmark 32 (92 checks). D-201 at full vacuum with rings every 3 m.
- UI: external pressure inputs and table; a UI check. Checked in the real app.

### 2026-11-01: Session 2, continued (0.4.0: vessels milestone)
- A hardening test of 200 random vessels (every shape, nozzles, saddles/skirt/legs, flanges, lifting, vacuum): checked with finite results or refused, and round-tripped through the file. Full checks green (92 benchmark checks).

### Summary of the vessels stage (0.3.0–0.4.0)
- Owner's rules recorded: one app, open or free public sources only (no proprietary or government), vessel formulas as known without naming the code in public text, no comparison with commercial software anywhere public.
- Vessels: shells/heads/cones (0.3.1), nozzles (0.3.2), saddles (0.3.3), vertical vessels on skirts (0.3.4), bolted flanges and covers (0.3.5), lugs/trunnions/legs (0.3.6), the piping link (0.3.7), external pressure (0.3.8). Examples V-101 (with its inlet line) and D-201.
- Bugs found and fixed on the way: edit hooks overwritten (stale results), nozzle sizes 40× too small in the editor, schedule boxes blank, a WebGL context leak per view switch.
- Not done (on purpose or for later): weld-neck flange hub factors (closed forms unverified here), vessel local stresses at nozzles (copyrighted bulletins; later by our own shell model), tubesheets, fitness for service.

### 2026-11-01: Session 2, continued (0.4.1: natural frequencies)
- Owner: "start 0.5 dynamics". Stage split into 0.4.1–0.4.4 and 0.5.0.
- Engine: `weight_g`, `Part.mass_len`, `System::modes`, `jacobi`, `Mode`; benchmark 33 (94 checks: the 6 m fixed-fixed span was 1.1 % low from shear flexibility, a real Timoshenko effect, so the benchmark uses a slender 10 m span); worked test; speed gate (20 modes of big(160) in 1.6 s). The rack's lowest modes are loops rocking about the line's axis (rests and guides hold no pipe rotation): real for the model, noted in help.
- UI: Dynamics in Analysis settings, Results › Modes; the moving mass counts unheld points only. Steam rack example finds 10 modes.

### 2026-11-01: Session 2, continued (0.4.2: modes in the 3D view)
- UI: Show per mode, Stop in the legend; three UI checks. Checked in the real app (mode 1 of the steam rack swinging).

### 2026-11-01: Session 2, continued (0.4.3: response spectrum)
- Engine: settings, the spectrum case, benchmark 34 (95 checks: base shear within 0.21 % of 0.6131 m L Sa). Steam rack gets a spectrum (OCC SPEC 0.463).
- UI: spectrum editor under Dynamics, "not applicable" in the equilibrium table; two UI checks. Checked in the real app.

### 2026-11-01: Session 2, continued (0.4.4: harmonic response)
- Engine: `Harmonic`, case HARM, benchmark 35 (96 checks: a 10 t mass on a 3 m 6" cantilever with a 20 mm rigid end, driven at half its frequency; the hand model first missed the 20 mm arm and was 2.3 % off: fixed by carrying the pipe's end deflection and turn out to the load). Steam rack gets a 500 N pulsation at 60, 12 Hz (HARM max 0.37 mm). Warning when harmonic is set with no modes.
- UI: Harmonic load editor under Dynamics (node checked, force, moment, frequency; remove/add); the damping ratio moved out of the spectrum so it shows for both; seven UI checks.
- Bug found in the real app: Summary showed "< 1e-12" equilibrium for OCC SPEC (and HARM); now "not applicable", with a UI check. HARM's nozzle loads are not judged pass/fail (amplitudes); its Summary check reads "Amplitudes, not checked".

### 2026-11-01: Session 2, continued (0.5.0: dynamics milestone)
- Review of modes, spectrum and harmonic code: CQC coefficient, spectrum interpolation, input checks and repairs read correct.
- New tests: harmonic at 0 Hz equals static with 2 or 10 modes, linear in the load, finite at resonance (5–12 × static); random models run dynamics on seeds ≡ 1 mod 4 and must give ≥ 10 OCC SPEC/HARM cases.
- Found by the review: the printed report had no dynamics input, no modes table and no HARM support loads (HARM is Ope, filtered out). Fixed, with a UI check. The Equilibrium table now says HARM's values are amplitudes (it said "combined").

### 2026-11-02: Session 2, continued (cleanup, no functional change)
- A CSS comment in `ui/src/styles.css` named another product; it now describes the styles only (ribbon.tsx had no such comment). README and NOTICE gain: "Product and file-format names are trademarks of their owners and are used only to describe file compatibility."

### 2026-11-02: Session 2, continued (brainstorm: the plant-design workflow)
- Owner asked what Datum Pipe needs if Schema (P&ID) → a new layout app → Pipe (stress). Read Datum Schema (read-only clone): it has DEXPI, pipe specs, the line list and `to_pipe`. The answer is held in the roadmap backlog ("Held for later"); no code changed. Owner: finish piping and vessels first.

### 2026-11-02: Session 2, continued (0.5.1: cold spring)
- Owner: "go with the 0.6, complete all the things": cold spring, power piping, European piping, buried pipe, European vessels. Split into 0.5.1–0.5.5 and 0.6.0.
- Engine: `Element.cold`, `Recipe::Sum.cs`, case COLD, benchmark 36 (98 checks), worked test on an L-bend (superposition, range unchanged).
- UI: Cold spring in the Inspector for straight pipe, a Cold spring column in Load Cases, CS in recipes and the report; four UI checks.

### 2026-11-02: Session 2, continued (0.5.2: power piping code)
- Engine: `PipingCode`, `occ_k`, power piping sustained and range, benchmark 37 (100 checks with the example's equilibrium row: bending plus torsion under pressure), unit test by hand; example "power" (boiler steam to a header).
- UI: Piping code and occasional factor in Analysis settings; `codeName`, `checkText` (≤ k Sh) in results and report; help and Sources; README/NOTICE trademarks; five UI checks.

### 2026-11-02: Session 2, continued (0.5.3: European piping standard)
- Engine: `PipingCode::European`, `wall_y`, `range_allow`; benchmark 38 (102 checks); unit test by hand.
- UI: the third code in the choice, CEN in the report's disclaimer, help and Sources; one UI check.
- The standard's own text was not reachable; told the owner.

### 2026-11-02: Session 2, continued (0.5.4: buried pipe)
- Engine: `soil.rs`, soil springs in the nonlinear iteration, soil loads in results and equilibrium; benchmark 39 (Hetényi 0.40 %, friction 0.06 %; 105 checks); hardening buries every other element on 1 in 7 random models; example "buried".
- UI: Loads › Soils dialog, Buried in on elements, Results › Soil, report soils and soil loads, earth-coloured buried pipe in 3D, help and four Sources rows; six UI checks.

### 2026-11-02: Session 2, continued (0.5.5: European vessel rules)
- Engine: `Rules`, `pressure_by`, European test pressure and test limit; benchmark 40 (108 checks); unit test by hand.
- UI: Rules select on the Vessels page, the rules in the report line, help and Sources; two UI checks.
- Not done: the European torispherical rule (its β function could not be read from an open source here); told the owner.
- 0.5.4 has no release of its own: GitHub returned 502/500 to both triggers and no run was created; the workflow builds the branch head, so 0.5.5 carries it (said in the 0.5.5 notes).

### 2026-11-02: Session 2, continued (0.6.0: more codes milestone)
- Review of 0.5.1–0.5.5. Found and fixed: `Op::Split` copied a cold spring to both halves (now shared in proportion); `Op::Loop` copied it to bends (now on the last straight). Worked tests for both; hardening runs every piping code and cold springs, random vessels both rules.
- 0.5.4 shipped inside 0.5.5 (release service errors); said in the notes.

## 5. Known limits (0.2.0)

- One-way supports are rests only (+Y); a guide or restraint with a gap acts both ways.
- Friction is the same in every direction across a support (no separate coefficients), and is path-independent: each case slides from the installed position.
- Each case finds its own support states; there is no option to use the operating states for the sustained case.
- No pressure thrust on straight pipe from bends (only bellows). Wind has no shielding and acts on a bend's chord; seismic is one static vector (both directions as an option, no per-axis cases); occasional loads go with operating case 1 only.
- The deformed shape moves arcs' vertices by the chord-interpolated displacement (fine to 45° per arc segment; bends are split in two).
- Bends' pressure opening (Bourdon effect of curved pipe) is not modelled; pressure elongation is for straight pipe only and off by default.
- Minimum wall: t ≥ D/6 fails rather than using a thick-wall equation; bends are checked at the intrados only.
- Two rigid supports holding the same direction at one node share the reaction equally (the true split is indeterminate).
- Spring hangers are sized from operating case 1 only, hold vertically only, and use one spring each (no double springs or load-to-travel charts).
- The .pcf import reads geometry, sizes, in-line items and supports; schedules, materials, temperatures and pressures in the file are not read (set them in Datum Pipe).
- Pump nozzle checks are component and resultant checks against the entered table values; the pump standard's combined interaction equations are not applied.
- Relief valve thrust is for an ideal gas at an open discharge only (no liquids, two-phase flow or closed discharge systems), acts with operating case 1 only, and is static (times the load factor; no time history).
- The .pcf export writes geometry, bores, weights and supports only; expansion joints come back as rigid parts and springs or restraints as rests on import.
- Expansion joints: one bellows per element, no hinged or gimbal joints, no movement or cycle check against the maker's limits.
- Vessels: external pressure by classical buckling with a single design factor (no plasticity interaction beyond the yield cap); cones under external pressure treated as cylinders of their large diameter; wind and seismic are static, not combined, and the larger is used; no wind-induced vibration; skirt-to-head weld and openings in the skirt not checked; saddles: two equal saddles, no wear plates, stiffening rings not themselves checked, compression checked against S (no buckling allowable); nozzle reinforcement without inward projection, same material for neck, pad and shell, nozzles not counted in the weights; heads formed from one plate (no crown/knuckle thickness split); weights from the inside surface times the thickness.
- Nozzle flexibility is the user's stiffness values only; vessel local stresses are not calculated.
- Response spectrum: no missing-mass (rigid) correction, components combined separately (signs lost), one spectrum for all directions with factors.
- European vessel rules: torispherical and ellipsoidal heads by the default rules; no European nozzle (pressure-area), external pressure, flange or saddle rules yet.
- Buried pipe: soil springs at segment ends (finer segments for accuracy), elastic–perfectly plastic, path-independent per case; Rankine lateral (no depth factor), no soil overburden on the pipe, no code rules for restrained buried pipe; spectrum and harmonic reactions leave the soil out.
- Harmonic response: one force and moment at one node at one frequency (no sweep, no imposed displacement), equal damping in every mode; its amplitudes are not stress-checked and nozzle allowables are not applied to them.
- Natural modes: lumped translational masses (no rotary inertia), supports linear and touching, no friction; rests and guides do not hold the pipe's rotation about its axis, so long lines can show slow rocking modes.
- The equilibrium check cannot find stiffness, SIF or axis mistakes (each element is self-equilibrated); the benchmarks and worked cases test those.
