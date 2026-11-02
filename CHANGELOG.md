# Changelog

## 0.5.5 — 2026-11-02

European vessel rules.

**A choice of rules for each vessel.** The Vessels page has Rules for shells and heads: the default thin-shell rules (as before) or the European unfired pressure vessel standard (EN 13445-3). With the European rules, cylinders are calculated using its equation e = P Di / (2 f z − P), spheres and hemispherical heads e = P Di / (4 f z − P), and cones e = P Dk / ((2 f z − P) cos α), with the MAWP from the same equations; the hydrostatic test pressure is the larger of 1.25 Pd × f20/fd and 1.43 Pd, and the test's membrane stress is limited to the yield strength / 1.05. Torispherical and ellipsoidal heads keep the default rules for now, and the results say so. The design stress f is your input from the standard's rules for the material.

**Verification.** A new benchmark: the vertical separator V-101 by the European rules, its shell thickness, MAWP and test pressure by hand (108 checks), and a unit test of each equation.

## 0.5.4 — 2026-11-02

Buried pipe.

**Pipe in the ground.** Loads › Soils keeps the soils of the model: the depth to the pipe's centre line, unit weight, friction angle, cohesion, the pipe surface's friction and adhesion factors, and the movements at which the soil yields. Choose a soil for any element under Buried in in its Inspector. The analysis then puts soil springs at both ends of every buried segment for half its length: along the pipe, across it, and up and down. Each is elastic up to the soil's ultimate resistance and then yields at it, so a long hot line slides through the soil near its bends and ends and is held fully further in. The resistances come from classical soil mechanics: along, the friction of the earth pressure at rest; across, the Rankine passive and active pressures; up, the soil block above the pipe and the shear on its sides; down, the bearing capacity. You can give your own instead. The soil's force at every buried point, and where it has yielded, is in Results › Soil and counts in the equilibrium check; the soil's stiffness is in the natural modes; buried pipe has no wind; and the 3D view draws buried pipe earth-coloured. Take every soil value from the site's soil report.

**Example.** Buried hot water line: 8" at 90 °C, 60 m in sand rising at a bend to a pump-house wall; the soil yields at 21 of its 30 points when hot.

**Verification.** A new benchmark with two checks: a long buried pipe pushed sideways at its end against Hetényi's beam on an elastic foundation, and a long heated pipe's free end against the classical sliding-friction result F² / (2 EA f) (105 checks). The random-model test now also buries every other element on some models, and every case must still balance.

## 0.5.3 — 2026-11-02

European piping standard.

**A third piping code.** Analysis settings › Piping code now also offers the European piping standard (EN 13480-3). With it the stresses are calculated using its equations: the sustained stress σ1 = p Do / 4en + 0.75 i MA / Z (0.75 i at least 1), the sustained plus occasional stress against k fh (k your input, 1.2 filled in), and the displacement stress range σ3 = i MC / Z against fa = U (1.25 fc + 0.25 fh); with the liberal option, its alternative σ1 + σ3 ≤ fh + fa. The minimum wall uses its equation e = p Do / (2 f z + p). The material's allowable table is the design stress f, your input from the standard's rules for the material; check every factor against the edition you work to.

**Verification.** A new benchmark: a heated L-bend at 10 MPa with the European standard, its design thickness and the alternative range allowable at the anchor by hand (102 checks).

## 0.5.2 — 2026-11-02

Power piping code.

