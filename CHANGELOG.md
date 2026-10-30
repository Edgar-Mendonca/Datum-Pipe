# Changelog

## 0.1.8 — 2026-10-30

Friction at rests and guides.

**Friction.** Rests, guides, line stops and restraints can have a friction coefficient μ (0.3 is typical for steel on steel, 0.1 for PTFE slide plates). It acts across the support's direction: a rest resists the pipe sliding horizontally, a guide resists it moving along the pipe.

**Sticking and sliding.** In each load case a support with friction holds the pipe while the force that needs is within μ × the load pressing the support; beyond that the pipe slides, and the support pushes back with exactly μ × its load against the movement, in whatever direction the pipe moves across it. A rest that lifts off has no friction. This is solved in the same iteration as one-way rests and gaps; sliding forces are found exactly from the support's own flexibility, one support after another, so supports at one node settle together.

**Results.** Support Loads shows each support with friction as sticking or sliding, with its friction force, which is part of the support's load and of the equilibrium check.

**Examples.** The steam line's rests and guides have μ = 0.3.

**Verification.** A new benchmark: friction at a heated rest, sliding (μ P, and the movement ε L − μ P L / EA) and sticking (E A ε) (38 checks).

## 0.1.7 — 2026-10-29

One-way rests and gaps.

**One-way rests.** A rest can let the pipe lift off: it pushes the pipe up but never pulls it down. New rests are one-way; the Inspector switches it. Rests in files from earlier versions stay two-way, so their results do not change.

**Gaps.** Rests, guides, line stops and restraints can have a gap: below the pipe for a rest, on each side for the others. The support takes no load until the pipe has moved across it.

**How it is solved.** With one-way rests or gaps, each load case is solved again until every such support has settled as touching or open: a touching support must push the pipe, not pull it, and an open one must not be passed. The expansion range is still the difference of the operating and sustained results. If only open gaps hold the piping in some direction, they keep a light stiffness (1 N/mm) so the case can be solved, with a warning.

**Results.** Support Loads shows each one-way or gapped support as touching, open or lifted off, with the pipe's movement and the gap, and how many solves the case took.

**Examples.** The examples' rests are one-way, and the steam line's guides have 3 mm gaps.

**Verification.** Two new benchmarks: a one-way rest under a cantilever (lifting off when pushed up, carrying the load when pushed down) and a gap closing at a cantilever's tip (34 checks).

## 0.1.6 — 2026-10-29

Windows now lists the publisher as Datum.

## 0.1.5 — 2026-10-29

Code check fixes, and where every equation comes from.

**Minimum wall.**
- A bend is also checked at its intrados, where the wall needs most metal: the design thickness uses the bend factor I = (4R/D − 1)/(4R/D − 2).
- A pressure design thickness of D/6 or more now fails, with a warning: the thin-wall equation does not apply there.
- A reducer is checked at both ends, each with its own pipe (diameter, wall, corrosion and mill tolerance); it was checked at its large end only.

**Tees.** The branch of a tee uses the effective section modulus Ze = π r₂² Ts, with Ts the lesser of the run's wall and ii × the branch's wall. Analysis Settings can turn it off for code editions that use the branch pipe's own section modulus.

**Pressure elongation.** Straight pipe can grow with its pressure in the operating cases (ε = (σL − ν σh)/E). Off unless you choose it in Analysis Settings; the sustained case never includes it.

**Shared supports.** Two rigid supports holding the same direction at one node now share its load equally, with a warning; the first one took it all.

**Sources.** Help › Sources lists every equation with where it is published in the open engineering literature: Barlow and Lamé, the membrane theory of a torus, von Kármán, Clark and Reissner, Markl, Rodabaugh and George, and beam theory. The README, NOTICE and report say that Datum Pipe contains none of the code's text, tables or figures and is not affiliated with or endorsed by ASME.

**Verification.** Three new benchmarks: a bend's wall at its intrados, pressure elongation between two anchors, and a tee branch's effective section modulus (29 checks).

## 0.1.4 — 2026-10-28

Polish: the build to start with. Every example analyzes, draws its isometric and prints its report.

**Fixes from an independent review of the engine.**
- An expansion range between two operating cases now takes the hot allowable Sh at the hotter of the two cases; it took the second case's even when the first was hotter, which was on the unsafe side.
- Guides and line stops at a tee act along the run; the branch was averaged into the pipe's direction, so a line stop could take loads across the pipe. At a corner, where there is no single direction, Datum Pipe now says so.
- A tee drawn as the end of its branch takes its stress intensification from the run pipe, and a tee without a straight run gets its plane from its legs (with a warning).
- Bends' stress intensification factors use the lowest operating pressure for the pressure correction (on the safe side in every case); their flexibility still uses the highest.
- A flexible anchor's stiffness applies to its three movements; its rotations are held.
- A reducer's small end uses its own pipe's corrosion allowance in the sustained stress.

**Isometric.** The sheet fills the window at the proportions of a landscape page, with its title block the same width.

**Report.** The title block's labels print in their boxes (they were pushed to the foot of the first page).

**Results.** Code stresses are listed in table order, element by element, with each bend after its straight part; Worst first still sorts them by ratio.

**Examples.** The turbine nozzle of the steam line points the right way (from the turbine into the pipe).

**Checked.** A final pass over every worked example: each one analyzes with its reactions balancing its loads, draws its isometric and prints a report of four to six pages. New screenshots for the README and the family site.

