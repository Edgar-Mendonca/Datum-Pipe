// Dialogs: saving changes, closing, recovery, and the help pages (shortcuts, getting started,
// glossary). The property managers (pipes, materials, line conditions, load cases) are in props.tsx.
import { For, createSignal } from "solid-js";
import { Out } from "./api";
import { openDialog, closeDialog, call, out, setOut } from "./store";
import { C } from "./cmds";
import { Dialog } from "./ui";

export const keepChanges = (then: () => unknown) => {
  if (!out()?.dirty) { then(); return; }
  openDialog(() => (
    <Dialog title="Save changes?" okText="Save" cancel="Cancel" ok={async () => { await C.save(); if (out()?.dirty) return false; setTimeout(then); }}
      extra={<button class="btn" onClick={() => { closeDialog(); setTimeout(then); }}>Don't save</button>}>
      <p>“{out()?.m.name}” has changes that are not saved.</p>
    </Dialog>
  ));
};

export const confirmClose = (quit: () => void) => openDialog(() => (
  <Dialog title="Save changes?" okText="Save" cancel="Cancel" ok={async () => { await C.save(); if (!out()?.dirty) quit(); }}
    extra={<button class="btn" onClick={() => { closeDialog(); quit(); }}>Don't save</button>}>
    <p>“{out()?.m.name}” has changes that are not saved.</p>
  </Dialog>
));

export const shortcuts = () => openDialog(() => (
  <Dialog title="Keyboard shortcuts">
    <table class="tbl keys"><tbody><For each={[
      ["Save / Save As", "Ctrl+S / Ctrl+Shift+S"], ["Open", "Ctrl+O"], ["New", "Ctrl+N"], ["Print", "Ctrl+P"], ["Tell me what you want to do", "Ctrl+K"],
      ["Undo / Redo", "Ctrl+Z / Ctrl+Y"], ["Delete the selected element or support", "Del"], ["Add an element after the selected one", "Ctrl+Enter"],
      ["Next / previous row in the table", "Enter / Shift+Enter, ↓ / ↑"], ["Next cell", "Tab"],
      ["Analyze", "F5"], ["Fit the model in the view", "F"], ["Standard views: iso, plan, front, side", "1 2 3 4"], ["Orbit / pan / zoom", "Left drag / right or middle drag / wheel"], ["This list", "F1"],
    ] as [string, string][]}>{([a, b]) => <tr><td>{a}</td><td><kbd>{b}</kbd></td></tr>}</For></tbody></table>
  </Dialog>
));

export const gettingStarted = () => openDialog(() => (
  <Dialog title="Getting started" wide>
    <div class="helpcard">
      <p>Datum Pipe works out how a piping system moves and what it pushes on when it heats up, carries its weight and holds its pressure: the displacements, the support loads, the loads on equipment nozzles and the code stresses. You describe the piping as a table of elements, as you would on a stress sketch.</p>
      <h4>Route the piping</h4>
      <ol>
        <li>In the <b>element table</b> under the 3D view, each row is one piece of pipe: From node, To node and its length along X, Y and Z (DX, DY, DZ). Y is up. Type DX, DY or DZ in the empty last row and a new element appears, numbered on from the last node (10, 20, 30 …).</li>
        <li>Pick the <b>pipe</b> (size and schedule), <b>material</b> and <b>line conditions</b> (temperatures and pressures) in the row; new rows copy them from the row above.</li>
        <li>A <b>bend</b> or a <b>tee</b> goes at the To node of its row: choose it in the Component column. Valves and flanges are rigid elements with a weight.</li>
      </ol>
      <h4>Support and load it</h4>
      <ol>
        <li>Click a node (in the 3D view or the table), then a support on the <b>Supports</b> tab: anchor, rest, guide, line stop, restraint, spring or constant-force support. A nozzle point marks equipment loads to check.</li>
        <li>On the <b>Loads</b> tab, set the line conditions: up to three operating temperatures and pressures and the hydrotest pressure. Imposed displacements (equipment growth, settlement) go on anchors.</li>
        <li>The load cases (SUS, OPE, EXP, HYD) are built for you; Loads › Load Cases shows and edits them.</li>
      </ol>
      <h4>Check and report</h4>
      <ol>
        <li><b>Analyze</b> (F5) solves every load case and checks the stresses with the equations of the process piping code. The 3D view is coloured by the stress ratio; the Results view has the tables: displacements, forces, support loads, nozzle loads and stresses.</li>
        <li>The <b>Isometric</b> view draws the line as a 30° isometric with dimensions and optional result tags; the <b>Report</b> prints the stress report with the isometric and the equilibrium checks.</li>
      </ol>
      <h4>Good to know</h4>
      <ul>
        <li>Values are shown in SI (mm, N, MPa, bar, °C) or US units (ft, in, lbf, psi, °F): Home › Units. The file keeps SI units, so nothing is lost when you switch.</li>
        <li>Allowable stresses are your inputs. Datum Pipe fills typical values; check them against the edition of the code you work to.</li>
        <li>Every tool has a small ? beside it. Help › Verification solves the benchmark problems in front of you. Results must be checked by a qualified engineer.</li>
      </ul>
    </div>
  </Dialog>
));

export const glossary = () => openDialog(() => {
  const [q, setQ] = createSignal("");
  const G: [string, string][] = [
    ["Element", "One row of the table: a piece of pipe between two nodes, with its size, material, conditions and component (bend, tee, reducer, valve, flange)."],
    ["Node", "A numbered point where elements join, and where supports, nozzles and loads go."],
    ["Anchor", "A support that holds the pipe still in all six directions: three movements and three rotations."],
    ["Rest", "A support under the pipe that carries its weight (+Y). In 0.1 it also holds the pipe down."],
    ["Guide", "A support that stops the pipe moving sideways but lets it slide along its axis."],
    ["Line stop", "A support that stops the pipe moving along its axis."],
    ["Constant-force support", "A spring hanger that carries the same load whatever the pipe's movement."],
    ["Nozzle", "Where the pipe joins a pump, vessel or other equipment. The loads there are what the equipment must carry."],
    ["SUS (sustained)", "The weight and pressure case: loads that act all the time. Its stresses are checked against the hot allowable stress."],
    ["OPE (operating)", "Everything at once: weight, temperature, pressure and imposed displacements. Used for support and nozzle loads; no code check."],
    ["EXP (expansion)", "The displacement stress range between the installed and operating states. Checked against the allowable stress range."],
    ["HYD (hydrotest)", "The pipe full of water at the test pressure: the loads the supports see during the test."],
    ["SIF (stress intensification factor)", "How much higher the fatigue stress is at a bend or tee than in straight pipe under the same moment."],
    ["Flexibility factor", "How much more a bend bends than a straight pipe of the same length, because its section ovalises."],
    ["Flexibility characteristic h", "h = T·R / r² for a bend (wall T, bend radius R, mean pipe radius r). The SIFs and flexibility factor follow from it."],
    ["Allowable stress range", "SA = f (1.25 Sc + 0.25 Sh): the limit for the expansion stress range, from the cold and hot allowables and the cycle factor f."],
    ["Liberal allowable", "Adds the unused part of the hot allowable (Sh − SL) to the allowable stress range."],
    ["Hoop stress", "The stress round the pipe wall from internal pressure."],
    ["Minimum wall", "The wall thickness the pressure needs, plus corrosion allowance and mill tolerance."],
    ["Mill tolerance", "How much thinner than nominal the pipe may be made: usually 12.5 %."],
    ["Equilibrium check", "After every solve the support reactions are added up: they must balance the applied loads."],
    ["Isometric", "A drawing of the piping at 30° angles so all three directions show, with dimensions and support symbols."],
  ];
  return <Dialog title="Glossary"><input placeholder="Search" value={q()} onInput={e => setQ(e.currentTarget.value)} style={{ width: "100%", "margin-bottom": "8px" }} />
    <dl class="gloss"><For each={G.filter(g => (g[0] + g[1]).toLowerCase().includes(q().toLowerCase()))}>{([t, d]) => <><dt>{t}</dt><dd>{d}</dd></>}</For></dl></Dialog>;
});

/** Recover unsaved work after a crash. */
export const recoverDialog = (name: string, saved: string, done: () => void) => openDialog(() => (
  <Dialog title="Recover unsaved work?" okText="Recover" cancel="Not now" ok={async () => { const r = await call<Out>("recover", { keep: true }); if (r) { setOut(r); done(); } }}
    extra={<button class="btn" onClick={() => { call("recover", { keep: false }); closeDialog(); }}>Discard</button>}>
    <p>Datum Pipe closed before “{name}” was saved. A copy from {saved} was kept.</p>
  </Dialog>
));
