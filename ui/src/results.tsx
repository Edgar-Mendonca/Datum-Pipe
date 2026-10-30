// Results: the ribbon's Results tab and the Results view, with a tab per load case and the tables
// the engine computed: summary, displacements, forces and moments, support loads, nozzle loads,
// code stresses, minimum wall and the equilibrium check.
import { For, Show, createSignal, createMemo, JSX } from "solid-js";
import { CaseResult, Results, num } from "./api";
import { model, setSel, setView, openMenu } from "./store";
import { C } from "./cmds";
import { G, S, B } from "./ribbon";
import { Help } from "./help";
import { results, stale, solving, show, setShowPart, curCase, setCaseId } from "./rstate";
import { unit, toU } from "./units";
import { supName } from "./panels";
import { Icon } from "./ui";

export const [tableKind, setTableKind] = createSignal("summary");
const TABLES: [string, string][] = [["summary", "Summary"], ["stress", "Code stresses"], ["disp", "Displacements"], ["forces", "Forces and moments"], ["supports", "Support loads"], ["nozzles", "Nozzle loads"], ["wall", "Minimum wall"], ["hangers", "Hangers"], ["equilibrium", "Equilibrium"]];
const KIND: Record<string, string> = { sus: "Sustained", ope: "Operating", exp: "Expansion range", hyd: "Hydrotest", occ: "Occasional" };
const CHECK: Record<string, string> = { sus: "Sustained stress ≤ Sh", exp: "Stress range ≤ SA", ope: "Loads only", hyd: "Loads only", occ: "≤ 1.33 Sh" };
const openTable = (k: string) => { setTableKind(k); setView("results"); };

export const Badge = (p: { pass: boolean | null | undefined }) => p.pass === null || p.pass === undefined ? <span class="muted">—</span> : <span class={p.pass ? "pass" : "fail"}>{p.pass ? "PASS" : "FAIL"}</span>;
const casePass = (c: CaseResult) => c.stresses.length ? c.stresses.every(s => s.pass) && c.nozzles.every(z => z.pass !== false) : c.nozzles.some(z => z.pass !== null) ? c.nozzles.every(z => z.pass !== false) : null;

export const ResultsRibbon = () => {
  const r = () => results();
  return <>
    <G l="Analyze" h="analyze"><B big i="solve" l="Analyze" run={C.analyze} off={solving()} title="Solve every load case (F5)" /></G>
    <G l="Load Case" h="results"><B big i="cases" l={curCase()?.name ?? "Load Case"} off={!r()} dd={e => openMenu(e, (r()?.cases ?? []).map(c => ({ label: `${c.name} · ${KIND[c.kind]}`, on: c.id === curCase()?.id, run: () => setCaseId(c.id) })))} /></G>
    <G l="Show" h="results"><S><B i="contour" l="Stress Ratio Colours" on={show().colour} off={!r()} run={() => setShowPart({ colour: !show().colour })} /><B i="deform" l="Deformed Shape" on={show().deformed} off={!r()} run={() => setShowPart({ deformed: !show().deformed })} /><B i="play" l="Animate" on={show().anim} off={!r()} run={() => setShowPart({ anim: !show().anim, deformed: true })} /></S></G>
    <G l="Tables" h="results"><S><B i="table" l="Summary" off={!r()} run={() => openTable("summary")} /><B i="table" l="Code Stresses" off={!r()} run={() => openTable("stress")} /><B i="table" l="Displacements" off={!r()} run={() => openTable("disp")} /></S>
      <S><B i="table" l="Support Loads" off={!r()} run={() => openTable("supports")} /><B i="nozzle" l="Nozzle Loads" off={!r()} run={() => openTable("nozzles")} /><B i="table" l="Forces" off={!r()} run={() => openTable("forces")} /></S></G>
    <G l="Overstressed" h="code"><B big i="info" l="Overstressed" off={!r()} dd={e => { const list = over(); openMenu(e, list.length ? list.slice(0, 30).map(o => ({ label: `${o.label} · ${o.case} · ratio ${num(o.ratio, 3)}`, run: () => jump(o.node) })) : [{ label: "Nothing is overstressed", off: true }]); }} /></G>
  </>;
};