## 0.1.3 — 2026-10-28

Editing tools, smoother typing in the table, and a tidier report and isometric.

**Expansion loop wizard.** Select a straight element and choose Model › Expansion Loop: give the loop's width and height, sideways or upwards, and the element becomes a loop in its middle with four long-radius bends, drawn to scale before you make it.

**Copy and mirror a section.** Model › Copy Section copies a range of table rows, with their components and supports, onto any node; Mirror Section does the same mirrored across X, Y or Z.

**Typing in the table.** Tab and Enter keep the keyboard in the table after every edit (before, an edit rebuilt the rows and the cursor was lost), and Enter in the last row adds the element and stays there for the next one.

**Report and isometric.** Report pages keep their content inside the header and footer, headings print in plain type, the isometric shows one stress ratio tag per element (bends showed two), and the 3D view fits the model more closely.

**More checks.** Hand-worked cases for the hydrotest (the line full of water) and for an occasional case (checked against 1.33 Sh).

## 0.1.2 — 2026-10-28

Every component checked by hand, four more worked examples, the isometric drawing, and the results in the stress report.

**Components and supports, checked.** Tees, reducers, valves and flanged joints, springs, guides, line stops, skewed restraints, constant-force supports and imposed settlements each have a hand-worked case: a spring under a cantilever, a settling anchor, a valve's weight, a tapering reducer, a welding tee's intensification, a nozzle's loads in its own axes, the liberal allowable and the cycle factor.

**Isometric drawing.** The Isometric view draws the line as a 30° isometric: pipe lines and bends, valves, flanges, reducers, tees, support symbols, nozzles, node numbers, a dimension along every element (mm, or feet and inches), a north arrow, and tags with the highest stress ratio of each element or the support loads. Large models split across sheets with their joins marked. Print it with the report or save it as SVG.

**The stress report, complete.** After an analysis the printed report adds the stress summary of every case with its worst point, the highest sustained and expansion stresses, the support loads per case, the nozzle loads, the minimum wall, the equilibrium checks, and the isometric sheets, each with the title block.

**Deformed shape.** Results › Deformed Shape draws the pipe moved by the case's displacements, with a scale slider, and Animate moves it between installed and operating.

**Examples.** A pump discharge line with an expansion loop and nozzle loads, a compressor suction line with a spring hanger, a steam line on a pipe rack with guides and a line stop, and a tank outlet that settles: six worked examples in all, each solving with its reactions balancing its loads.

## 0.1.1 — 2026-10-28

The solver and the 3D view: Datum Pipe now analyzes the piping.

**3D view.** The piping drawn at its true diameter as you type: bends as arcs, insulation as a translucent sleeve, valves, flanges, reducers, tees, anchors, rests, guides, line stops, restraints, springs, constant-force supports and nozzles, with node numbers. Orbit, pan and zoom; Iso, Plan, Front and Side views (keys 1 to 4) and Fit (F). Click a pipe or a node to select it; the table and the Inspector follow.

**Analyze (F5).** Datum Pipe's own 3D pipe element solver: every element's stiffness from its exact flexibility (shear deformation included), bends with their flexibility factor, supports, imposed displacements, and every load case (sustained, operating, expansion range, hydrotest) solved together. An equilibrium check after every solve: the reactions are added up against the loads in every case.

**Results.** The model coloured by stress ratio with a legend, and tables per load case: a summary, code stresses with PASS or FAIL, displacements, forces and moments in the pipe's axes, support loads, nozzle loads, the minimum wall and the equilibrium check. Click a row to jump to the node.

**Verification.** Help › Verification solves 21 benchmark checks in front of you: a cantilever, a span under its own weight, heated L-bends, Z-bends and an expansion loop against the flexibility method by hand, a bend's end rotation, the minimum wall equation and the equilibrium of every example. Each matches its reference to within a millionth.

**Examples.** A simple L-bend and a Z-bend, ready to check by hand.

## 0.1.0 — 2026-10-28

The first release of Datum Pipe: the window, the element table and the model file.

**The window.** The Datum family window in the moss accent: the ribbon (Home, Model, Loads, Supports, Analyze, Results, Isometric, View, Help), Tell Me (Ctrl+K), the File page with the updates card, Options, light, dark and automatic themes, and a ? card beside every tool.

**Type the piping.** The element table: each row is a piece of pipe from a node to the next, with its lengths along X, Y (up) and Z. Type a length in the last row and an element appears, numbered on in steps of 10, with the pipe, material and line conditions of the row above. Bends, tees, reducers, valves and flanges go in the Component column.

**Pipe data.** Nominal pipe sizes ½" to 24" in every schedule; carbon steel, low-alloy steel, stainless steel 304 and 316 and duplex 2205 with modulus and expansion against temperature and their sources; typical allowable stresses, marked as your inputs to check against your code edition. Your own sizes and materials too.

**Supports, conditions and load cases.** Anchors, rests, guides, line stops, restraints in any direction, springs, constant-force supports and nozzle points; up to three operating temperatures and pressures and the hydrotest pressure; the load cases (sustained, operating, expansion range, hydrotest) built for you and editable.

**Files.** Models save and open as `.dpipe` files. A damaged file opens as far as possible and says what it left out; a recovery copy is kept while you work.

**Report.** File › Print shows the stress report's input pages (title block, pipes, materials, conditions, elements, supports, load cases) and prints them or saves them as PDF.

Next: the solver and the 3D view (0.1.1).
