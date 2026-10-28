# Changelog

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
