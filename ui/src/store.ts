// Window state: the open model, the selection, results, menus, dialogs and messages.
import { createSignal, batch, JSX } from "solid-js";
import { invoke, Out, Options, Model } from "./api";

export const [out, setOutRaw] = createSignal<Out | null>(null);
export const model = (): Model | undefined => out()?.m;
export const setOut = (o: Out | null | undefined) => {
  if (!o) return;
  batch(() => {
    setOutRaw(o);
    // Drop selected items that no longer exist (deleted or undone away).
    const ids = new Set([...o.m.elements, ...o.m.supports, ...o.m.nozzles, ...o.m.loads].map(x => x.id)), nodes = new Set(o.m.nodes.map(n => n.num));
    const s = sel();
    if (s && ((s.kind === "node" && !nodes.has(s.id)) || (s.kind !== "node" && !ids.has(s.id)))) setSel(null);
  });
  if (o.warnings?.length) toast(o.warnings[0].startsWith("Imported") ? o.warnings.join(" ") : `Opened with repairs: ${o.warnings.join(" ")}`);
};

/** auto, light or dark; auto follows Windows. */
export const [theme, setThemeRaw] = createSignal("auto");
const darkMq = typeof matchMedia === "function" ? matchMedia("(prefers-color-scheme: dark)") : null;
export const [themeTick, setThemeTick] = createSignal(0);
export const applyTheme = () => { document.documentElement.dataset.theme = theme() === "auto" ? (darkMq?.matches ? "dark" : "light") : theme(); setThemeTick(t => t + 1); };
darkMq?.addEventListener("change", () => theme() === "auto" && applyTheme());
export const setTheme = (t: string) => { setThemeRaw(t); applyTheme(); const o = { ...opts(), theme: t }; setOpts(o); invoke("set_options", { opts: o }).catch(() => {}); };
export const [opts, setOpts] = createSignal<Options>({ autosave: 5, beta: false, theme: "auto", openLast: false, units: "SI", contours: "rainbow", labels: true, deform: 0, paper: "A4", landscape: false, company: "", autoAnalyze: true });

export type View = "model" | "results" | "iso" | "report" | "vessel";
export const [view, setView] = createSignal<View>("model");
export const [backstage, setBackstage] = createSignal<string | null>("home");
export const [tab, setTab] = createSignal("home");

// ---------- selection ----------
/** What is selected: an element (the table row), a node, a support, a nozzle or a load. */
export type Sel = { kind: "element" | "node" | "support" | "nozzle" | "load"; id: number };
export const [sel, setSel] = createSignal<Sel | null>(null);

// ---------- editing ----------
export const act = async (op: object | string, quiet = false): Promise<Out | null> => {
  try { const o = await invoke<Out>("apply", { op }); setOut(o); note(describeOp(op), "edit"); onEdit(); return o; }
  catch (e) { if (!quiet) toast(String(e)); return null; }
};
/** What runs after every edit (results go out of date, vessels are checked again). Every hook runs:
 *  a single slot let the results module's second hook replace its first (found in 0.3.1). */
const edited: (() => void)[] = [];
export const onEditHook = (f: () => void) => { edited.push(f); };
export const onEdit = () => edited.forEach(f => f());
export const call = async <T = any>(cmd: string, args?: object): Promise<T | undefined> => {
  try { return await invoke<T>(cmd, args); } catch (e) { toast(String(e)); return undefined; }
};
const loadHooks: (() => void)[] = [];
export const onLoadHook = (f: () => void) => { loadHooks.push(f); };
const loaded = () => loadHooks.forEach(f => f());
export const loadOut = async (cmd: string, args?: object) => {
  const o = await call<Out | null>(cmd, args);
  if (o) batch(() => { setSel(null); setOut(o); loaded(); setBackstage(null); setView("model"); });
  return o;
};

// ---------- messages, menus, dialogs ----------
/** Messages (0.6.1): everything the window tells you, kept with its time (the bottom panel's
 *  Messages tab), so a note that vanished can be read again. `kind`: note, ok, warn, edit, solve. */
export type Note = { t: Date; text: string; kind: "note" | "ok" | "warn" | "edit" | "solve" };
export const [log, setLog] = createSignal<Note[]>([]);
export const [unread, setUnread] = createSignal(0);
/** The bottom panel's tab: the element table, the messages or the solver's log. */
export const [ptab, setPtabRaw] = createSignal<string>("elements");
export const setPtab = (t: string) => { setPtabRaw(t); if (t === "messages") setUnread(0); };
export const note = (text: string, kind: Note["kind"] = "note") => {
  setLog(l => [...l.slice(-499), { t: new Date(), text, kind }]);
  if ((kind === "warn") && ptab() !== "messages") setUnread(n => n + 1);
};
export const [msg, setMsg] = createSignal<{ text: string; ok: boolean } | null>(null);
let timer = 0;
export const toast = (text: string, ok = false, keep = true) => { setMsg({ text, ok }); if (keep) note(text, ok ? "ok" : "warn"); clearTimeout(timer); timer = window.setTimeout(() => setMsg(null), ok ? 3000 : 7000); };
/** An edit in words, for the Messages tab. */
const VERB: Record<string, string> = { Add: "Added", Set: "Changed", Delete: "Deleted", Split: "Inserted a node", Loop: "Added an expansion loop", Renumber: "Renumbered the nodes", Duplicate: "Copied", SetSettings: "Changed the settings", SetCases: "Changed the load cases", RecommendCases: "Rebuilt the load cases", SetVessels: "Changed the vessels", SetReliefs: "Changed the relief valves", SetSoils: "Changed the soils", SetEquipment: "Changed the equipment", SetFlanges: "Changed the flange checks", SetProject: "Changed the title block", Move: "Moved" };
export const describeOp = (op: object | string): string => {
  const k = typeof op === "string" ? op : Object.keys(op)[0] ?? "Edit";
  const body = typeof op === "string" ? null : (op as any)[k];
  const item = body && typeof body === "object" && body.item ? Object.keys(body.item)[0] : null;
  const verb = VERB[k] ?? k.replace(/([a-z])([A-Z])/g, "$1 $2");
  return item ? `${verb} ${item.replace(/([a-z])([A-Z])/g, "$1 $2").toLowerCase()}` : verb;
};
export type Item = { label: string; run?: () => void; keys?: string; off?: boolean; on?: boolean; sub?: Item[]; icon?: string } | "-";
export const [menu, setMenu] = createSignal<{ x: number; y: number; items: Item[] } | null>(null);
export const openMenu = (e: MouseEvent | { x: number; y: number }, items: Item[]) => {
  if ("preventDefault" in e) e.preventDefault();
  const r = (e as MouseEvent).currentTarget instanceof HTMLElement && (e as MouseEvent).type === "click" ? ((e as MouseEvent).currentTarget as HTMLElement).getBoundingClientRect() : null;
  setMenu({ x: r ? r.left : (e as MouseEvent).clientX ?? (e as any).x, y: r ? r.bottom + 2 : (e as MouseEvent).clientY ?? (e as any).y, items });
};
export const [dialogs, setDialogs] = createSignal<(() => JSX.Element)[]>([]);
export const openDialog = (d: () => JSX.Element) => setDialogs(v => [...v, d]);
export const closeDialog = () => setDialogs(v => v.slice(0, -1));
