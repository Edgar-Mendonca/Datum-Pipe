// The model tree (0.6.2, after Datum Mesh): everything in the model in one list with counts, a
// filter, and Model and Results tabs. A click selects an item (its card opens over the view) or
// opens the dialog that edits that kind of item.
import { For, Show, createSignal, JSX } from "solid-js";
import { model, sel, setSel, setView, setTab } from "./store";
import { results, stale, setCaseId } from "./rstate";
import { Icon } from "./ui";
import { Help } from "./help";
import { num } from "./api";
import * as P from "./props";
import { SUPPORTS } from "./cmds";

export const [treeTab, setTreeTab] = createSignal<"model" | "results">("model");
/** The card over the view: open for the item selected, or for the model's own settings. */
export const [cardOpen, setCardOpen] = createSignal(true);
export const openCard = () => setCardOpen(true);
const [filter, setFilter] = createSignal("");
const [open, setOpen] = createSignal<Record<string, boolean>>({ elements: false, supports: true });

type Row = { label: string; on?: boolean; go: () => void };
/** A group: its icon, name and count; rows under it when it is open (or when the filter matches). */
const Group = (p: { k: string; i: string; l: string; n: number | string; rows?: Row[]; go?: () => void }) => {
  const f = () => filter().trim().toLowerCase();
  const rows = () => (p.rows ?? []).filter(r => !f() || r.label.toLowerCase().includes(f()));
  const shown = () => p.rows && (f() ? rows().length > 0 : open()[p.k]);
  const visible = () => !f() || p.l.toLowerCase().includes(f()) || rows().length > 0;
  return (
    <Show when={visible()}>
      <button class="ti" onClick={() => { if (p.rows) setOpen(o => ({ ...o, [p.k]: !o[p.k] })); else p.go?.(); }} title={p.rows ? `Show or hide the ${p.l.toLowerCase()}` : `Open ${p.l}`}>
        <span class="tw">{p.rows ? (shown() ? "▾" : "▸") : ""}</span><Icon n={p.i} /><b>{p.l}</b><em>{p.n}</em>
      </button>
      <Show when={shown()}><For each={rows()}>{r => <button class="ti k" classList={{ sel: !!r.on }} onClick={r.go}><b>{r.label}</b></button>}</For></Show>
    </Show>
  );
};
const Head = (p: { children: JSX.Element }) => <div class="th">{p.children}</div>;

const supName = (k: string) => SUPPORTS.find(s => s[0] === k)?.[1] ?? k;
const pick = (kind: "element" | "support" | "nozzle" | "load", id: number) => { setSel({ kind, id }); setView("model"); openCard(); };

const ModelTree = () => {
  const m = () => model()!;
  const s = () => sel();
  return (
    <div class="tlist">
      <button class="troot" onClick={() => { setSel(null); setView("model"); openCard(); }} title="The model's settings: code, units, analysis, occasional loads, dynamics"><Icon n="pipe" /><b>{m().name || "Untitled"}</b></button>
      <Head>Route</Head>
      <Group k="elements" i="addel" l="Elements" n={m().elements.length} rows={m().elements.map(e => ({ label: `${e.from} → ${e.to} · ${e.comp.kind}${e.soil ? " · buried" : ""}`, on: s()?.kind === "element" && s()!.id === e.id, go: () => pick("element", e.id) }))} />
      <Group k="nodes" i="dist" l="Nodes" n={m().nodes.length} go={() => { setTab("home"); setView("model"); }} />
      <Head>Properties</Head>
      <Group k="pipes" i="pipe" l="Pipes" n={m().pipes.length} go={() => P.pipes()} />
      <Group k="materials" i="material" l="Materials" n={m().materials.length} go={() => P.materials()} />
      <Group k="conds" i="temp" l="Line conditions" n={m().conds.length} go={() => P.conds()} />
      <Head>Supports and loads</Head>
      <Group k="supports" i="anchor" l="Supports" n={m().supports.length} rows={m().supports.map(x => ({ label: `${x.tag || supName(x.kind.kind)} · ${supName(x.kind.kind).toLowerCase()} · node ${x.node}`, on: s()?.kind === "support" && s()!.id === x.id, go: () => pick("support", x.id) }))} />
      <Group k="nozzles" i="nozzle" l="Nozzles" n={m().nozzles.length} rows={m().nozzles.map(z => ({ label: `${z.name} · node ${z.node}`, on: s()?.kind === "nozzle" && s()!.id === z.id, go: () => pick("nozzle", z.id) }))} />
      <Group k="equipment" i="opts" l="Equipment" n={(m().equipment ?? []).length} go={() => P.equipmentDialog()} />
      <Group k="loads" i="force" l="Point loads" n={m().loads.length} rows={m().loads.map(l => ({ label: `Node ${l.node}`, on: s()?.kind === "load" && s()!.id === l.id, go: () => pick("load", l.id) }))} />
      <Group k="occ" i="wind" l="Wind and seismic" n={[m().settings.wind_q > 0 || (m().settings.wind_profile ?? []).length ? "wind" : "", m().settings.seis.some(v => v !== 0) ? "seismic" : ""].filter(Boolean).join(", ") || "none"} go={() => { setSel(null); openCard(); }} />
      <Group k="relief" i="relief" l="Relief valves" n={(m().reliefs ?? []).length} go={() => P.reliefDialog()} />
      <Group k="soils" i="soil" l="Soils" n={(m().soils ?? []).length} go={() => P.soilDialog()} />
      <Group k="flanges" i="flange" l="Flange checks" n={(m().flanges ?? []).length} go={() => P.flangeDialog()} />
      <Group k="cases" i="cases" l="Load cases" n={m().cases.length} go={() => P.cases()} />
      <Head>Analysis</Head>
      <Group k="dyn" i="wave" l="Dynamics" n={(m().settings.modes ?? 0) > 0 ? `${m().settings.modes} modes` : "off"} go={() => { setSel(null); openCard(); }} />
      <Group k="vessels" i="vessel" l="Vessels" n={(m().vessels ?? []).length} go={() => setView("vessel")} />
      <Group k="settings" i="opts" l="Settings" n={`${(m().settings.code ?? "b31.3").toUpperCase().replace("EN13480", "EN 13480")} · ${m().units === "US" ? "US" : "SI"}`} go={() => { setSel(null); openCard(); }} />
    </div>
  );
};

