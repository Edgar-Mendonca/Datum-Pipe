// "?" help: a small button beside a tool; a click opens a short card saying what the tool is for
// and how to use it. One place for every explanation, so the whole app teaches itself.
import { For, Show } from "solid-js";
import { openDialog } from "./store";
import { Dialog } from "./ui";

type Topic = { t: string; what: string; how?: string[]; tip?: string };
export const HELP: Record<string, Topic> = {
  // Home and model
  units: { t: "Units", what: "How values are shown and typed: SI (mm, N, N·m, MPa, bar, °C) or US (ft for lengths along the pipe, in for pipe sizes and displacements, lbf, lbf·ft, psi, °F).", tip: "The file always keeps SI base units, so switching never rounds anything." },
  import: { t: "Import", what: "File › Export › Import a piping component file: a plain-text .pcf file from a design tool becomes a new model. Pipes, elbows and bends (with their radius), tees and olets, reducers, valves and flanges (rigid, with their weight) and supports are read; coordinates turn from Z up to Datum Pipe's Y up.",
    how: ["Pipes are the library's nominal sizes at Std wall; the material and line conditions are the defaults: set them for the line.", "Supports become one-way rests, or anchors, guides and line stops when their name says so: check every one.", "Anything the import could not read is listed when it opens."] },
  examples: { t: "Examples", what: "Worked models: an L-bend and a Z-bend you can check by hand, a pump discharge line with an expansion loop, a compressor suction line with nozzle loads, a steam line on a pipe rack, and a tank outlet line with settlement." },
  table: { t: "Element table", what: "The piping as a table: each row is one element from a node to the next, with its lengths along X, Y and Z (Y is up).",
    how: ["Type DX, DY or DZ in the last row to add an element; it starts at the node above and is numbered on in steps of 10.", "Each row has its pipe, material, line conditions and component; new rows copy them from the row above.", "A bend or tee is at the row's To node. Click a row to select it in the 3D view; Ctrl+Enter adds a row after it."],
    tip: "To start a branch, type the From node of an existing node (a tee goes at that node)." },
  elements: { t: "Elements", what: "Add, insert and delete elements: Insert Node splits the selected element in two, Renumber numbers every node again in steps of 10." },
  components: { t: "Components", what: "What sits on an element: straight pipe, a bend (long radius 1.5D, short 1D, or your own radius), a tee (welding, fabricated, reinforced), a concentric reducer, a valve or a flanged joint (rigid, with a weight).",
    how: ["A bend at a node turns the element into the next one; it needs room for its radius on both sides.", "A tee at a node is where three elements meet; its stress intensification applies to all three.", "Valves and flanges are much stiffer than the pipe; give their weight without the contents."] },
  pipes: { t: "Pipes", what: "Pipe sections: nominal size and schedule from the library (½\" to 24\"), or your own diameter and wall, with corrosion allowance, mill tolerance, insulation, cladding and contents." },
  materials: { t: "Materials", what: "Modulus and thermal expansion against temperature, density, Poisson's ratio, and the allowable stresses, each with its source.", tip: "Allowable stresses are your inputs: the typical values filled in (a third of the tensile strength or two thirds of the yield, whichever is less) must be checked against your code edition." },
  conds: { t: "Line conditions", what: "Up to three operating temperatures (T1–T3) and pressures (P1–P3), and the hydrotest pressure, shared by every element that uses them.", tip: "A temperature equal to the installed temperature and a pressure of 0 leave that case unused." },
  edit: { t: "Edit", what: "Copy a section, mirror it, or turn a straight run into an expansion loop." },
  // Loads and supports
  loads: { t: "Loads", what: "Weight is always included: the pipe, its contents, insulation and cladding, valves and flanges. Add extra forces and moments at nodes, or an extra uniform load along an element." },
  hangers: { t: "Spring hangers", what: "A spring hanger holds the pipe up with a spring, so it follows the pipe as it grows. Put one on a node (Supports › Spring Hanger) and Analyze sizes it from the spring table.",
    how: ["Hot load H: the weight the hanger carries with the pipe held level at it.", "Travel Δ: its movement in operating case 1 with the hanger carrying H.", "The spring: the first in the table whose working range takes H and the cold load C = H + k Δ, with the load variation |k Δ| / H within the limit (25 % unless you change it).", "Every case is then solved with that spring, set to C when installed. With no spring that fits, it is a constant-force support carrying H, with a warning."],
    tip: "The spring table filled in is generic, not any maker's catalogue: use your supplier's sizes (Supports › Spring Table)." },
  occasional: { t: "Occasional loads", what: "Wind and static seismic, checked as occasional loads. Wind on each element is q × Cd × D per metre (the design pressure, the shape factor, and the outside diameter over insulation), from the part of the wind that blows across it. Static seismic is the operating weight per metre times the factors you give along X, Y and Z (in g).",
    how: ["Select nothing and open Occasional loads in the Inspector (or Loads › Wind, Static Seismic).", "With wind, the cases OPE1+WIN (operating case 1 with the wind) and OCC WIN (the wind's own effect, OPE1+WIN − OPE1) are added; the same for seismic.", "The occasional check: the sustained stress plus the stress of the occasional loads at each point, against 1.33 Sh."],
    tip: "The pressure and factors are your inputs from the wind and seismic standards for the site; Datum Pipe reproduces none of their tables. Wind is the same at every height." },
  disp: { t: "Imposed displacements", what: "Movements forced on the pipe at an anchor for each operating case: a pump nozzle growing with heat, a tank settling. Set them in the Inspector when an anchor is selected." },
  cases: { t: "Load cases", what: "Built for you from the model: SUS (weight and pressure), OPE1–3 (each operating case), EXP1–3 (the range between installed and operating), HYD when the hydrotest is on, and OPE1+WIN, OCC WIN, OPE1+SEIS and OCC SEIS when there is wind or seismic.",
    how: ["Load Cases lists them with what each is made of; you can add, edit or remove cases.", "Recommend builds them again from the model and keeps them up to date."] },
  supports: { t: "Supports", what: "Where the pipe is held. New rests are one-way: they push the pipe up but let it lift off. Rests, guides, line stops and restraints can have a gap before they take load, and friction.",
    how: ["Select a node, then pick the support.", "Anchor: all six directions. Rest: up (+Y). Guide: sideways. Line stop: along the pipe.", "Restraint: one direction (X, Y, Z or any). Spring: a stiffness along a direction. Constant: a hanger that carries a set load.", "In the Inspector: One-way for a rest, and the gap (below the pipe for a rest, on each side for the others), and the friction coefficient μ."],
    tip: "With one-way rests, gaps or friction, each load case is solved again until every such support is either touching or open, and sticking or sliding (pushing back μ × its load against the movement); Support Loads shows which." },
  nozzles: { t: "Nozzles", what: "A nozzle point reports the loads on the equipment in the nozzle's own axes (axial, shear, torsion, bending) and compares them with the allowable loads you give.", tip: "Put an anchor at the nozzle too: the anchor carries the load, the nozzle reports it. Pump and compressor standard checks arrive in 0.3." },
  // Analyze and results
  analyze: { t: "Analyze", what: "Builds the stiffness of every element (3D pipe beams with shear deformation; bends with their flexibility factor), solves each load case, and checks the code stresses (F5).",
    how: ["If the piping can move without resistance (not enough supports), Analyze says so instead of giving numbers.", "After an edit the results are kept but marked out of date; press F5 again."] },
  settings: { t: "Analysis settings", what: "The installed temperature, the number of displacement cycles (the allowable range's factor f), the liberal allowable, the bend pressure correction, pressure elongation of straight pipe in the operating cases (off unless you choose it), the effective section modulus at tee branches (on; turn it off if your code edition uses the branch pipe's own), and the hydrotest case." },
  equilibrium: { t: "Equilibrium check", tip: "It proves the loads were all applied and the equations were solved; it does not prove the model or the stiffnesses are right. That is what the benchmarks in Help › Verification and your own review are for.", what: "After every solve, the support reactions are added up: they must balance the applied loads (forces and moments). Datum Pipe shows the difference for every case." },
  code: { t: "Code checks", what: "Calculated using the equations of the process piping code (ASME B31.3): hoop stress and minimum wall, sustained longitudinal stress against the hot allowable, and the displacement stress range against the allowable range.",
    how: ["Each check is at every element end (and the middle of each bend): stress, allowable, ratio and PASS or FAIL.", "Sustained stresses use the wall less corrosion; expansion stresses the nominal wall; minimum wall includes the mill tolerance."],
    tip: "Allowables are your inputs. Datum Pipe is not affiliated with or endorsed by ASME and contains none of the code's text, tables or figures; where each equation is published is in Help › Sources. Results must be checked by a qualified engineer against the code edition of the project." },
  results: { t: "Results", what: "The 3D view coloured by stress ratio, the deformed shape (with a scale and an animation from installed to operating), and the tables per load case: displacements, forces and moments, support loads, nozzle loads and code stresses.",
    how: ["Pick the load case on the Results tab or above the tables.", "Click a row of the overstressed list to jump to the node."] },
  verification: { t: "Verification", what: "Benchmarks with known answers (beam theory, the thermal L-bend and Z-bend by hand, a bend's flexibility, the minimum wall equation on straight pipe and at a bend's intrados, an expansion loop, pressure elongation between anchors, a tee branch's effective section modulus, a one-way rest lifting off and a gap closing, friction at a rest sliding and sticking, wind and static seismic on a cantilever and the occasional stress, a spring hanger sized by hand, equilibrium on every example), solved by this version of Datum Pipe and compared with the reference. They also run as tests before every release." },
  // Isometric, report, view
  iso: { t: "Isometric", what: "An automatic 30° isometric of the model with bends, tees, valves, flanges, supports, node numbers and dimensions, and optional tags (the highest stress ratio of each element, support loads). Large models split across sheets.", how: ["Print it, or save it as SVG or PDF (File › Print, Save as PDF)."] },
  report: { t: "Report", what: "The stress report: title block, input echo (pipes, materials, conditions), load cases, stress summary per case with the worst point, support loads, nozzle loads, the isometric, the equilibrium checks and the solver version." },
  view: { t: "View", what: "Fit the model; standard views (iso, plan, front, side); node labels. Orbit with the left button, pan with the right or middle button, zoom with the wheel." },
  inspector: { t: "Inspector", what: "The values of what is selected, in display units: an element, a node with its supports, nozzle and loads, or the model's settings when nothing is selected." },
  view3d: { t: "3D view", what: "The piping at its true diameter, with insulation as a translucent sleeve, bends, tees, valves, flanges, supports and node labels. Click an element or a node to select it; the table follows." },
};

