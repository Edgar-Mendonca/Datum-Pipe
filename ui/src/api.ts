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
  | { kind: "reducer"; to: number } | { kind: "rigid"; rigid: RigidKind; mass: number } | ({ kind: "bellows" } & Bellows);
export interface Bellows { axial: number; lateral: number; angular: number; torsion: number; area: number; tied: boolean; mass: number }
export interface Element { id: number; from: number; to: number; d: V3; pipe: number; material: number; cond: number; comp: Comp; extra: number }
export interface Pipe { id: number; name: string; nps: string; sched: string; od: number; wall: number; corrosion: number; mill: number; ins_t: number; ins_rho: number; clad_t: number; clad_rho: number; fluid_rho: number }
export interface Material { id: number; name: string; rho: number; nu: number; e: [number, number][]; alpha: [number, number][]; allow: [number, number][]; quality: number; weld: number; y: number; source: string }
export interface Cond { id: number; name: string; t: V3; p: V3; hydro: number }
export type SupKind =
  | { kind: "anchor" } | { kind: "rest" } | { kind: "guide" } | { kind: "linestop" } | { kind: "restraint"; dir: V3 }
  | { kind: "spring"; dir: V3; k: number } | { kind: "rotspring"; dir: V3; k: number } | { kind: "constant"; load: number } | { kind: "hanger"; k: number; cold: number };
export interface Support { id: number; node: number; kind: SupKind; k: number; disp: [number[], number[], number[]]; tag: string; one_way: boolean; gap: number; mu: number }
export interface Nozzle { id: number; node: number; name: string; axis: V3; allow: [number, number, number, number]; stiff?: [number, number, number, number, number]; vessel?: V3 }
export interface Load { id: number; node: number; f: V3; m: V3 }
export type CaseKind = "sus" | "ope" | "exp" | "hyd" | "occ";
export type Recipe = { kind: "sum"; w: boolean; water: boolean; t: number; d: number; p: number; wind?: boolean; seis?: boolean; relief?: boolean; neg?: boolean } | { kind: "range"; a: number; b: number };
export interface LoadCase { id: number; name: string; kind: CaseKind; recipe: Recipe }
export interface Project { project: string; line: string; calc: string; by: string; checked: string; rev: string; client: string }
export interface Settings { ambient: number; g: number; hydro: boolean; liberal: boolean; cycles: number; bend_pressure: boolean; auto_cases: boolean; pressure_elongation: boolean; branch_ze: boolean; step: number; wind_q: number; wind_cd: number; wind_dir: V3; seis: V3; wind_profile?: [number, number][]; occ_both?: boolean; springs: SpringSize[]; variability: number }
export interface Model {
  format: number; name: string; notes: string; units: "SI" | "US"; project: Project; settings: Settings;
  nodes: Node[]; elements: Element[]; pipes: Pipe[]; materials: Material[]; conds: Cond[]; supports: Support[]; nozzles: Nozzle[]; loads: Load[]; cases: LoadCase[]; next: number; equipment?: Equipment[]; flanges?: FlangeSpec[]; reliefs?: Relief[]; vessels?: Vessel[];
}
export interface Out { m: Model; path: string | null; dirty: boolean; undo: boolean; redo: boolean; created: number[]; warnings: string[]; geom: Geom | null; geom_error: string | null }

