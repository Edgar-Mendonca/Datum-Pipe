// The bottom panel's other tabs: Messages (everything the window said, kept with its time) and the
// solver's log (0.6.1); the Nodes, Supports and Loads tables and Paste from Excel (0.6.3). The
// element table is its first tab (table.tsx).
import { For, Show, createSignal } from "solid-js";
import { log, setLog, ptab, setPtab, unread, Note, model, act, sel, setSel, toast, note, openDialog, closeDialog } from "./store";
import { solverLog, setSolverLog } from "./rstate";
import { Load, Support, num, parseNum } from "./api";
import { toU, fromU, unit, parseQ } from "./units";
import { SUPPORTS } from "./cmds";
import { Dialog } from "./ui";

const KIND: Record<Note["kind"], string> = { note: "Note", ok: "Done", warn: "Check", edit: "Edit", solve: "Solve" };
const hhmm = (d: Date) => d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });

/** The panel's tabs. */
export const PanelTabs = () => (
  <nav class="ptabs">
    <For each={[["elements", "Elements"], ["nodes", "Nodes"], ["supports", "Supports"], ["loads", "Loads"], ["messages", "Messages"], ["log", "Solver log"]]}>{([k, l]) =>
      <button classList={{ on: ptab() === k }} onClick={() => setPtab(k)}>{l}<Show when={k === "messages" && unread() > 0}><i class="badge">{unread()}</i></Show></button>}</For>
  </nav>
);

export const Messages = () => (
  <div class="pbody msgs">
    <Show when={log().length} fallback={<p class="muted pad">Nothing yet. Every edit, solve, warning and note the window gives is kept here with its time.</p>}>
      <div class="pbar"><span class="muted">{log().length} messages, newest last</span><span class="grow" /><button class="btn sm" onClick={() => setLog([])}>Clear</button></div>
      <For each={log()}>{m => <div class={`msg ${m.kind}`}><time>{hhmm(m.t)}</time><b>{KIND[m.kind]}</b><span>{m.text}</span></div>}</For>
    </Show>
  </div>
);

export const SolverLog = () => (
  <div class="pbody slog">
    <Show when={solverLog().length} fallback={<p class="muted pad">No solve yet. Analyze (F5): each solve adds a line with the cases, the equations and the time it took.</p>}>
      <div class="pbar"><span class="muted">One line per solve</span><span class="grow" /><button class="btn sm" onClick={() => setSolverLog([])}>Clear</button></div>
      <pre>{solverLog().join("\n")}</pre>
    </Show>
  </div>
);

/** A number cell that edits a value in display units (an empty cell is 0). */
const QCell = (p: { q: Parameters<typeof toU>[0]; v: number; set: (si: number) => void }) =>
  <td><input class="mono" value={p.v ? num(toU(p.q, p.v), 6) : ""} placeholder="0" onChange={e => { const t = e.currentTarget.value.trim(); const q = t === "" ? 0 : parseQ(t, p.q); if (typeof q === "string") { e.currentTarget.value = p.v ? num(toU(p.q, p.v), 6) : ""; return toast(q); } p.set(fromU(p.q, q)); }} /></td>;

/** Nodes: every node's position (from the elements), what is at it, a click selects it. */
export const NodesTable = () => {
  // The model's own node positions: the corner of a bend, not the point the analysis puts on its arc
  // (found in the real app: nodes 50 and 60 showed the bends' mid points).
  const pts = () => [...(model()?.nodes ?? [])].sort((a, b) => a.num - b.num).map(n => ({ node: n.num, p: [n.x, n.y, n.z] }));
  const at = (n: number) => [...(model()?.supports ?? []).filter(s => s.node === n).map(s => s.tag || SUPPORTS.find(x => x[0] === s.kind.kind)?.[1] || ""), ...(model()?.nozzles ?? []).filter(z => z.node === n).map(z => z.name), ...(model()?.loads ?? []).filter(l => l.node === n).map(() => "load")].join(", ");
  return (
    <div class="pbody">
      <Show when={pts().length} fallback={<p class="muted pad">No nodes yet: type the first element in the Elements tab.</p>}>
        <table class="egrid ptable"><thead><tr><th>Node</th><th class="r">X ({unit("coord")})</th><th class="r">Y</th><th class="r">Z</th><th>At the node</th></tr></thead>
          <tbody><For each={pts()}>{p => <tr classList={{ on: sel()?.kind === "node" && sel()!.id === p.node }} onClick={() => setSel({ kind: "node", id: p.node })}><td class="r mono">{p.node}</td><For each={[0, 1, 2]}>{k => <td class="r mono">{num(toU("coord", p.p[k]), 6)}</td>}</For><td>{at(p.node)}</td></tr>}</For></tbody></table>
        <p class="muted small pad">Positions follow from the elements' lengths, from the first node. X east, Y up, Z south.</p>
      </Show>
    </div>
  );
};