/** Help › Sources: where each equation Datum Pipe uses is published. Equations and physical
 *  facts are not anyone's property; the code's own text, tables and figures are, and none of them
 *  is in the app. Keep this list true when an equation is added. */
export const SOURCES: [string, string, string][] = [
  ["Pressure design thickness and hoop stress", "t = P D / (2 (S E W + P Y))", "Barlow's thin-wall hoop stress formula σ = P D / 2t, with the coefficient Y bringing it towards Lamé's thick-cylinder solution (G. Lamé, Leçons sur la théorie mathématique de l'élasticité des corps solides, 1852). Y is your input on the material."],
  ["Limit of the thin-wall equation", "t < D/6", "The range where the thin-wall formula stays close to Lamé's solution; beyond it the check fails and asks for a thick-wall design."],
  ["Bend wall at the intrados", "I = (4R/D − 1) / (4R/D − 2)", "Derived from the membrane equilibrium of a torus under pressure, σ = (P r / 2t)(2R + r sin φ)/(R + r sin φ), at r = D/2 on the inside of the bend, divided by the straight pipe's P r / t. The membrane theory of shells of revolution is in shell-theory textbooks, e.g. S. Timoshenko and S. Woinowsky-Krieger, Theory of Plates and Shells, McGraw-Hill, 2nd ed., 1959."],
  ["Bend flexibility factor", "k = 1.65 / h, h = T R / r²", "Th. von Kármán, Zeitschrift des Vereines Deutscher Ingenieure 55 (1911) 1889–1895, on the flattening of curved tubes; R. A. Clark and E. Reissner, Bending of curved tubes, Advances in Applied Mechanics 2 (1951) 93."],
  ["Bend and tee stress intensification factors", "i = 0.9 / h^⅔, 0.75 / h^⅔; tees h = 3.1 T / r, T / r", "A. R. C. Markl, Fatigue tests of piping components, Transactions of the ASME 74 (1952) 287–303."],
  ["Pressure correction of bend factors", "k / (1 + 6 (P/E)(r/T)^(7/3)(R/r)^(1/3)), i / (1 + 3.25 (P/E)(r/T)^(5/2)(R/r)^(2/3))", "E. C. Rodabaugh and H. H. George, Effect of internal pressure on flexibility and stress-intensification factors of curved pipe or welding elbows, Transactions of the ASME, 1957."],
  ["Reducer stress intensification", "i = 0.5 + 0.01 α (D₂/t₂)^½, 1 to 2", "The equation as the process piping code states it; an equation is not protected, and only the equation is used."],
  ["Effective branch section modulus", "Ze = π r₂² Ts, Ts = min(Th, ii Tb)", "The section modulus of a thin tube, π r² t (beam theory), with the effective thickness the code's earlier editions use. Off in Analysis Settings if your edition uses the branch pipe's own."],
  ["Sustained and displacement stresses", "SL = √((|Fa/A| + Sb + Sp)² + (2St)²), SE = √((|Fa/A| + Sb)² + (2St)²)", "Beam theory: axial, bending and torsion stresses on the pipe section, combined by the maximum shear stress; Sp = P di² / (Do² − di²) is the axial stress of a closed cylinder (statics)."],
  ["Wind and static seismic loads", "q Cd D per metre; m a", "Drag on a cylinder across a flow (pressure × shape factor × projected width) and Newton's second law. The pressure, Cd and seismic factors are your inputs from the standards for the site."],
  ["Occasional stress", "SL + S_occ ≤ 1.33 Sh", "The process piping code's rule for occasional loads (para. 302.3.6), used as an equation only: the sustained stress plus the stress of the occasional loads."],
  ["Spring hangers", "C = H + k Δ, |k Δ| / H", "Hooke's law for the spring; the hot load and travel from the analysis. The spring sizes are your inputs from your supplier."],
  ["Allowable stress range", "SA = f (1.25 Sc + 0.25 Sh), f = 6 N^−0.2", "Markl's fatigue curve i S = constant × N^−0.2 (Markl, 1952, above); flexibility analysis on this basis is set out in M. W. Kellogg Company, Design of Piping Systems, Wiley, 2nd ed., 1956. The factors 1.25 and 0.25 and the limits of f are the code's equation, used as an equation only."],
  ["Pressure elongation", "ε = (σL − ν σh) / E", "Hooke's law with the axial and hoop stresses of a closed cylinder."],
  ["Pipe sizes", "OD and wall of each schedule", "The published nominal dimensions of wrought and stainless steel pipe (ASME B36.10M and B36.19M), as given in pipe makers' catalogues: dimensions are facts."],
  ["Materials and allowable stresses", "Typical values", "Typical property trends for each material family (Datum Atlas); allowables worked out as the lesser of a third of the tensile and two thirds of the yield strength. They are your inputs: check them against your code edition."],
];