/** Every failing point, worst first. */
const over = () => {
  const r = results(); if (!r) return [];
  return r.cases.flatMap(c => c.stresses.filter(s => !s.pass).map(s => ({ case: c.name, ratio: s.ratio, label: r.geom.points[s.point]?.label ?? "", node: nodeOf(r, s.point) }))).sort((a, b) => b.ratio - a.ratio);
};
/** The model node nearest a point (bends' near and far points belong to the bend's node). */
const nodeOf = (r: Results, pt: number) => r.geom.points[pt]?.node ?? (parseInt(r.geom.points[pt]?.label ?? "") || null);
const jump = (node: number | null) => { if (node !== null) { setSel({ kind: "node", id: node }); setView("model"); } };

const Th = (p: { children: JSX.Element; r?: boolean }) => <th class={p.r ? "r" : ""}>{p.children}</th>;
const u = (q: Parameters<typeof unit>[0]) => <span class="unit">{unit(q)}</span>;
const v = (q: Parameters<typeof toU>[0], x: number, sig = 4) => num(toU(q, x), sig);

export const ResultsPage = () => {
  const r = () => results();
  const c = () => curCase();
  return (
    <div class="page results">
      <h1>Results <Help k="results" /><Show when={r()}><span class="muted">{r()!.cases.length} load cases · {r()!.stats.equations} equations · Datum Pipe {r()!.version}</span></Show></h1>
      <Show when={r()} fallback={<div class="emptypage"><Icon n="solve" size={30} /><b>No results yet</b><span>Analyze the model (F5): every load case is solved, the stresses are checked and the support and nozzle loads are worked out.</span><button class="btn pri" onClick={C.analyze}>Analyze</button></div>}>
        <Show when={stale()}><p class="warnline">The model changed since this analysis. Press F5 to analyze again.</p></Show>
        <Show when={r()!.warnings.length}><p class="warnline">{r()!.warnings.join(" ")}</p></Show>
        <div class="rtabs"><For each={TABLES}>{([k, l]) => <button classList={{ on: tableKind() === k }} onClick={() => setTableKind(k)}>{l}</button>}</For></div>
        <Show when={!["summary", "wall", "equilibrium"].includes(tableKind())}>
          <div class="ctabs"><For each={r()!.cases}>{x => <button classList={{ on: x.id === c()?.id }} onClick={() => setCaseId(x.id)}><b>{x.name}</b><small>{KIND[x.kind]}</small><Badge pass={casePass(x)} /></button>}</For></div>
        </Show>
        <div class="rtable">
          <Show when={tableKind() === "summary"}><Summary r={r()!} /></Show>
          <Show when={tableKind() === "stress" && c()}><Stresses r={r()!} c={c()!} /></Show>
          <Show when={tableKind() === "disp" && c()}><Disp r={r()!} c={c()!} /></Show>
          <Show when={tableKind() === "forces" && c()}><Forces r={r()!} c={c()!} /></Show>
          <Show when={tableKind() === "supports" && c()}><Supports r={r()!} c={c()!} /></Show>
          <Show when={tableKind() === "nozzles" && c()}><Nozzles r={r()!} c={c()!} /></Show>
          <Show when={tableKind() === "wall"}><Walls r={r()!} /></Show>
          <Show when={tableKind() === "hangers"}><Hangers r={r()!} /></Show>
          <Show when={tableKind() === "equilibrium"}><Equilibrium r={r()!} /></Show>
        </div>
      </Show>
    </div>
  );
};

export const Summary = (p: { r: Results }) => (
  <table class="tbl"><thead><tr><Th>Case</Th><Th>Kind</Th><Th>Check</Th><Th r>Highest ratio</Th><Th>At</Th><Th r>Stress {u("stress")}</Th><Th r>Allowable</Th><Th>Result</Th><Th r>Equilibrium</Th></tr></thead><tbody>
    <For each={p.r.cases}>{c => { const w = c.worst !== null ? c.stresses[c.worst] : null; return <tr class="f" onClick={() => { setCaseId(c.id); setTableKind(c.stresses.length ? "stress" : "supports"); }}>
      <td><b>{c.name}</b></td><td>{KIND[c.kind]}</td><td>{CHECK[c.kind]}</td>
      <td class="r">{w ? num(w.ratio, 3) : "—"}</td><td>{w ? p.r.geom.points[w.point]?.label : ""}</td><td class="r">{w ? v("stress", w.stress) : ""}</td><td class="r">{w ? v("stress", w.allow) : ""}</td>
      <td><Badge pass={casePass(c)} /></td><td class="r" classList={{ bad: c.imbalance > 1e-6 }}>{c.imbalance < 1e-12 ? "< 1e-12" : c.imbalance.toExponential(1)}</td></tr>; }}</For>
    <Show when={p.r.walls.length}><tr class="f" onClick={() => setTableKind("wall")}><td><b>Pressure</b></td><td>Minimum wall</td><td>Nominal wall ≥ needed</td><td class="r">{num(Math.max(...p.r.walls.map(w => w.ratio)), 3)}</td><td /><td /><td /><td><Badge pass={p.r.walls.every(w => w.pass)} /></td><td /></tr></Show>
  </tbody></table>
);

