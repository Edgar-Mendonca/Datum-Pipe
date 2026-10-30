// Commands: what the ribbon, menus, keys and Tell Me run.
import { Out, Element, SupKind, Model } from "./api";
import { setOut, act, call, sel, setSel, setBackstage, loadOut, toast, setView, model, onEdit } from "./store";
import * as D from "./dialogs";

/** The node a support or load goes on: the selected node, or the To node of the selected element. */
export const selNode = (m = model()): number | null => {
  const s = sel(); if (!s || !m) return null;
  if (s.kind === "node") return s.id;
  if (s.kind === "element") return m.elements.find(e => e.id === s.id)?.to ?? null;
  const x = [...m.supports, ...m.nozzles, ...m.loads].find(x => x.id === s.id);
  return x?.node ?? null;
};
const selElement = (m: Model): Element | undefined => { const s = sel(); return s?.kind === "element" ? m.elements.find(e => e.id === s.id) : undefined; };

export const SUPPORTS: [string, string, SupKind][] = [
  ["anchor", "Anchor", { kind: "anchor" }], ["rest", "Rest (+Y)", { kind: "rest" }], ["guide", "Guide", { kind: "guide" }], ["linestop", "Line stop", { kind: "linestop" }],
  ["rx", "Restraint X", { kind: "restraint", dir: [1, 0, 0] }], ["ry", "Restraint Y", { kind: "restraint", dir: [0, 1, 0] }], ["rz", "Restraint Z", { kind: "restraint", dir: [0, 0, 1] }],
  ["skew", "Skewed restraint", { kind: "restraint", dir: [1, 0, 1] }],
  ["spring", "Spring (vertical)", { kind: "spring", dir: [0, 1, 0], k: 1e6 }], ["rotspring", "Rotational spring", { kind: "rotspring", dir: [1, 0, 0], k: 1e6 }],
  ["constant", "Constant-force support", { kind: "constant", load: 1000 }],
  ["hanger", "Spring hanger (sized)", { kind: "hanger", k: 0, cold: 0 }],
];

let solveHook: () => Promise<void> = async () => toast("Analysis arrives in Datum Pipe 0.1.1. File › Roadmap shows what comes when.");
export const setSolve = (f: () => Promise<void>) => { solveHook = f; };
let fitHook: () => void = () => {};
export const setFit = (f: () => void) => { fitHook = f; };
let viewHook: (v: string) => void = () => {};
export const setStdView = (f: (v: string) => void) => { viewHook = f; };