**A second piping code.** Analysis settings has a Piping code choice: the process piping code (ASME B31.3, as before and the default) or the power piping code (ASME B31.1). With the power piping code the stresses are calculated using its equations: the sustained stress P Do / 4t + 0.75 i M / Z (0.75 i at least 1, M the resultant moment with torsion; Datum Pipe also adds the axial force over the area, which the equation's older form leaves out), the occasional stress as the sustained stress plus that of the occasional loads, and the displacement stress range i Mc / Z, with one stress intensification factor at each point (the larger of the in- and out-of-plane values from the same flexibility characteristic). The minimum wall, hoop stress, allowable range and liberal allowance use the same forms in both codes. Allowables stay your inputs.

**Occasional factor.** The factor on the hot allowable for occasional loads is now an input next to the code: 1.33 is usual for process piping, 1.15 or 1.2 for power piping by how long the load acts. Choosing a code fills in its usual value; check it against your code edition. The results, help and report show the factor and name the code in use.

**Example.** Boiler steam to a header: 6" Sch 80 at 300 °C and 40 bar with wind, checked with the power piping code (sustained 0.42, range 0.33, wind 0.44 of their allowables).

**Verification.** A new benchmark: a cantilever with an arm, so the root carries both bending and torsion, under pressure, against the power piping sustained equation by hand, and the new example's equilibrium (100 checks).

## 0.5.1 — 2026-11-02

Cold spring.

**A pipe made short on purpose.** Straight pipe has a new input in the Inspector, Cold spring: the length it is made short by and pulled together at installation. It acts as an imposed shortening of that element. The load cases follow it: a new case COLD is the installed state with the cold spring pulled in (weight, pressure and the cold spring), so its support and nozzle loads are the cold loads; the operating, occasional and hydrotest cases include it; the sustained case does not (its stress is weight and pressure only); and the expansion range is taken from COLD, so the stress range is the same as without it, as the piping codes give no credit for cold spring in the range. The hot loads on anchors and nozzles go down, the cold ones up. Load Cases has a Cold spring column for your own cases; the report lists each element's cold spring.

**Verification.** A new benchmark: a straight pipe between anchors made 3 mm short, installed and hot, against Hooke's law (98 checks). A worked test checks on an L-bend that the hot reactions are those without the cold spring plus the installed ones, and that the stress range does not change.

## 0.5.0 — 2026-11-01

The dynamics stage, complete.

**Dynamics in Datum Pipe.** Since 0.4.1 the analysis can find the piping's natural frequencies and mode shapes, with the mass each mode moves, and draw each mode moving in the 3D view; put a response spectrum on the modes, combined by CQC and checked as an occasional load (OCC SPEC); and give the steady response to a force at a frequency with damping (HARM). The steam rack example shows all three.

**Reviewed.** Every dynamics step with the full checks: 96 benchmark checks (three for dynamics, each against classical theory), the unit tests, the UI checks and the real app. A new test checks that a harmonic load at 0 Hz gives the static deflection whatever the number of modes, that its response grows in step with the load, and that at resonance it stays finite and near 1/(2ζ) of the static; the random-model test now also runs modes, a spectrum and a harmonic load on a quarter of its 80 models.

**Fixed.** The printed report now has the dynamics inputs (modes, damping, spectrum, harmonic load), the natural modes table and HARM's support loads, which were missing.

## 0.4.4 — 2026-11-01

Harmonic response.

**A load at a frequency.** Under Dynamics in Analysis settings, Harmonic load takes a force and moment amplitude at one node and a frequency, for a pulsation or a machine's out-of-balance. With the natural modes found, the steady response is the static response plus each mode's dynamic part, u = K⁻¹F + Σ φ (φᵀF) [1/(ω² − Ω² + 2iζωΩ) − 1/ω²], with the damping ratio. The result is the case HARM: the amplitudes of the movements, section forces and support loads. Its values are amplitudes without signs, so equilibrium does not apply to it, and it is not stress-checked (judge its forces and movements against your own vibration limits). If no modes are asked for, a warning says the harmonic load needs them.

**Fixed.** Results › Summary showed an equilibrium figure for OCC SPEC; it now says not applicable, as the Equilibrium table does. A nozzle with allowables is not marked pass or fail in HARM, whose loads are amplitudes.

**Example.** The steam rack example has a 500 N sideways pulsation at the corner 60 at 12 Hz (example values): case HARM moves at most 0.37 mm.

**Verification.** A new benchmark: a heavy mass on a cantilever driven at half its natural frequency, against the one-degree-of-freedom amplitude (96 checks).

## 0.4.3 — 2026-11-01

Response spectrum.

**A spectrum on the modes.** Under Dynamics in Analysis settings, Response spectrum takes the acceleration (g) against frequency from the site's seismic study, a factor for X, Y and Z, and the damping ratio. With the natural modes found, each mode's response in each direction is φ Γ Sa(f) / ω², giving its displacements, section forces and support loads; the modes are combined by the complete quadratic combination (CQC) with the damping, the directions by the square root of the sum of squares. The result is the case OCC SPEC, checked as occasional: the sustained stress plus its stress, against 1.33 Sh; its support loads, nozzle loads and displacements are in the results like any case. A combined spectrum has no signs, so the equilibrium check says it does not apply to it.

**Example.** The steam rack example has a horizontal spectrum (example values) on its ten modes: OCC SPEC at 0.46 of its allowable.

**Verification.** A new benchmark: a cantilever's base shear in a flat spectrum with one mode, against the classical 0.6131 of its mass (95 checks).

## 0.4.2 — 2026-11-01

Modes in the 3D view.

**See each mode move.** Results › Modes has a Show button on every mode: the 3D view draws that mode's shape swinging about the still position, with the still centreline dashed, and the legend names the mode and its frequency with a Stop button that goes back to the load case. Colours by stress ratio are off while a mode is shown (a mode's size has no meaning, so it is drawn at a fixed share of the model).