const Stresses = (p: { r: Results; c: CaseResult }) => {
  const [worst, setWorst] = createSignal(false);
  // In table order: by element, then along it (its straight part, then its bend).
  const order = createMemo(() => { const m = model(); const idx = new Map((m?.elements ?? []).map((e, i) => [e.id, i])); return (si: number) => idx.get(p.r.geom.segs[si]?.element) ?? 0; });
  const rows = createMemo(() => { const s = [...p.c.stresses]; return worst() ? s.sort((a, b) => b.ratio - a.ratio) : s.sort((a, b) => order()(a.seg) - order()(b.seg)); });
  const el = (si: number) => { const e = model()?.elements.find(x => x.id === p.r.geom.segs[si]?.element); return e ? `${e.from}–${e.to}` : ""; };
  return <Show when={p.c.stresses.length} fallback={<p class="muted">{KIND[p.c.kind]} cases have no code stress check: see the support and nozzle loads.</p>}>
    <p class="ibtns"><label class="chk"><input type="checkbox" checked={worst()} onChange={e => setWorst(e.currentTarget.checked)} />Worst first</label><span class="muted small">{p.c.kind === "sus" ? "Sustained longitudinal stress SL (pressure and weight, wall less corrosion) against the hot allowable Sh." : p.c.kind === "exp" ? "Displacement stress range SE (nominal wall, in- and out-of-plane SIFs) against the allowable range SA." : "Sustained plus occasional stress against 1.33 Sh."}</span></p>
    <table class="tbl"><thead><tr><Th>Point</Th><Th>Element</Th><Th r>Stress {u("stress")}</Th><Th r>Allowable</Th><Th r>Ratio</Th><Th r>SIF in / out</Th><Th>Result</Th></tr></thead><tbody>
      <For each={rows()}>{s => <tr class="f" onClick={() => jump(nodeOf(p.r, s.point))}><td>{p.r.geom.points[s.point]?.label}</td><td>{el(s.seg)}</td><td class="r">{v("stress", s.stress)}</td><td class="r">{v("stress", s.allow)}</td><td class="r"><b>{num(s.ratio, 3)}</b></td><td class="r">{num(s.ii, 3)} / {num(s.io, 3)}</td><td><Badge pass={s.pass} /></td></tr>}</For></tbody></table>
  </Show>;
};

const Disp = (p: { r: Results; c: CaseResult }) => (
  <table class="tbl"><thead><tr><Th>Point</Th><Th r>DX {u("disp")}</Th><Th r>DY</Th><Th r>DZ</Th><Th r>RX {u("rot")}</Th><Th r>RY</Th><Th r>RZ</Th></tr></thead><tbody>
    <For each={p.c.disp}>{(d, i) => <tr><td>{p.r.geom.points[i()].label}</td><For each={[0, 1, 2]}>{k => <td class="r">{v("disp", Math.abs(d[k]) < 1e-12 ? 0 : d[k], 4)}</td>}</For><For each={[3, 4, 5]}>{k => <td class="r">{v("rot", Math.abs(d[k]) < 1e-14 ? 0 : d[k], 4)}</td>}</For></tr>}</For></tbody></table>
);

const Forces = (p: { r: Results; c: CaseResult }) => {
  const el = (si: number) => { const e = model()?.elements.find(x => x.id === p.r.geom.segs[si]?.element); return e ? `${e.from}–${e.to}` : ""; };
  return <>
    <p class="muted small">In the pipe's local axes: x along the pipe (from the From node to the To node), y up (across a vertical pipe: towards X), z across. The force and moment the part towards the From node exerts on the rest.</p>
    <table class="tbl"><thead><tr><Th>Element</Th><Th>Point</Th><Th r>Fx (axial) {u("force")}</Th><Th r>Fy</Th><Th r>Fz</Th><Th r>Mx (torsion) {u("moment")}</Th><Th r>My</Th><Th r>Mz</Th></tr></thead><tbody>
      <For each={p.c.forces}>{(f, i) => <For each={[0, 1]}>{end => <tr><td>{end === 0 ? el(i()) : ""}</td><td>{p.r.geom.points[end === 0 ? p.r.geom.segs[i()].a : p.r.geom.segs[i()].b]?.label}</td><For each={[0, 1, 2]}>{k => <td class="r">{v("force", f[end][k])}</td>}</For><For each={[3, 4, 5]}>{k => <td class="r">{v("moment", f[end][k])}</td>}</For></tr>}</For>}</For></tbody></table>
  </>;
};