export const C = {
  // ---------- file ----------
  save: async (asNew = false) => { const o = await call<Out | null>("save", { asNew }); if (o) { setOut(o); toast("Saved.", true); } },
  // Each of these replaces the open model, so unsaved changes are offered for saving first.
  open: () => D.keepChanges(() => loadOut("open", { path: null }).then(o => { if (o) setTimeout(fitHook, 60); })),
  importPcf: () => D.keepChanges(() => loadOut("import_pcf", { path: null }).then(o => { if (o) setTimeout(fitHook, 60); })),
  openPath: (path: string) => D.keepChanges(() => loadOut("open", { path }).then(o => { if (o) setTimeout(fitHook, 60); })),
  newModel: (example: string | null) => D.keepChanges(() => loadOut("new_model", { example }).then(o => { if (o) setTimeout(fitHook, 60); })),
  print: () => setBackstage("print"),
  undo: async () => { const o = await call<Out>("history", { redo: false }); if (o) { setOut(o); onEdit(); } },
  redo: async () => { const o = await call<Out>("history", { redo: true }); if (o) { setOut(o); onEdit(); } },

  // ---------- elements ----------
  del: () => {
    const s = sel(); if (!s) return toast("Select an element, a support, a nozzle or a load first.");
    if (s.kind === "node") { const m = model()!; const ids = m.elements.filter(e => e.from === s.id || e.to === s.id).map(e => e.id); return act({ Delete: { ids } }).then(o => o && setSel(null)); }
    act({ Delete: { ids: [s.id] } }).then(o => o && setSel(null));
  },
  /** A new element after the selected one, continuing from its To node in the same direction. */
  addAfter: async () => {
    const m = model(); if (!m) return;
    const e = selElement(m) ?? m.elements[m.elements.length - 1];
    if (!e) return toast("Type the first element in the table: its length in DX, DY or DZ.");
    const step = m.settings.step || 10, used = new Set(m.nodes.map(n => n.num));
    let to = e.to + step; while (used.has(to)) to += step;
    const el = { ...e, id: 0, from: e.to, to, comp: { kind: "pipe" } };
    const o = await act({ Add: { item: { Element: el }, after: e.id } });
    if (o?.created.length) setSel({ kind: "element", id: o.created[0] });
  },
  split: () => { const m = model(); const e = m && selElement(m); if (!e) return toast("Select the element to split in the table or the 3D view."); act({ Split: { id: e.id, at: 0.5 } }); },
  renumber: () => { const m = model(); if (m) act({ Renumber: { start: m.settings.step || 10, step: m.settings.step || 10 } }).then(o => o && toast(`Nodes numbered ${m.settings.step || 10}, ${2 * (m.settings.step || 10)}, … in table order.`, true)); },
  units: (u: "SI" | "US") => { const m = model(); if (m && m.units !== u) act({ SetSettings: { s: m.settings, units: u } }); },
  settings: (s: Partial<Model["settings"]>) => { const m = model(); if (m) return act({ SetSettings: { s: { ...m.settings, ...s }, units: m.units } }); },

  // ---------- supports, nozzles, loads ----------
  support: async (key: string) => {
    const m = model(), n = selNode(m); if (!m) return;
    if (n === null) return toast("Select a node first: click it in the 3D view, or select an element (the support goes at its To node).");
    const k = SUPPORTS.find(s => s[0] === key)?.[2] ?? { kind: "anchor" };
    const o = await act({ Add: { item: { Support: { id: 0, node: n, kind: k, k: 0, disp: [[0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0]], tag: "", one_way: k.kind === "rest", gap: 0, mu: 0 } }, after: null } });
    if (o?.created.length) setSel({ kind: "support", id: o.created[0] });
  },
  nozzle: async () => {
    const m = model(), n = selNode(m); if (!m) return;
    if (n === null) return toast("Select the node at the equipment nozzle first.");
    const o = await act({ Add: { item: { Nozzle: { id: 0, node: n, name: `Nozzle ${n}`, axis: [0, 0, 0], allow: [0, 0, 0, 0] } }, after: null } });
    if (o?.created.length) setSel({ kind: "nozzle", id: o.created[0] });
  },
  load: async () => {
    const m = model(), n = selNode(m); if (!m) return;
    if (n === null) return toast("Select the node the force acts on first.");
    const o = await act({ Add: { item: { Load: { id: 0, node: n, f: [0, -1000, 0], m: [0, 0, 0] } }, after: null } });
    if (o?.created.length) setSel({ kind: "load", id: o.created[0] });
  },
  recommendCases: () => act("RecommendCases").then(o => o && toast("Load cases built from the model.", true)),

  // ---------- analysis and views ----------
  analyze: () => solveHook(),
  view: (v: "model" | "results" | "iso" | "report") => setView(v),
  fit: () => fitHook(),
  std: (v: string) => viewHook(v),
  planned: (what: string, v: string) => toast(`${what} arrives in ${v}. File › Roadmap shows what comes when.`),
};

export const ALL_CMDS: [string, () => void, string?][] = [
  ["Analyze", () => C.analyze(), "F5"], ["Save", () => C.save(), "Ctrl+S"], ["Save As", () => C.save(true)], ["Open", C.open, "Ctrl+O"], ["Import a piping component file (.pcf)", C.importPcf], ["New model", () => setBackstage("home"), "Ctrl+N"], ["Print / PDF", C.print, "Ctrl+P"],
  ["Undo", C.undo, "Ctrl+Z"], ["Redo", C.redo, "Ctrl+Y"], ["Delete", C.del, "Del"],
  ["Add an element after the selected one", C.addAfter, "Ctrl+Enter"], ["Insert a node (split the element)", C.split], ["Renumber the nodes", C.renumber],
  ...SUPPORTS.map(([k, l]) => [`Support: ${l}`, () => C.support(k)] as [string, () => void]),
  ["Expansion loop wizard", () => import("./tools").then(t => t.loopWizard())], ["Copy a section", () => import("./tools").then(t => t.copySection())], ["Mirror a section", () => import("./tools").then(t => t.copySection(true))],
  ["Nozzle point", C.nozzle], ["Force or moment at a node", C.load], ["Recommend load cases", C.recommendCases],
  ["SI units", () => C.units("SI")], ["US units", () => C.units("US")],
  ["Fit the model", C.fit, "F"], ["Isometric view", () => C.std("iso"), "1"], ["Plan view", () => C.std("plan"), "2"], ["Front view", () => C.std("front"), "3"], ["Side view", () => C.std("side"), "4"],
  ["Results tables", () => setView("results")], ["Isometric drawing", () => setView("iso")], ["Stress report", () => setView("report")],
  ["Options", () => setBackstage("options")], ["Check for updates", () => setBackstage("about")], ["Roadmap", () => setBackstage("roadmap")],
  ["Keyboard shortcuts", () => D.shortcuts(), "F1"], ["Getting started", () => D.gettingStarted()], ["Glossary", () => D.glossary()],
];
