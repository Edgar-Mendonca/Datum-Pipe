// "?" help: a small button beside a tool; a click opens a short card saying what the tool is for
// and how to use it. One place for every explanation, so the whole app teaches itself.
import { For, Show } from "solid-js";
import { openDialog } from "./store";
import { Dialog } from "./ui";

type Topic = { t: string; what: string; how?: string[]; tip?: string };
export const HELP: Record<string, Topic> = {
  // Home and model
  units: { t: "Units", what: "How values are shown and typed: SI (mm, N, N·m, MPa, bar, °C) or US (ft for lengths along the pipe, in for pipe sizes and displacements, lbf, lbf·ft, psi, °F).", tip: "The file always keeps SI base units, so switching never rounds anything." },
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
  disp: { t: "Imposed displacements", what: "Movements forced on the pipe at an anchor for each operating case: a pump nozzle growing with heat, a tank settling. Set them in the Inspector when an anchor is selected." },
  cases: { t: "Load cases", what: "Built for you from the model: SUS (weight and pressure), OPE1–3 (each operating case), EXP1–3 (the range between installed and operating), and HYD when the hydrotest is on.",
    how: ["Load Cases lists them with what each is made of; you can add, edit or remove cases.", "Recommend builds them again from the model and keeps them up to date."] },
  supports: { t: "Supports", what: "Where the pipe is held. Linear in 0.1: each support holds both ways.",
    how: ["Select a node, then pick the support.", "Anchor: all six directions. Rest: up (+Y). Guide: sideways. Line stop: along the pipe.", "Restraint: one direction (X, Y, Z or any). Spring: a stiffness along a direction. Constant: a hanger that carries a set load."],
    tip: "One-way rests, gaps and friction arrive in 0.2." },
  nozzles: { t: "Nozzles", what: "A nozzle point reports the loads on the equipment in the nozzle's own axes (axial, shear, torsion, bending) and compares them with the allowable loads you give.", tip: "Put an anchor at the nozzle too: the anchor carries the load, the nozzle reports it. Pump and compressor standard checks arrive in 0.3." },
  // Analyze and results
  analyze: { t: "Analyze", what: "Builds the stiffness of every element (3D pipe beams with shear deformation; bends with their flexibility factor), solves each load case, and checks the code stresses (F5).",
    how: ["If the piping can move without resistance (not enough supports), Analyze says so instead of giving numbers.", "After an edit the results are kept but marked out of date; press F5 again."] },
  settings: { t: "Analysis settings", what: "The installed temperature, the number of displacement cycles (the allowable range's factor f), the liberal allowable, the bend pressure correction and the hydrotest case." },
  equilibrium: { t: "Equilibrium check", what: "After every solve, the support reactions are added up: they must balance the applied loads (forces and moments). Datum Pipe shows the difference for every case, so a wrong answer cannot hide." },
  code: { t: "Code checks", what: "Calculated using the equations of the process piping code (ASME B31.3): hoop stress and minimum wall, sustained longitudinal stress against the hot allowable, and the displacement stress range against the allowable range.",
    how: ["Each check is at every element end (and the middle of each bend): stress, allowable, ratio and PASS or FAIL.", "Sustained stresses use the wall less corrosion; expansion stresses the nominal wall; minimum wall includes the mill tolerance."],
    tip: "Allowables are your inputs. Results must be checked by a qualified engineer." },
  results: { t: "Results", what: "The 3D view coloured by stress ratio, the deformed shape (with a scale and an animation from installed to operating), and the tables per load case: displacements, forces and moments, support loads, nozzle loads and code stresses.",
    how: ["Pick the load case on the Results tab or above the tables.", "Click a row of the overstressed list to jump to the node."] },
  verification: { t: "Verification", what: "Benchmarks with known answers (beam theory, the thermal L-bend and Z-bend by hand, a bend's flexibility, the minimum wall equation, an expansion loop, equilibrium on every example), solved by this version of Datum Pipe and compared with the reference. They also run as tests before every release." },
  // Isometric, report, view
  iso: { t: "Isometric", what: "An automatic 30° isometric of the model with bends, tees, valves, flanges, supports, node numbers and dimensions, and optional tags (the highest stress ratio of each element, support loads). Large models split across sheets.", how: ["Print it, or save it as SVG or PDF (File › Print, Save as PDF)."] },
  report: { t: "Report", what: "The stress report: title block, input echo (pipes, materials, conditions), load cases, stress summary per case with the worst point, support loads, nozzle loads, the isometric, the equilibrium checks and the solver version." },
  view: { t: "View", what: "Fit the model; standard views (iso, plan, front, side); node labels. Orbit with the left button, pan with the right or middle button, zoom with the wheel." },
  inspector: { t: "Inspector", what: "The values of what is selected, in display units: an element, a node with its supports, nozzle and loads, or the model's settings when nothing is selected." },
  view3d: { t: "3D view", what: "The piping at its true diameter, with insulation as a translucent sleeve, bends, tees, valves, flanges, supports and node labels. Click an element or a node to select it; the table follows." },
};

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