/** The state of a one-way, gapped or friction support in a case: "touching", "open, 2.1 mm of
 *  3 mm", "sliding, 1.2 kN (μ 0.3)". */
export const contactText = (c: CaseResult, support: number) => [
  ...(c.contacts ?? []).filter(x => x.support === support)
    .map(x => x.state === "touching" ? "touching" : `${x.state}, ${v("disp", Math.abs(x.movement))} ${unit("disp")}${x.gap > 0 ? ` of ${v("disp", x.gap)}` : ""}`),
  ...(c.friction ?? []).filter(x => x.support === support && x.state !== "free")
    .map(x => `${x.state}, ${v("force", Math.hypot(...x.force))} ${unit("force")} (μ ${x.mu})`),
].join("; ");

export const Supports = (p: { r: Results; c: CaseResult }) => {
  const m = () => model();
  return <>
    <p class="muted small">The force and moment each support puts on the pipe, in global axes (Y up): a rest carrying weight shows a positive FY. The support itself carries the same load the other way. One-way rests and supports with a gap show whether they are touching, open or lifted off, with the pipe's movement along them; supports with friction whether they stick or slide, with the friction force (included in the loads).</p>
    <Show when={p.c.contacts?.length}><p class="muted small">{p.c.name}: the supports settled after {p.c.iterations} {p.c.iterations === 1 ? "solve" : "solves"}.</p></Show>
    <table class="tbl"><thead><tr><Th>Node</Th><Th>Support</Th><Th>Tag</Th><Th>State</Th><Th r>FX {u("force")}</Th><Th r>FY</Th><Th r>FZ</Th><Th r>MX {u("moment")}</Th><Th r>MY</Th><Th r>MZ</Th></tr></thead><tbody>
      <For each={p.c.reactions} fallback={<tr><td colSpan={10} class="muted">No supports.</td></tr>}>{x => { const s = m()?.supports.find(y => y.id === x.support); return <tr class="f" onClick={() => jump(x.node)}><td>{x.node}</td><td>{s ? supName(s.kind) : ""}</td><td>{s?.tag}</td><td>{contactText(p.c, x.support)}</td><For each={[0, 1, 2]}>{k => <td class="r">{v("force", x.f[k])}</td>}</For><For each={[3, 4, 5]}>{k => <td class="r">{v("moment", x.f[k])}</td>}</For></tr>; }}</For></tbody></table>
  </>;
};

export const Nozzles = (p: { r: Results; c: CaseResult }) => (
  <Show when={p.c.nozzles.length} fallback={<p class="muted">No nozzle points in the model. Select the node at an equipment nozzle and use Supports › Nozzle Point.</p>}>
    <p class="muted small">The load the pipe puts on each nozzle, in the nozzle's axes: a along the nozzle, b and c across it. Ratios against the allowables you entered (— where none is given).</p>
    <table class="tbl"><thead><tr><Th>Nozzle</Th><Th r>Fa {u("force")}</Th><Th r>Fb</Th><Th r>Fc</Th><Th r>Ma (torsion) {u("moment")}</Th><Th r>Mb</Th><Th r>Mc</Th><Th r>Shear</Th><Th r>Bending</Th><Th r>Ratios a · s · t · b</Th><Th>Result</Th></tr></thead><tbody>
      <For each={p.c.nozzles}>{z => <tr class="f" onClick={() => jump(z.node)}><td>{model()?.nozzles.find(x => x.id === z.nozzle)?.name} ({z.node})</td><For each={[0, 1, 2]}>{k => <td class="r">{v("force", z.local[k])}</td>}</For><For each={[3, 4, 5]}>{k => <td class="r">{v("moment", z.local[k])}</td>}</For>
        <td class="r">{v("force", z.shear)}</td><td class="r">{v("moment", z.bending)}</td><td class="r">{z.ratios.map(x => x ? num(x, 2) : "—").join(" · ")}</td><td><Badge pass={z.pass} /></td></tr>}</For></tbody></table>
  </Show>
);