/** Supports: tag, kind, node, gap, friction and stiffness, typed into in place. */
export const SupportsTable = () => {
  const put = (s: Support, x: Partial<Support>) => act({ Set: { item: { Support: { ...s, ...x } } } });
  return (
    <div class="pbody">
      <Show when={model()?.supports.length} fallback={<p class="muted pad">No supports yet: select a node and add one from the Supports tab of the ribbon.</p>}>
        <table class="egrid ptable"><thead><tr><th>Tag</th><th>Kind</th><th>Node</th><th class="r">Gap ({unit("disp")})</th><th class="r">Friction μ</th><th class="r">Stiffness ({unit("tstiff")}, 0 = rigid)</th></tr></thead>
          <tbody><For each={model()!.supports}>{s => <tr classList={{ on: sel()?.kind === "support" && sel()!.id === s.id }} onClick={() => setSel({ kind: "support", id: s.id })}>
            <td><input value={s.tag} onChange={e => put(s, { tag: e.currentTarget.value })} /></td>
            <td class="muted">{SUPPORTS.find(x => x[0] === s.kind.kind)?.[1] ?? s.kind.kind}</td>
            <td><input class="mono n" value={s.node} onChange={e => { const v = parseNum(e.currentTarget.value); if (v === null || v < 1 || v % 1) { e.currentTarget.value = String(s.node); return toast("A node number is a whole number from 1."); } put(s, { node: v }); }} /></td>
            <QCell q="disp" v={s.gap} set={gap => gap >= 0 ? put(s, { gap }) : toast("The gap must be 0 or more.")} />
            <td><input class="mono" value={s.mu ? String(s.mu) : ""} placeholder="0" onChange={e => { const v = parseNum(e.currentTarget.value || "0"); if (v === null || v < 0) { e.currentTarget.value = s.mu ? String(s.mu) : ""; return toast("μ must be 0 or more."); } put(s, { mu: v }); }} /></td>
            <QCell q="tstiff" v={s.k} set={k => k >= 0 ? put(s, { k }) : toast("The stiffness must be 0 or more.")} />
          </tr>}</For></tbody></table>
      </Show>
    </div>
  );
};

/** Point loads: node and the force and moment, typed into in place. */
export const LoadsTable = () => {
  const put = (l: Load, x: Partial<Load>) => act({ Set: { item: { Load: { ...l, ...x } } } });
  return (
    <div class="pbody">
      <Show when={model()?.loads.length} fallback={<p class="muted pad">No point loads. Select a node and add one from the Loads tab of the ribbon.</p>}>
        <table class="egrid ptable"><thead><tr><th>Node</th><th class="r">FX ({unit("force")})</th><th class="r">FY</th><th class="r">FZ</th><th class="r">MX ({unit("moment")})</th><th class="r">MY</th><th class="r">MZ</th></tr></thead>
          <tbody><For each={model()!.loads}>{l => <tr classList={{ on: sel()?.kind === "load" && sel()!.id === l.id }} onClick={() => setSel({ kind: "load", id: l.id })}>
            <td><input class="mono n" value={l.node} onChange={e => { const v = parseNum(e.currentTarget.value); if (v === null || v < 1 || v % 1) { e.currentTarget.value = String(l.node); return toast("A node number is a whole number from 1."); } put(l, { node: v }); }} /></td>
            <For each={[0, 1, 2]}>{k => <QCell q="force" v={l.f[k]} set={v => put(l, { f: l.f.map((x, i) => i === k ? v : x) as Load["f"] })} />}</For>
            <For each={[0, 1, 2]}>{k => <QCell q="moment" v={l.m[k]} set={v => put(l, { m: l.m.map((x, i) => i === k ? v : x) as Load["m"] })} />}</For>
          </tr>}</For></tbody></table>
      </Show>
    </div>
  );
};

