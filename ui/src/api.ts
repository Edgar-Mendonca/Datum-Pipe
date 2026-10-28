// Calls into the Rust shell and the shapes it returns (they mirror crates/core/src/model.rs).
declare global {
  interface Window { __TAURI__?: { core: { invoke: (cmd: string, args?: object) => Promise<any> }; event: { listen: (e: string, f: (x: { payload: any }) => void) => Promise<() => void> } } }
}

export const invoke = <T = any>(cmd: string, args?: object): Promise<T> =>
  window.__TAURI__ ? window.__TAURI__.core.invoke(cmd, args) : Promise.reject("Datum Pipe must run inside its desktop window.");
export const listen = (e: string, f: (p: any) => void) => window.__TAURI__?.event.listen(e, x => f(x.payload));

export type V3 = [number, number, number];
export interface Node { num: number; x: number; y: number; z: number }
export type BendKind = "long" | "short" | "custom";
export type TeeKind = "welding" | "fabricated" | "reinforced";
export type RigidKind = "valve" | "flange" | "slipon" | "rigid";
export type Comp =
  | { kind: "pipe" } | { kind: "bend"; bend: BendKind; r: number } | { kind: "tee"; tee: TeeKind; pad: number }
  | { kind: "reducer"; to: number } | { kind: "rigid"; rigid: RigidKind; mass: number };
export interface Element { id: number; from: number; to: number; d: V3; pipe: number; material: number; cond: number; comp: Comp; extra: number }
export interface Pipe { id: number; name: string; nps: string; sched: string; od: number; wall: number; corrosion: number; mill: number; ins_t: number; ins_rho: number; clad_t: number; clad_rho: number; fluid_rho: number }
export interface Material { id: number; name: string; rho: number; nu: number; e: [number, number][]; alpha: [number, number][]; allow: [number, number][]; quality: number; weld: number; y: number; source: string }
export interface Cond { id: number; name: string; t: V3; p: V3; hydro: number }
export type SupKind =
  | { kind: "anchor" } | { kind: "rest" } | { kind: "guide" } | { kind: "linestop" } | { kind: "restraint"; dir: V3 }
  | { kind: "spring"; dir: V3; k: number } | { kind: "rotspring"; dir: V3; k: number } | { kind: "constant"; load: number };
export interface Support { id: number; node: number; kind: SupKind; k: number; disp: [number[], number[], number[]]; tag: string }
export interface Nozzle { id: number; node: number; name: string; axis: V3; allow: [number, number, number, number] }
export interface Load { id: number; node: number; f: V3; m: V3 }
export type CaseKind = "sus" | "ope" | "exp" | "hyd" | "occ";
export type Recipe = { kind: "sum"; w: boolean; water: boolean; t: number; d: number; p: number } | { kind: "range"; a: number; b: number };
export interface LoadCase { id: number; name: string; kind: CaseKind; recipe: Recipe }
export interface Project { project: string; line: string; calc: string; by: string; checked: string; rev: string; client: string }
export interface Settings { ambient: number; g: number; hydro: boolean; liberal: boolean; cycles: number; bend_pressure: boolean; auto_cases: boolean; step: number }
export interface Model {
  format: number; name: string; notes: string; units: "SI" | "US"; project: Project; settings: Settings;
  nodes: Node[]; elements: Element[]; pipes: Pipe[]; materials: Material[]; conds: Cond[]; supports: Support[]; nozzles: Nozzle[]; loads: Load[]; cases: LoadCase[]; next: number;
}
export interface Out { m: Model; path: string | null; dirty: boolean; undo: boolean; redo: boolean; created: number[]; warnings: string[]; geom: Geom | null; geom_error: string | null }