export const sourcesDialog = () => openDialog(() => (
  <Dialog title="Sources">
    <div class="helpcard">
      <p>Datum Pipe calculates using the published equations of the process piping code (ASME B31.3). It contains none of the code's text, tables or figures, and is not affiliated with or endorsed by ASME. The equations come from the open engineering literature listed here. ASME and B31.3 are trademarks of the American Society of Mechanical Engineers.</p>
      <table class="tbl"><thead><tr><th>What</th><th>Equation</th><th>Where it is published</th></tr></thead><tbody>
        <For each={SOURCES}>{([w, e, s]) => <tr><td>{w}</td><td class="mono">{e}</td><td>{s}</td></tr>}</For>
      </tbody></table>
      <p class="muted">Results must be checked by a qualified engineer against the code edition that applies to the project before they are used for design, fabrication or approval.</p>
    </div>
  </Dialog>
));

/** The small "?" beside a tool. */
export const Help = (p: { k: string }) => <button class="help" title={`What is ${HELP[p.k]?.t ?? "this"}?`} onClick={e => { e.stopPropagation(); e.preventDefault(); helpCard(p.k); }}>?</button>;

export const helpCard = (k: string) => openDialog(() => {
  const h = HELP[k];
  return (
    <Dialog title={h?.t ?? "Help"}>
      <div class="helpcard"><p>{h?.what}</p>
        <Show when={h?.how?.length}><h4>How to use it</h4><ul><For each={h!.how}>{x => <li>{x}</li>}</For></ul></Show>
        <Show when={h?.tip}><p class="muted">{h!.tip}</p></Show></div>
    </Dialog>
  );
});