/** Rows pasted from a spreadsheet (0.6.3): DX DY DZ, or From To DX DY DZ, in the shown length unit,
 *  separated by tabs, commas, semicolons or spaces. Headings and rows not understood are left out
 *  and said in Messages. The new elements continue from the last one, with its pipe, material and
 *  line conditions. */
export const parseRows = (text: string): { rows: { from?: number; to?: number; d: [number, number, number] }[]; skipped: number } => {
  const rows: { from?: number; to?: number; d: [number, number, number] }[] = [];
  let skipped = 0;
  for (const line of text.split(/\r?\n/)) {
    if (!line.trim()) continue;
    const cells = line.trim().split(/[\t;,]+|\s{1,}/).filter(Boolean).map(c => parseNum(c));
    if (cells.some(c => c === null) || (cells.length !== 3 && cells.length !== 5)) { skipped++; continue; }
    const v = cells as number[];
    const d = (v.length === 3 ? v : v.slice(2)).map(x => fromU("coord", x)) as [number, number, number];
    if (d.every(x => x === 0)) { skipped++; continue; }
    rows.push(v.length === 5 ? { from: v[0], to: v[1], d } : { d });
  }
  return { rows, skipped };
};

export const pasteDialog = () => openDialog(() => {
  const [text, setText] = createSignal("");
  const parsed = () => parseRows(text());
  const go = async () => {
    const m = model(); if (!m) return;
    const { rows, skipped } = parsed();
    if (!rows.length) { toast("Nothing to add: paste rows of DX, DY, DZ (or From, To, DX, DY, DZ)."); return false; }
    const step = m.settings.step || 10;
    const used = new Set(m.nodes.map(n => n.num));
    const last = m.elements[m.elements.length - 1];
    let from = last?.to ?? 10;
    const next = (f: number) => { let n = Math.ceil((f + 1) / step) * step; while (used.has(n)) n += step; used.add(n); return n; };
    const ops = rows.map(r => {
      const a = r.from ?? from, b = r.to ?? next(a);
      used.add(a); used.add(b); from = b;
      return { Add: { item: { Element: { id: 0, from: a, to: b, d: r.d, pipe: last?.pipe ?? m.pipes[0].id, material: last?.material ?? m.materials[0].id, cond: last?.cond ?? m.conds[0].id, comp: { kind: "pipe" }, extra: 0 } }, after: null } };
    });
    const o = await act({ Batch: ops });
    if (o) { note(`Pasted ${rows.length} element${rows.length > 1 ? "s" : ""}${skipped ? `; ${skipped} row${skipped > 1 ? "s" : ""} not understood were left out` : ""}.`, skipped ? "warn" : "ok"); closeDialog(); }
    return false;
  };
  return (
    <Dialog title="Paste from Excel" ok={go} okText="Add the elements" help="table">
      <p class="muted">Copy rows from a spreadsheet and paste them here: DX, DY, DZ in {unit("coord")} (each row an element continuing from the last), or From, To, DX, DY, DZ. Headings and rows that are not numbers are left out.</p>
      <textarea class="mono paste" rows={10} placeholder={"6000\t0\t0\n0\t0\t-3000"} value={text()} onInput={e => setText(e.currentTarget.value)} />
      <p class="muted small">{parsed().rows.length} element{parsed().rows.length === 1 ? "" : "s"} to add{parsed().skipped ? `, ${parsed().skipped} row${parsed().skipped > 1 ? "s" : ""} left out` : ""}.</p>
    </Dialog>
  );
});