// ---------- analysis geometry and results (crates/core/src/geom.rs, solve.rs) ----------
export interface GPoint { p: V3; label: string; node: number | null }
export type GShape = { kind: "straight" } | { kind: "arc"; centre: V3; normal: V3; r: number; angle: number };
export interface Seg { a: number; b: number; shape: GShape; element: number; pipe: number; pipe_b: number; material: number; cond: number; rigid: [RigidKind, number] | null; bellows?: Bellows; bend: number | null; extra: number; length: number }
export interface GTee { node: number; point: number; kind: TeeKind; pad: number; pipe: number; normal: V3 }
export interface Geom { points: GPoint[]; segs: Seg[]; tees: GTee[]; node_point: Record<string, number>; warnings: string[] }
export type Six = [number, number, number, number, number, number];
export interface Reaction { support: number; node: number; f: Six }
export interface NozzleLoad { nozzle: number; node: number; local: Six; shear: number; bending: number; ratios: [number, number, number, number]; pass: boolean | null }
export interface Stress { seg: number; end: number; point: number; stress: number; allow: number; ratio: number; pass: boolean; ii: number; io: number }
export interface CaseResult { id: number; name: string; kind: CaseKind; disp: Six[]; forces: [Six, Six][]; reactions: Reaction[]; nozzles: NozzleLoad[]; stresses: Stress[]; applied: Six; reacted: Six; imbalance: number; worst: number | null; contacts: Contact[]; iterations: number; friction: Friction[]; equipment?: EquipCheck[]; flanges?: FlangeCheck[] }
export interface Friction { support: number; node: number; state: "sticking" | "sliding" | "free"; force: V3; normal: number; mu: number }
export interface Contact { support: number; node: number; dir: V3; state: "touching" | "open" | "lifted off"; movement: number; gap: number }
export interface Wall { element: number; case: number; p: number; t: number; s: number; t_design: number; t_min: number; t_nominal_needed: number; t_nominal: number; hoop: number; ratio: number; pass: boolean; at: string; i: number; od: number; thick: boolean }
export interface Sif { ii: number; io: number; k: number; h: number }
export interface SolveStats { equations: number; nonzeros: number; residual: number; ms: number }
export interface Results { geom: Geom; cases: CaseResult[]; walls: Wall[]; sifs: [Sif, Sif][]; stats: SolveStats; warnings: string[]; version: string; hangers: HangerDesign[]; reliefs?: ReliefResult[] }
export interface Check { bench: number; name: string; quantity: string; q: string; reference: number; result: number; error: number; tolerance: number; source: string; pass: boolean }
export interface Example { id: string; name: string; what: string }
export interface Size { nps: string; od: number; schedules: [string, number][] }
export interface LibMaterial { id: string; name: string; family: string; rho: number; nu: number; e: [number, number][]; alpha: [number, number][]; allow: [number, number][]; sy: number; su: number; source: string }
export interface Library { sizes: Size[]; materials: LibMaterial[]; source: string; springs: SpringSize[] }
export interface SpringSize { name: string; k: number; min: number; max: number }
export interface HangerDesign { support: number; node: number; hot: number; travel: number; size: string; k: number; cold: number; variability: number; range: [number, number] }
export interface Shown { q: string; label: string; factor: number; offset: number; dim?: number[] }
export interface Typed { sym: string; factor: number; offset: number; dim: number[] }
export interface Options {
  autosave: number; beta: boolean; theme: string; openLast: boolean; units: "SI" | "US"; contours: "rainbow" | "viridis"; labels: boolean; deform: number;
  paper: string; landscape: boolean; company: string; autoAnalyze: boolean;
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

export type EquipKind = "turbine" | "compressor" | "pump";
export interface Coef { single: [number, number, number]; de_cap: number; de_over: [number, number]; combined: [number, number, number]; dc_cap: number; dc_over: [number, number]; comp: [number, number, number, number, number, number] }
export interface PumpAllow { nozzle: number; f: V3; m: V3; fr: number; mr: number }
export interface Equipment { name: string; kind: EquipKind; nozzles: number[]; reference: number; shaft: V3; factor: number; coef: Coef; pump: PumpAllow[] }
export interface EquipRow { what: string; unit: string; value: number; allow: number; ratio: number }
export interface EquipCheck { name: string; kind: EquipKind; rows: EquipRow[]; pass: boolean }
export type PartKind = { kind: "cylinder" } | { kind: "ellipsoidal" } | { kind: "torispherical"; crown: number; knuckle: number } | { kind: "hemispherical" } | { kind: "cone"; d_end: number };
export type VPart = { name: string; d: number; len: number; t: number; e: number } & PartKind;
export interface VNozzle { name: string; part: number; pos: number; angle: number; nps: string; od: number; t: number; pipe: boolean; leg: number; pad_od: number; pad_t: number; pad_leg: number; proj: number }
export interface NozzleResult { name: string; d: number; tr: number; trn: number; a_req: number; a1: number; a2: number; a41: number; a5: number; a42: number; a_avail: number; small?: boolean; area_pass: boolean; t_neck_req: number; t_neck: number; neck_pass: boolean; pass: boolean; at: [number, number]; warnings: string[] }
export interface Saddles { a: number; theta: number; b: number; welded: boolean; rings: boolean }
export interface SaddleRow { what: string; stress: number; allow: number; ratio: number; pass: boolean }
export interface SaddleResult { q: number; m1: number; m2: number; k: number[]; rows: SaddleRow[]; pass: boolean }
export interface Skirt { h: number; t: number; ring_od: number; ring_id: number; ring_t: number; bolts: number; bolt_d: number; bc: number; bolt_s: number; bearing: number }
export interface SectionRow { name: string; z: number; m_wind: number; m_seis: number; weight: number; tension: number; compression: number; allow_t: number; allow_c: number; pass: boolean }
export interface SupportResult { shear_wind: number; moment_wind: number; shear_seis: number; moment_seis: number; weight_op: number; weight_empty: number; sections: SectionRow[]; bolt_load: number; bolt_stress: number; bolt_allow: number; bearing: number; bearing_allow: number; ring_t_req: number; ring_t: number; pass: boolean }
export interface Vessel { skirt?: Skirt | null; wind_cd?: number; wind_dfac?: number; seis?: number; emod?: number; base_el?: number; saddles?: Saddles | null; nozzles?: VNozzle[]; name: string; vertical: boolean; p: number; temp: number; s: number; s_cold: number; sy: number; corrosion: number; under: number; rho: number; liquid: number; level: number; extra: number; parts: VPart[] }
export interface VPartResult { name: string; z: [number, number]; p: number; static_head: number; t_req: number; t_min: number; t_nom: number; mawp: number; mawp_top: number; map: number; test_stress: number; test_allow: number; pass: boolean; formula: string; volume: number; metal: number }
export interface VesselResult { name: string; parts: VPartResult[]; mawp: number; map: number; test: number; governs: number | null; volume: number; empty: number; operating: number; test_weight: number; pass: boolean; outline: [number, number, number][]; nozzles?: NozzleResult[]; saddles?: SaddleResult | null; support?: SupportResult | null; warnings: string[] }
export interface VesselCheck { result: VesselResult | null; error: string | null }
export interface Relief { name: string; node: number; flow: number; k: number; temp: number; mw: number; area: number; dir: V3; dlf: number; ambient: number }
export interface ReliefResult { name: string; node: number; area: number; velocity: number; pressure: number; choked: boolean; stat: number; thrust: number; dlf: number; force: V3 }
export interface FlangeSpec { element: number; gasket: number; rating: number }
export interface FlangeCheck { element: number; at: string; force: number; moment: number; pressure: number; equivalent: number; rating: number; ratio: number; pass: boolean }