**Help.** Natural modes has the steps.

## 0.4.1 — 2026-11-01

Natural frequencies and mode shapes: the first step of dynamics.

**Natural modes.** Set how many modes to find under Dynamics in Analysis settings and analyze: Results › Modes lists each mode's frequency and period, the share of the moving mass it carries in X, Y and Z, and the running totals. The modes use the analysis's own stiffness with the mass of the pipe, its contents, insulation, rigid parts and expansion joints lumped at the points; one-way rests and gaps are taken as touching and friction is left out. They are found by subspace iteration (one factorisation, the trial vectors kept mass-orthonormal, the small projected problem by Jacobi rotations): 20 modes of a 3 200-point rack take under two seconds.

**Example.** The steam rack example finds its ten lowest modes: the first at 1.38 Hz, the turbine leg swinging sideways.

**Verification.** A new benchmark: the first frequency of a cantilever and of a fixed-fixed span against beam theory, (λ²/2π) √(EI / m L⁴), within 0.6 % (94 checks); a worked test of the modes' order, mass fractions and orthogonality, and a speed gate.

**Roadmap.** Dynamics in small steps: 0.4.2 modes in the 3D view, 0.4.3 response spectrum, 0.4.4 harmonic response, 0.5.0 the milestone.

## 0.4.0 — 2026-11-01

The vessels stage, complete.

**Pressure vessels in Datum Pipe.** Since 0.3.1 the same app designs pressure vessels next to the piping: shells, heads and cones under internal pressure with the liquid's static head (MAWP, MAP, test pressure, weights); nozzles by area replacement; horizontal vessels on two saddles by Zick's method; vertical vessels on a skirt or legs in wind and seismic, with their anchor bolts and base ring; bolted flanges and covers; lifting lugs and trunnions; the piping's loads checked on the vessel's nozzles; and external pressure with stiffening rings. Two worked examples show it: the Vertical separator V-101 with its inlet line, and the Horizontal drum D-201 on saddles at full vacuum.

**Reviewed.** Every vessel step with the full checks: 92 benchmark checks (eight of them for vessels), the unit tests by hand, the UI checks and the real app. A new hardening test checks 200 random vessels of every shape, with nozzles, saddles or skirts or legs, flanges, lifting and vacuum: each is checked with every result a number, or refused with a reason, and every one round-trips through the file.

**Sources.** Every vessel equation is from the open literature or first principles (membrane theory, area replacement, Zick 1951, the bolted flange method of 1937, plate and beam theory, classical shell buckling), with the allowable stresses, joint efficiencies, gasket factors and yield strengths left as your inputs; Help › Sources lists each one.

**Next.** Dynamics (0.5), then more codes (0.6) and more options (0.7); the rest of the plant (hydraulics, valve sizing, tanks, nozzle stresses by our own shell model) waits in the roadmap.