// ---------- analysis geometry and results (crates/core/src/geom.rs, solve.rs) ----------
export interface GPoint { p: V3; label: string; node: number | null }
export type GShape = { kind: "straight" } | { kind: "arc"; centre: V3; normal: V3; r: number; angle: number };
export interface Seg { a: number; b: number; shape: GShape; element: number; pipe: number; pipe_b: number; material: number; cond: number; rigid: [RigidKind, number] | null; bend: number | null; extra: number; length: number }
export interface GTee { node: number; point: number; kind: TeeKind; pad: number; pipe: number; normal: V3 }
export interface Geom { points: GPoint[]; segs: Seg[]; tees: GTee[]; node_point: Record<string, number>; warnings: string[] }
export type Six = [number, number, number, number, number, number];
export interface Reaction { support: number; node: number; f: Six }
export interface NozzleLoad { nozzle: number; node: number; local: Six; shear: number; bending: number; ratios: [number, number, number, number]; pass: boolean | null }
export interface Stress { seg: number; end: number; point: number; stress: number; allow: number; ratio: number; pass: boolean; ii: number; io: number }
export interface CaseResult { id: number; name: string; kind: CaseKind; disp: Six[]; forces: [Six, Six][]; reactions: Reaction[]; nozzles: NozzleLoad[]; stresses: Stress[]; applied: Six; reacted: Six; imbalance: number; worst: number | null }
export interface Wall { element: number; case: number; p: number; t: number; s: number; t_design: number; t_min: number; t_nominal_needed: number; t_nominal: number; hoop: number; ratio: number; pass: boolean }
export interface Sif { ii: number; io: number; k: number; h: number }
export interface SolveStats { equations: number; nonzeros: number; residual: number; ms: number }
export interface Results { geom: Geom; cases: CaseResult[]; walls: Wall[]; sifs: [Sif, Sif][]; stats: SolveStats; warnings: string[]; version: string }
export interface Check { bench: number; name: string; quantity: string; q: string; reference: number; result: number; error: number; tolerance: number; source: string; pass: boolean }
export interface Example { id: string; name: string; what: string }
export interface Size { nps: string; od: number; schedules: [string, number][] }
export interface LibMaterial { id: string; name: string; family: string; rho: number; nu: number; e: [number, number][]; alpha: [number, number][]; allow: [number, number][]; sy: number; su: number; source: string }
export interface Library { sizes: Size[]; materials: LibMaterial[]; source: string }
export interface Shown { q: string; label: string; factor: number; offset: number }
export interface Options {
  autosave: number; beta: boolean; theme: string; openLast: boolean; units: "SI" | "US"; contours: "rainbow" | "viridis"; labels: boolean; deform: number;
  paper: string; landscape: boolean; company: string;
}

/** Rounding noise to zero: values smaller than a billionth of the scale they belong to. */
export const clean = (v: number, scale: number) => (Math.abs(v) <= 1e-9 * Math.abs(scale) ? 0 : v);

/** A number typed by a person: "−" and "," are understood ("1,5" is 1.5, "1,200" is 1200); empty or
 * not a number gives null. */
export const parseNum = (s: string): number | null => {
  let t = s.trim().replace(/[−–]/g, "-").replace(/\s/g, "");
  if (!t) return null;
  if (t.includes(",")) t = t.includes(".") || /^-?\d{1,3}(,\d{3})+$/.test(t) ? t.replace(/,/g, "") : t.replace(",", ".");
  const v = Number(t);
  return Number.isFinite(v) ? v : null;
};

/** Numbers for people: up to 4 significant digits, no trailing zeros. */
export const num = (v: number, sig = 4) => {
  if (!isFinite(v)) return v > 0 ? "∞" : v < 0 ? "−∞" : "—";
  if (v === 0) return "0";
  const a = Math.abs(v);
  if (a >= 1e7 || a < 1e-3) return v.toExponential(Math.max(0, sig - 1)).replace(/\.?0+e/, "e").replace("-", "−");
  return String(+v.toPrecision(Math.max(sig, Math.ceil(Math.log10(a + 1))))).replace("-", "−");
};