export const Hangers = (p: { r: Results }) => <Show when={p.r.hangers?.length} fallback={<p class="muted">No spring hangers. Put one on a node from Supports › Spring Hanger; Analyze sizes it.</p>}>
  <p class="muted small">Hot load H: the weight the hanger carries with the pipe held level there. Travel Δ: its movement from cold to operating case 1 (up positive). The spring is the first in the spring table whose working range takes H and the cold load C = H + k Δ, with the load variation |k Δ| / H within {num(100 * (model()?.settings.variability ?? 0.25), 3)} %. Set the spring to C when installing.</p>
  <table class="tbl"><thead><tr><Th>Node</Th><Th>Tag</Th><Th>Size</Th><Th r>Rate {u("tstiff")}</Th><Th r>Hot load {u("force")}</Th><Th r>Travel {u("disp")}</Th><Th r>Cold load {u("force")}</Th><Th r>Variation</Th><Th r>Range {u("force")}</Th></tr></thead><tbody>
    <For each={p.r.hangers}>{h => <tr class="f" onClick={() => jump(h.node)}><td>{h.node}</td><td>{model()?.supports.find(s => s.id === h.support)?.tag}</td><td>{h.size === "constant" ? "constant-force" : h.size}</td><td class="r">{h.k ? v("tstiff", h.k) : "—"}</td><td class="r">{v("force", h.hot)}</td><td class="r">{v("disp", h.travel)}</td><td class="r">{v("force", h.cold)}</td><td class="r">{h.k ? `${num(100 * h.variability, 3)} %` : "—"}</td><td class="r">{h.k ? `${v("force", h.range[0])}–${v("force", h.range[1])}` : "—"}</td></tr>}</For></tbody></table>
</Show>;

export const Walls = (p: { r: Results }) => {
  const el = (id: number) => { const e = model()?.elements.find(x => x.id === id); return e ? `${e.from}–${e.to}` : ""; };
  return <Show when={p.r.walls.length} fallback={<p class="muted">No element has an internal pressure.</p>}>
    <p class="muted small">Pressure design thickness t = P D / (2 (S E W / I + P Y)), plus the corrosion allowance, divided by (1 − mill tolerance), against the nominal wall; the hoop stress on the wall that is left. I = 1 on straight pipe; on a bend, the factor at its intrados, (4R/D − 1) / (4R/D − 2). A reducer is checked at both ends. A thickness of D/6 or more fails: the thin-wall equation does not apply.</p>
    <table class="tbl"><thead><tr><Th>Element</Th><Th>At</Th><Th r>I</Th><Th>Case</Th><Th r>P {u("press")}</Th><Th r>T {u("temp")}</Th><Th r>S {u("stress")}</Th><Th r>t {u("len")}</Th><Th r>tm</Th><Th r>Nominal needed</Th><Th r>Nominal</Th><Th r>Hoop {u("stress")}</Th><Th r>Ratio</Th><Th>Result</Th></tr></thead><tbody>
      <For each={p.r.walls}>{w => <tr><td>{el(w.element)}</td><td>{w.at}</td><td class="r">{num(w.i, 3)}</td><td>{w.case}</td><td class="r">{v("press", w.p)}</td><td class="r">{v("temp", w.t)}</td><td class="r">{v("stress", w.s)}</td><td class="r">{v("len", w.t_design)}</td><td class="r">{v("len", w.t_min)}</td><td class="r">{v("len", w.t_nominal_needed)}</td><td class="r">{v("len", w.t_nominal)}</td><td class="r">{v("stress", w.hoop)}</td><td class="r">{num(w.ratio, 3)}</td><td><Badge pass={w.pass} />{w.thick ? " t ≥ D/6" : ""}</td></tr>}</For></tbody></table>
  </Show>;
};