## 0.3.8 — 2026-11-01

External pressure on vessels.

**Vacuum and external pressure.** A vessel takes an external design pressure (full vacuum is 1.013 bar outside), a design factor on collapse (3 by default) and stiffening rings on its shell (spacing and moment of inertia). From classical shell buckling, with no code charts: a cylinder between stiffeners collapses at 2.42 E (t/D)^2.5 / ((1 − ν²)^0.75 (L/D − 0.45 √(t/D))), never below the long-tube 2 E / (1 − ν²) (t/D)³, L the shell between the heads plus a third of each head's depth, or the ring spacing; heads are spheres of their crown radius (0.9 D for a 2:1 head) at 0.1875 E (t/R)², the classical result with a knockdown for imperfections; each is capped by yield and divided by the design factor. A ring needs a moment of inertia of at least the factor × pe R³ Ls / (3 E).

**Example.** D-201 is checked for full vacuum (the steam-out case) with rings every 3 m: the shell at 0.52 of its allowable, the rings at 0.92. Without the rings the 9.4 m shell would fail.

**Verification.** A new benchmark: D-201's allowable external pressure and its rings by hand (92 checks); a unit test of the collapse formulas.

## 0.3.7 — 2026-11-01

Vessels and piping together.

**The link.** A vessel nozzle can be linked to the piping model's nozzle point at its flange (the Piping column of the nozzles table). After the piping analysis, the nozzle's neck is checked under the piping loads of every case: the pressure's P d / 4t, the axial force and the bending on its corroded section, combined with torsion by the maximum shear stress, against 1.5 S. The loads in the nozzle's own axes are listed for the vessel engineer, who checks the local stresses in the vessel wall.

**Example.** V-101 now has its 8" inlet line: 2 m out from N1, 4 m down and 3 m along to an anchor, at 16 bar and 200 °C, with N1 moving 15 mm up and 2 mm out as the vessel grows. The piping and the vessel are in one model; N1's neck passes in every case.

**Fixed.** The 3D view kept each WebGL context when the Model view was left, so switching views many times could run the graphics out of memory and crash the window; it now releases the context and the scene's geometry each time.

**Verification.** A new benchmark: N1's neck stress in operation by hand from its piping loads, and the separator example's equilibrium (90 checks).

## 0.3.6 — 2026-11-01

Lifting lugs, trunnions and legs.

**Lifting.** Plate lifting lugs (number, thickness, width, hole height, metal round the hole, hole and pin diameters, base weld) lifted along a sling at an angle from vertical, and pipe trunnions (diameter, wall, arm), under the empty weight times an impact factor. The lugs are checked for tension across the hole, shear tear-out, pin bearing, tension and bending at the base, and the base fillet welds; the trunnions for bending and shear at the shell. Allowables are fractions of the yield: 0.6 in tension, 0.4 in shear and on the welds, 0.9 in bearing, 0.66 in bending.

**Legs.** A vertical vessel can stand on legs instead of a skirt: each leg carries the weight over the legs plus 4 M / (n D) from the overturning moment of the wind or seismic, checked against Euler buckling of a cantilever leg (K = 2) with a factor of 2 and 0.6 of the yield, with the base shear's bending in the leg and the two combined.

**Example.** V-101 has two 30 mm lifting lugs on its top head, lifted at 30° with an impact factor of 1.5: the highest ratio 0.31, on the base welds.

**Verification.** A new benchmark: V-101's lugs by hand (88 checks); a unit test of the legs.

## 0.3.5 — 2026-11-01

Bolted flanges and covers on vessels.

**The bolted flange method.** A vessel takes bolted flanges and covers at its design pressure: the gasket's effective width b and diameter G from its seating width, the operating bolt load Wm1 = H + Hp (the pressure's end force and the gasket compression m P) and the seating load Wm2 = π b G y, and the bolt area the larger needs against the bolts given. A ring flange (slip-on, lap joint) is checked by its tangential stress Y Mo / (t² B), from the moments of the end force, the face force and the gasket load about the bolt circle, in operation and at gasket seating; a blind cover by its thickness G √(0.3 P/S + 1.9 W hG/(S G³)), in operation and at seating. The gasket factors m and y, the dimensions and the allowables are your inputs. Hubbed weld-neck flanges are not calculated yet.