/** The Results tab: each case's worst ratio, the checks, the way to the tables (grown in 0.6.5). */
const ResultsTree = () => {
  const r = () => results();
  const worst = (c: NonNullable<ReturnType<typeof results>>["cases"][number]) => c.stresses.reduce((a, s) => s.ratio > a.ratio ? s : a, { ratio: 0, point: -1 } as { ratio: number; point: number });
  return (
    <Show when={r()} fallback={<p class="muted tpad">Analyze (F5): what was solved and the worst result of each case appear here.</p>}>
      <div class="tlist">
        <div class="troot static"><b>Analyzed {r()!.cases.length} load cases</b><small>{r()!.stats.equations} equations · {stale() ? "out of date" : "up to date"}</small></div>
        <Head>Code stresses · worst ratio</Head>
        <For each={r()!.cases.filter(c => c.stresses.length)}>{c => { const w = worst(c); return <button class="ti" onClick={() => { setCaseId(c.id); setView("results"); }}><span class="tw" /><b>{c.name}</b><em classList={{ bad: w.ratio > 1 }}>{num(w.ratio, 3)}{w.point >= 0 ? ` at ${r()!.geom.points[w.point]?.label ?? ""}` : ""}</em></button>; }}</For>
        <Head>Checks</Head>
        <button class="ti" onClick={() => setView("results")}><span class="tw" /><b>Equilibrium</b><em classList={{ ok: r()!.cases.every(c => c.spectrum || c.imbalance <= 1e-6), bad: r()!.cases.some(c => !c.spectrum && c.imbalance > 1e-6) }}>{r()!.cases.every(c => c.spectrum || c.imbalance <= 1e-6) ? "balanced ✓" : "check"}</em></button>
        <Show when={r()!.walls.length}><button class="ti" onClick={() => setView("results")}><span class="tw" /><b>Minimum wall</b><em classList={{ ok: r()!.walls.every(w => w.pass), bad: !r()!.walls.every(w => w.pass) }}>{num(Math.max(...r()!.walls.map(w => w.ratio)), 3)}</em></button></Show>
        <Show when={r()!.modes?.length}><button class="ti" onClick={() => setView("results")}><span class="tw" /><b>Lowest natural frequency</b><em>{num(r()!.modes![0].freq, 3)} Hz</em></button></Show>
        <Head>Tables and report</Head>
        <button class="ti" onClick={() => setView("results")}><span class="tw" /><b>Results tables</b><em /></button>
        <button class="ti" onClick={() => setView("report")}><span class="tw" /><b>Report</b><em /></button>
      </div>
    </Show>
  );
};

export const Tree = () => (
  <aside class="tree">
    <div class="ttabs"><button classList={{ on: treeTab() === "model" }} onClick={() => setTreeTab("model")}>Model</button><button classList={{ on: treeTab() === "results" }} onClick={() => setTreeTab("results")}>Results</button><Help k="window" /></div>
    <Show when={treeTab() === "model"}><div class="tf"><input placeholder="Filter: node, tag, kind…" value={filter()} onInput={e => setFilter(e.currentTarget.value)} aria-label="Filter the tree" /></div></Show>
    <Show when={model()}><Show when={treeTab() === "model"} fallback={<ResultsTree />}><ModelTree /></Show></Show>
  </aside>
);