export const Equilibrium = (p: { r: Results }) => (
  <>
    <p class="muted small">For every case, the applied loads (weight, extra loads) and the support reactions are added up independently: they must balance. Moments are about the origin.</p>
    <table class="tbl"><thead><tr><Th>Case</Th><Th r>Σ FX loads {u("force")}</Th><Th r>Σ FX reactions</Th><Th r>Σ FY loads</Th><Th r>Σ FY reactions</Th><Th r>Σ FZ loads</Th><Th r>Σ FZ reactions</Th><Th r>Σ M loads {u("moment")}</Th><Th r>Σ M reactions</Th><Th r>Imbalance</Th></tr></thead><tbody>
      <For each={p.r.cases}>{c => <tr><td>{c.name}</td><For each={[0, 1, 2]}>{k => <><td class="r">{v("force", c.applied[k])}</td><td class="r">{v("force", c.reacted[k])}</td></>}</For>
        <td class="r">{v("moment", Math.hypot(c.applied[3], c.applied[4], c.applied[5]))}</td><td class="r">{v("moment", Math.hypot(c.reacted[3], c.reacted[4], c.reacted[5]))}</td>
        <td class="r" classList={{ bad: c.imbalance > 1e-6 }}>{c.imbalance < 1e-12 ? "< 1e-12" : c.imbalance.toExponential(1)} {c.imbalance <= 1e-6 ? "✓" : "✗"}</td></tr>}</For></tbody></table>
    <p class="muted small">Solver: {p.r.stats.equations} equations, {p.r.stats.nonzeros} stored entries, residual {p.r.stats.residual.toExponential(1)}, {num(p.r.stats.ms, 3)} ms. Datum Pipe {p.r.version}.</p>
  </>
);

// ---------- the report's results pages ----------
import { setResultBlocks } from "./report";
setResultBlocks(m => {
  const r = results();
  if (!r || stale()) return [() => <p class="muted">Analyze the model (F5) to add the stress summary, support loads, nozzle loads and the equilibrium checks to the report.{r && stale() ? " (The model changed since the last analysis.)" : ""}</p>];
  const exp = r.cases.find(c => c.kind === "exp") ?? r.cases[0];
  const sus = r.cases.find(c => c.kind === "sus");
  const ope = r.cases.find(c => c.kind === "ope") ?? r.cases[0];
  const blocks = [
    () => <div><h3>3. Stress summary</h3><Summary r={r} />
      <p class="muted small">Calculated using the equations of the process piping code: sustained stress SL against Sh, displacement stress range SE against SA = f (1.25 Sc + 0.25 Sh){m.settings.liberal ? " with the liberal allowance" : ""}, f = {num(Math.min(1, 6 * Math.pow(Math.max(1, m.settings.cycles), -0.2)), 3)} for {m.settings.cycles} cycles.</p></div>,
    ...(sus ? [() => <div><h4>Highest sustained stresses ({sus.name})</h4><TopStresses r={r} c={sus} /></div>] : []),
    ...(exp && exp.stresses.length ? [() => <div><h4>Highest expansion stress ranges ({exp.name})</h4><TopStresses r={r} c={exp} /></div>] : []),
    () => <div><h3>4. Support loads ({ope.name})</h3><Supports r={r} c={ope} /></div>,
    ...(r.hangers?.length ? [() => <div><h4>Spring hangers</h4><Hangers r={r} /></div>] : []),
    ...r.cases.filter(c => c.kind !== "ope" && c.id !== ope.id && c.kind !== "exp").map(c => () => <div><h4>Support loads ({c.name})</h4><Supports r={r} c={c} /></div>),
    ...(m.nozzles.length ? r.cases.filter(c => c.kind !== "exp").map(c => () => <div><h3>{c === r.cases.find(x => x.kind !== "exp") ? "5. Nozzle loads" : ""}</h3><h4>{c.name}</h4><Nozzles r={r} c={c} /></div>) : []),
    ...(r.walls.length ? [() => <div><h3>6. Minimum wall</h3><Walls r={r} /></div>] : []),
    () => <div><h3>7. Equilibrium checks</h3><Equilibrium r={r} /></div>,
  ];
  return blocks;
});

/** The ten highest stress ratios of a case. */
const TopStresses = (p: { r: Results; c: CaseResult }) => {
  const top = [...p.c.stresses].sort((a, b) => b.ratio - a.ratio).slice(0, 10);
  return <table class="tbl"><thead><tr><Th>Point</Th><Th r>Stress {u("stress")}</Th><Th r>Allowable</Th><Th r>Ratio</Th><Th r>SIF in / out</Th><Th>Result</Th></tr></thead><tbody>
    <For each={top}>{s => <tr><td>{p.r.geom.points[s.point]?.label}</td><td class="r">{v("stress", s.stress)}</td><td class="r">{v("stress", s.allow)}</td><td class="r">{num(s.ratio, 3)}</td><td class="r">{num(s.ii, 3)} / {num(s.io, 3)}</td><td><Badge pass={s.pass} /></td></tr>}</For></tbody></table>;
};
