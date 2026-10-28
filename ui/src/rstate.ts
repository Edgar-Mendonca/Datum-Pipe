// Results state: the last analysis, whether the model changed since, the load case shown, and what
// the 3D view shows (stress ratio colours, the deformed shape and its scale).
import { createSignal, batch } from "solid-js";
import { Results, CaseResult, invoke, num } from "./api";
import { model, toast, onEditHook, onLoadHook, setTab } from "./store";
import { setSolve } from "./cmds";

export const [results, setResults] = createSignal<Results | null>(null);
export const [stale, setStale] = createSignal(false);
export const [solving, setSolving] = createSignal(false);
export const [caseId, setCaseId] = createSignal<number | null>(null);
export type ShowRes = { colour: boolean; deformed: boolean; scale: number; anim: boolean; tags: boolean };
export const [show, setShow] = createSignal<ShowRes>({ colour: true, deformed: false, scale: 0, anim: false, tags: true });
export const setShowPart = (s: Partial<ShowRes>) => setShow({ ...show(), ...s });

/** The case shown: the chosen one, else the first expansion case, else the first. */
export const curCase = (): CaseResult | null => {
  const r = results(); if (!r) return null;
  return r.cases.find(c => c.id === caseId()) ?? r.cases.find(c => c.kind === "exp") ?? r.cases[0] ?? null;
};
/** The highest stress ratio of every segment in a case (or over every checked case). */
export const segRatios = (c: CaseResult | null, nseg: number): number[] => {
  const out = new Array(nseg).fill(-1);
  const r = results(); if (!r) return out;
  const cs = c && c.stresses.length ? [c] : r.cases.filter(x => x.stresses.length);
  for (const x of cs) for (const s of x.stresses) out[s.seg] = Math.max(out[s.seg], s.ratio);
  return out;
};

onEditHook(() => { if (results()) setStale(true); });
onLoadHook(() => { setResults(null); setStale(false); setCaseId(null); });

setSolve(async () => {
  if (solving() || !model()) return;
  setSolving(true);
  try {
    const r = await invoke<Results>("analyze");
    batch(() => { setResults(r); setStale(false); if (!r.cases.some(c => c.id === caseId())) setCaseId(null); setTab("results"); });
    const bad = r.cases.filter(c => c.imbalance > 1e-6);
    const worst = r.cases.flatMap(c => c.stresses).reduce((m, s) => Math.max(m, s.ratio), 0);
    const fails = r.cases.flatMap(c => c.stresses).filter(s => !s.pass).length + r.walls.filter(w => !w.pass).length;
    const ms = r.stats.ms < 1 ? "under 1 ms" : r.stats.ms < 2000 ? `${Math.round(r.stats.ms)} ms` : `${(r.stats.ms / 1000).toFixed(1)} s`;
    const msg = `Analyzed ${r.cases.length} load cases (${r.stats.equations} equations) in ${ms}. ${bad.length ? `The reactions do not balance the loads in ${bad.map(c => c.name).join(", ")}: see Results.` : "Reactions balance the loads in every case."} ${worst > 0 ? `Highest stress ratio ${num(worst, 3)}${fails ? `: ${fails} checks fail.` : ", every check passes."}` : ""}`;
    toast([...r.warnings, msg].join(" "), !r.warnings.length && !bad.length && !fails);
  } catch (e) { setResults(null); toast(String(e)); }
  finally { setSolving(false); }
});
