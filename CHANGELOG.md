# Changelog

## 0.3.2 — 2026-11-01

Vessel nozzles.

**Nozzles on vessels.** Each vessel takes nozzles: the part they are on and where, the neck (a pipe size and schedule, its wall counted at 87.5 %, or the diameter and wall of a rolled-plate neck), the outside weld leg, and a reinforcing pad.

**Reinforcement by area replacement.** The metal the opening removes, d × tr (tr the wall the part needs for its pressure with a joint efficiency of 1), must be made up within the reinforcement limit by the shell's excess thickness, the neck's, the welds and the pad (counted up to twice the opening). The neck must be at least the larger of its own pressure thickness and the smaller of the shell's need and 87.5 % of the standard-wall pipe, with corrosion. Large openings are flagged.

**On the page.** Nozzles are edited under the parts, drawn on the vessel, and checked in a Nozzles table with every area; a nozzle that fails turns red and fails the vessel.

**Example.** V-101 has its 8" inlet, 6" gas outlet, 4" liquid outlet and 20" manway, with pads: all pass.

**Verification.** A new benchmark: V-101's inlet by area replacement and its neck's minimum by hand (76 checks); a unit test of the rules (pad past the limit, the neck's minimum).

## 0.3.1 — 2026-11-01

Pressure vessels: shells and heads.

**A Vessels view.** Datum Pipe now designs pressure vessels in the same model as the piping. The new Vessels view (left bar) holds any number of vertical or horizontal vessels: each is its design data (pressure, temperature, allowable stresses hot and cold, yield strength, corrosion allowance, plate under-tolerance, liquid and its level, extra weight) and a row of parts from the bottom (or the left) to the top: 2:1 ellipsoidal, torispherical and hemispherical heads, cylindrical shell courses and cones.

**What it works out.** For every part: the pressure it carries (the design pressure plus the liquid's static head at its lowest point), the thickness that pressure needs and with the corrosion allowance, its MAWP hot and corroded, and its MAP new and cold. For the vessel: the MAWP at the top and the part that sets it, the MAP, the hydrotest pressure (1.3 × MAWP × S cold / S hot) with each part's membrane stress in the test against 0.9 of the yield, the volume, and the weights empty, operating and full of water. Thin-shell membrane equations: cylinder t = P R / (S E − 0.6 P), sphere t = P R / (2 S E − 0.2 P), 2:1 head t = P D / (2 S E − 0.2 P), torispherical head t = P L M / (2 S E − 0.2 P), cone t = P D / (2 cos α (S E − 0.6 P)). The allowable stresses, joint efficiencies and yield are your inputs.

**Drawing and report.** The vessel is drawn to scale with its tangent lines and liquid level; the report prints each vessel.

**Example.** A new worked example, Vertical separator V-101: 1800 mm inside diameter, 6000 mm between tangent lines, 2:1 heads, 16 bar at 200 °C: the shell needs 12.54 mm (15.54 mm with corrosion, 16 mm nominal) and sets the MAWP of 16.59 bar; test pressure 21.57 bar; 6.6 t empty, 11.2 t operating.

**Fixed.** After an edit, the results were not marked out of date when Analyze automatically was off (one of two internal edit hooks replaced the other).

**Verification.** A new benchmark: V-101's shell and head thicknesses, MAWP, test pressure and volume by hand (73 checks).

## 0.3.0 — 2026-11-01

The equipment and flanges stage, complete.

**Reviewed.** Everything in 0.2.1–0.2.9 (ease of use, speed, equipment nozzles, flange leakage, relief valve thrust, nozzle flexibility, expansion joints, wind by height and both directions, .pcf export) was reviewed with the full checks: every benchmark (68), every hand-worked case, every example solving and balancing, the UI checks, and the real app.

**Sources checked.** Every equation and value the app uses was checked against one rule: only what is published openly, or derived from first principles, with code tables left as your inputs. Help › Sources now shows the flange leakage equation's derivation from the statics of the gasket ring.

**Next.** Pressure vessels, in Datum Pipe: shells and heads, nozzles, saddles, vertical vessels, body flanges, lugs and legs, the piping link and external pressure, step by step to 0.4.0.

## 0.2.9 — 2026-11-01

Export a piping component file.

**File › Export › Export a piping component file** writes the model's geometry as a plain-text .pcf file for design tools: pipes, bends as elbows (their ends and corner), tees (run and branch ends at half the run's diameter from the centre), concentric reducers, valves and flanges with their weight, other rigid parts and expansion joints as miscellaneous components, and supports named by their kind. Coordinates are in millimetres with Z up. Materials, schedules, temperatures and pressures are not part of the file. It is also in the command search (Ctrl+K).

**Checked by reading it back.** Every worked example is exported and imported again with the same length of piping, bends, tees, valves, supports by kind and support positions; random models are exported and read back too.

**Help.** The Import topic is now Import and export.

## 0.2.8 — 2026-11-01

Wind by height, and occasional loads in both directions.

**Wind that changes with height.** Occasional loads › Wind by height takes the design wind pressure at a few elevations (Y) from the wind standard for the site. It is interpolated between them and held beyond them, and each element carries its average along its length. Without a profile, one pressure applies everywhere, as before.

**Both directions.** Tick Wind and seismic in both directions for OPE1−WIN and OCC −WIN (and the same for seismic): the loads reversed. With one-way rests, gaps and friction the reversed case can be worse: a rest that carries more load one way lifts off the other way. The load case editor has the reversed options too, and the report says what was used.

**Example.** The steam rack example checks its wind and seismic in both directions.

**Verification.** A new benchmark: a 10 m vertical cantilever under a wind profile, its base shear and moment by statics, and the reversed case (68 checks). A worked test shows a one-way rest lifting off under upward seismic and carrying more under downward.

## 0.2.7 — 2026-11-01

Expansion joints.

**A new component.** An element can be a bellows expansion joint, with the maker's spring rates: axial, lateral (a parallel offset with the ends kept parallel), angular and torsional (0 is rigid), its effective area, its weight, and whether it is tied. Datum Pipe makes it a short beam with these rates (EA = ka L, EI = kθ L, GJ = kt L, and the shear flexibility that gives the lateral rate); a lateral rate higher than 12 kθ / L² cannot be modelled that way and is flagged.

**Pressure thrust.** An untied joint cannot hold the pressure: the pressure on its effective area pushes the pipe apart at both its ends in every case with pressure (and the hydrotest), so the main anchors carry it. A tied joint's rods carry the thrust and hold its length.

**Where to find it.** Model › Components › Expansion Joint, or Expansion joint in the table's component column; the rates in the Inspector. The 3D view draws the convolutions (and the tie rods), the isometric a zigzag. The code stresses are checked on the pipe at the joint's ends, not on the bellows.

**Example.** A new worked example, Expansion joint between anchors: 8" steam at 200 °C and 10 bar, 24 m between main anchors, an untied joint in the middle: each main anchor carries 60 kN in operation, mostly the 45 kN pressure thrust.

**Verification.** A new benchmark: a joint's lateral and angular rates, and the pressure thrust shared between the pipe and the joint, by hand, and the new example's equilibrium (65 checks). A worked test covers tied and untied joints, the warning and the stress points.

## 0.2.6 — 2026-11-01

Vessel and tank nozzle flexibility.

**The wall gives at the nozzle.** A nozzle point now has the stiffness of the vessel or tank wall at the nozzle: axial (radial to the shell), shear, torsion, and bending in the longitudinal plane (the plane of the nozzle and the vessel's axis) and in the circumferential plane, with the vessel's axis (vertical by default). With any of them above 0, the anchor at the nozzle's node holds along the nozzle's own axes with these stiffnesses; 0 holds that direction rigidly. The equipment's growth given on the anchor still moves the nozzle, and the nozzle loads are reported as before.

**Your inputs.** The stiffnesses come from the vessel's or tank's maker, or your own calculation; Datum Pipe does not calculate them, and local stresses in the vessel wall are left to the vessel engineer, who gets the nozzle loads in the nozzle's axes.

**Example.** The tank outlet example's nozzle has the tank shell's flexibility (example values).

**Verification.** A new benchmark: a cantilever from a flexible nozzle, the extra tip deflection P L²/kθ + P/ks and the nozzle's rotation by hand, the anchor's moment by statics (61 checks). A worked test covers a skewed nozzle, the nozzle's growth through the flexible wall and a nozzle on a head.

**Roadmap.** 0.2.7 expansion joints, 0.2.8 wind that changes with height and seismic in both directions, 0.2.9 .pcf export, then the 0.3.0 review. Later stages gain load combinations, a sustained case with the operating support states, more operating cases, other pipe size standards and jacketed pipe.

## 0.2.5 — 2026-10-31

Relief valve thrust.

**The thrust at an open discharge.** Loads › Relief Valves puts a relief valve at the open end of its outlet pipe, with its relieving flow, temperature, ratio of specific heats k and molar mass. Datum Pipe calculates the reaction from the conservation of momentum, F = ṁ V + (P − Pa) A, with the gas leaving at sound speed: V = √(2kRT/((k+1)M)) and exit pressure P = (ṁ/A)√(2RT/(k(k+1)M)). This is the same result as the published open-discharge reaction equation of API RP 520 Part II. The force is multiplied by a dynamic load factor (2 by default; check it against your code edition). It pushes the pipe back along the pipe at the open end, or against a direction you give. The exit area is the pipe's bore unless you enter one.

**Load cases.** With relief valves, OPE1+RV (operating case 1 with the thrust) and OCC RV (the thrust's own effect) are added; OCC RV is checked as an occasional load, the sustained stress plus its stress against 1.33 Sh. The load case editor has a Relief thrust option.

**Results.** Results › Relief shows each valve's exit area, velocity, exit pressure, whether the exit is choked, and the thrust before and after the load factor; the report has it too. Where the exit is not choked the pressure term is 0 and the sound speed is kept, which overstates the thrust.

**Example.** A new worked example, Safety valve vent stack: 3 kg/s of steam at 200 °C up a 4" stack, a 3.6 kN thrust with the load factor, OCC RV at 0.14.

**Units.** Mass flows (kg/s, lb/h), flow areas (mm², in²) and velocities (m/s, ft/s).

**Verification.** A new benchmark: a relief valve on an L-shaped outlet, its thrust by hand and the anchor's force and moment by statics, and the new example's equilibrium (58 checks). A unit test checks the thrust against the published form in US units.

## 0.2.4 — 2026-10-31

Flange leakage.

**Equivalent pressure.** Loads › Flange Leakage gives a flange pair or a valve's flanged ends a gasket reaction diameter G and a pressure rating at temperature. Each operating and sustained case then checks the joint by the equivalent pressure method: the bending moment M and the axial force F at the joint act on the gasket like an extra pressure P_eq = 16M/(πG³) + 4F/(πG²), and the case's pressure plus P_eq must stay within the rating. Axial compression is counted like tension, on the safe side. The hydrotest is not checked here: it is judged against a test rating.

**Your inputs.** G and the rating come from your flange data; no standard's tables are reproduced.

**Results.** Results › Flanges shows the force, moment, pressure, P_eq, rating and ratio at the joint's most loaded end; the report has it for the operating case.

**Examples.** The pump discharge's gate valve is checked with example values (a 180 mm gasket diameter, a 40 bar rating): 0.84 in operation.

**Verification.** A new benchmark: a flange pair on a cantilever, its moment and axial force by statics and its equivalent pressure by hand (54 checks).

## 0.2.3 — 2026-10-31

Rotating equipment nozzle checks.

**Equipment.** Supports › Equipment Checks groups the nozzle points of a steam turbine, a compressor or a pump.

**Turbines and compressors** are checked against the criteria their standards publish as equations, in the standard's units (lb, ft·lb, inches):
- each nozzle: 3F + M ≤ 500·De;
- all nozzles, resolved at the reference (exhaust or discharge) nozzle: 2Fc + Mc ≤ 250·Dc, with Dc from the nozzles' combined area, and a limit on each component of the combined loads;
- a compressor with a multiplier on every allowable (typically 1.85).

**Pumps** are checked against the allowable loads of the pump standard's table, which you enter for each nozzle (components and resultants, with a multiplier of 1 or 2).

**Your inputs.** Every coefficient, the rules for large nozzles, the component limits, the multipliers and the pump table values are your inputs, filled with typical values: check them against the edition of the standard that applies. No standard's tables are reproduced.

**Results.** Results › Equipment lists each check with its value, allowable and ratio for the chosen case; the report has it for the operating case, and a case with a failing check no longer shows PASS.

**Examples.** The compressor (K-201, factor 1.85) passes; the steam line's turbine (T-301) fails the turbine criteria, as a stiff line near a turbine does: a starting point for adding flexibility.

**Verification.** A new benchmark: a steam turbine with two nozzle lines, its nozzle loads by statics and its criteria by hand (52 checks).

## 0.2.2 — 2026-10-31

Speed on large models. Measured on a rack line of 1441 elements and 1602 supports (one-way rests with friction, guides, line stops):

| | Before | Now |
| --- | --- | --- |
| Analysis (every case settled) | 16.6 s | 0.18 s |
| Opening the model in the window | 7.2 s | 1.8 s |
| One edit, drawn again | 2.4 s | 0.6–0.75 s |

The window figures are from a test browser drawing in software; a graphics card is faster.

**Analysis.** With more than 64 supports with friction, a sliding force is found from an estimate of the support's flexibility (from the stiffness at its node) instead of an extra solve per support; it settles to the same answer, and the exact method takes over if it has not settled in 40 solves. A test checks Coulomb's law at every support of a large rack.

**Element table.** Only the rows in view are drawn, so a table of thousands of elements opens and scrolls like a short one; a row selected elsewhere is scrolled into view.

**3D view.** Parts that share a colour are drawn as one object (a click still finds the element, node or support), straight pipe is drawn as a cylinder, and large models use fewer facets.

**Speed checks.** The engine's tests time the large rack (under 1 s), and `npm run perf:ui` times the window on it.

## 0.2.1 — 2026-10-31

Ease of use: the first step of the third stage.

**Where a model is free.** When the piping can move without resistance, Analyze names the node and the direction nothing holds ("at node 30, turning about X") and shows that node.

**Analyze automatically.** Every edit is analysed again a moment later, and a model is analysed when it opens, so the stress colours, tables and support loads follow what you type. It stays quiet unless something is wrong. File › Options turns it off.

**Values with units.** Any number field, and the element table, takes a value with its unit: 20 ft, 3' 6", 1 ft 6 in, 150 mm, 3.5 bar, 300 °F. A unit of the wrong kind is refused with the reason.

**Right-click in the 3D view.** A node offers its supports, a nozzle point and a load; an element its component, Insert Node, Add an Element After, the expansion loop and Delete; empty space Fit and Analyze. A click within a few pixels of a node picks the node.

**Roadmap.** The third stage (equipment and flanges) is split into 0.2.1–0.2.6, with 0.3.0 as its milestone (File › Roadmap).

## 0.2.0 — 2026-10-30

The milestone that closes the second stage: nonlinear supports, occasional loads, spring hangers and import (0.1.5–0.1.11), reviewed. No new features.

**Fixes from an independent review of the engine.**
- Two one-way or gapped supports touching on the same direction at one node now share its load equally, with a warning; each reported the whole load, so the equilibrium check failed.
- A bend whose radius is half its pipe's diameter or less is refused (its bend factor turned negative and the wall check passed); the wall check of such a bend fails.
- A spring hanger whose attachment moves in operating case 1 is sized on the pipe's movement relative to the attachment, so it carries its hot load in operation.
- Importing a piping component file joins an olet's branch to its run pipe (the run was not split there, leaving the branch unconnected).

Each with a regression test. The verification record (48 checks), the screenshots, the README and the site describe everything up to this release.

## 0.1.11 — 2026-10-30

Import of piping component files.

**Import.** File › Export › Import a piping component file opens a plain-text .pcf file from a design tool as a new model:
- pipes; elbows and bends at their corner, with the radius from their tangent length; tees and olets; concentric and eccentric reducers (as concentric); valves, flanges and other in-line items as rigid parts with their weight;
- supports as one-way rests, or anchors, guides and line stops when their name says so;
- millimetres or inches; coordinates turned from Z up to Datum Pipe's Y up;
- pipes at the library's nominal sizes with Std wall, and the default material and line conditions: set them for the line.

Anything it cannot read is listed when the model opens. A sample file is in `docs/examples/6-P-101.pcf`.

**Roadmap.** The public piping benchmark problems published by nuclear regulators are dynamic problems (natural frequencies and response spectra), so they move to 0.4 with dynamics, where they can be checked. The next release is 0.2.0, which closes this stage.

## 0.1.10 — 2026-10-30

Spring hanger selection.

**Spring hangers.** Supports › Spring Hanger puts a hanger on a node, and Analyze sizes it:
- the hot load H: the operating weight it carries with the pipe held level there;
- the travel Δ: its movement in operating case 1 with the hanger carrying H;
- the spring: the first size in the spring table whose working range takes both H and the cold load C = H + k Δ, with the load variation |k Δ| / H within the limit (25 % unless you change it).

Every load case is then solved with the chosen springs, each set to its cold load. A hanger no spring fits becomes a constant-force support carrying H, with a warning.

**Spring table.** Supports › Spring Table: the sizes hangers are chosen from, in order, with the load variation allowed. It starts with a generic table (three series with 25, 50 and 100 mm of working travel, twenty sizes each), not any maker's catalogue: replace it with your supplier's sizes.

**Results.** Results › Hangers lists each hanger's size, rate, hot load, travel, cold load, load variation and working range; the report has the same table.

**Examples.** The tank outlet's first support is a spring hanger sized by Analyze.

**Verification.** A new benchmark: a spring hanger on a heated riser and run, its hot load, travel, spring, cold load and load in operation by hand (48 checks).

## 0.1.9 — 2026-10-30

Wind and static seismic loads, and the occasional stress check.

**Wind.** A design wind pressure, a shape factor Cd and the direction the wind blows, in the Inspector's Occasional loads (or Loads › Wind). Each element carries q × Cd × D per metre, with D over its insulation and cladding, from the part of the wind that blows across it: wind along a pipe puts no load on it.

**Static seismic.** Factors of g along X, Y and Z (Loads › Static Seismic); each element carries its operating weight per metre times them.

**Load cases.** With wind, OPE1+WIN (operating case 1 with the wind) and OCC WIN (the wind's own effect, OPE1+WIN − OPE1) are added; the same for seismic. The Load Cases dialog has an Occasional column for your own cases.

**Occasional check.** An occasional case's stress at each point is the sustained stress plus the stress of the occasional loads (the same expression, without pressure), against 1.33 Sh.

**Your inputs.** The wind pressure and the seismic factors come from the standards for the site; Datum Pipe reproduces none of their tables. Help › Sources lists the new equations.

**Examples.** The steam line on the rack has 0.8 kPa of wind from the south and 0.15 g of static seismic sideways.

**Verification.** Three new benchmarks: wind on a cantilever (tip deflection and root moment), static seismic on a cantilever (root force and moment), and the occasional stress at its root (43 checks).

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