**Example.** V-101's manway has a bolted cover: 50 mm against 42.9 mm needed in operation and 46.3 mm at gasket seating.

**Fixed.** Picking a nozzle's pipe size in the Vessels view gave dimensions 40 times too small (the library's sizes are already in metres), and the nozzles' schedule boxes did not show their schedule. Both have UI checks now.

**Verification.** A new benchmark: V-101's cover by hand, its bolt load and its thickness in operation and at seating (86 checks); a unit test of the factor Y and a ring flange's moments.

## 0.3.4 — 2026-11-01

Vertical vessels: wind, seismic and the skirt.

**Loads on a vertical vessel.** Wind (the site's pressure or profile from Analysis settings, times the shape factor and the outside diameter grown by a factor for ladders, platforms and insulation) and static seismic (a coefficient times the mass at each height) give the shear and moment at every height, the vessel taken as a cantilever from its base. The larger of the two moments is used.

**The checks.** Each shell course at its bottom: pressure plus bending less weight in tension against S E, bending and weight in compression against min(S, 0.0625 E t / R), axial buckling with a knockdown. On a skirt: the skirt at its top and base (no pressure), the anchor bolts (4 M / (N × bolt circle) − empty weight / N, on their root area), the foundation bearing under the base ring (weight over its area plus the moment over its section modulus), and the base ring's thickness (its projection as a cantilever strip, l √(3 σ / S)).

**On the page.** For vertical vessels, a Wind, seismic and skirt section takes the shape factor, the diameter factor, the seismic coefficient, the base elevation, the modulus and the skirt, base ring and bolts; the skirt and base ring are drawn; a table gives every section, the bolts, the bearing and the ring.

**Example.** V-101 now stands on a 2.5 m skirt with twelve M30 anchor bolts, in a 1 kPa wind and 0.15 g of seismic: everything passes, the bolts at 18 MPa.

**Verification.** A new benchmark: V-101's wind and seismic base shears, its anchor bolt stress and base ring thickness by hand (83 checks).

## 0.3.3 — 2026-11-01

Horizontal vessels on saddles.

**Saddles.** A horizontal vessel can sit on two saddles: the distance from each tangent line to its saddle, the contact angle (120° to 180°), the saddle width, welded or not, and stiffening rings. The vessel is checked by Zick's method (1951): a beam on two saddles carrying the heavier of its operating weight and its weight full of water, with the shell at the design pressure, hot and corroded: the longitudinal stress at mid-span and at the saddles (with an unstiffened shell's effective section), the tangential shear in the shell (and in the heads when they stiffen the shell, the saddle within R/2 of the tangent line), the circumferential stress at the saddle horn and the compression at the bottom of the shell. The constants come from their closed forms in the contact angle; at 120° they are the familiar 0.107, 0.192, 1.171, 0.880, 0.760 and 0.0528. Allowables: S for the longitudinal stresses, 0.8 S for shear, 1.25 S at the horn, 0.5 Sy at the bottom.

**Small openings.** A nozzle with a finished opening up to 60.3 mm, or up to 88.9 mm in a wall up to 10 mm, needs no reinforcement calculation (its neck is still checked).

**On the page.** The saddles are drawn under the vessel; a Saddles table gives each stress, its allowable and ratio; the saddle data is under the nozzles for horizontal vessels. On a horizontal vessel a nozzle's angle 0 is on top and 180 underneath.

**Example.** A new worked example, Horizontal drum D-201: 2.4 m × 9 m, 10 bar and 150 °C, on two 120° saddles 600 mm from the tangent lines, with an inlet, an outlet and a vent: every check passes, the highest the mid-span longitudinal stress at 0.46.

**Verification.** A new benchmark: D-201's mid-span moment by statics, and its shear and horn stresses by hand with the published 120° constants (79 checks); a unit test of the constants against the published table.

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
