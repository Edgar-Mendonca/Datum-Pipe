// The element table: the piping as rows (From, To, DX, DY, DZ, pipe, material, line conditions,
// component). Typing in the last row adds an element that starts at the node above; every cell
// sends one edit to the engine. The selected row follows the 3D view and the other way round.
import { For, Show, createSignal, createMemo } from "solid-js";
import { Element, Comp, Model, num, parseNum } from "./api";
import { model, act, sel, setSel, toast } from "./store";
import { toU, fromU, unit } from "./units";
import { Help } from "./help";

/** The component choices in the table, as "kind" or "kind:variant". */
export const COMPS: [string, string][] = [
  ["pipe", "Pipe"], ["bend:long", "Bend, long radius (1.5D)"], ["bend:short", "Bend, short radius (1D)"], ["bend:custom", "Bend, own radius"],
  ["tee:welding", "Welding tee at end"], ["tee:fabricated", "Fabricated tee at end"], ["tee:reinforced", "Reinforced tee at end"],
  ["reducer", "Reducer"], ["rigid:valve", "Valve"], ["rigid:flange", "Flange pair, weld neck"], ["rigid:slipon", "Flange pair, slip-on"], ["rigid:rigid", "Rigid part"],
];
export const compKey = (c: Comp) => c.kind === "bend" ? `bend:${c.bend}` : c.kind === "tee" ? `tee:${c.tee}` : c.kind === "rigid" ? `rigid:${c.rigid}` : c.kind;
export const compName = (c: Comp) => COMPS.find(x => x[0] === compKey(c))?.[1] ?? "Pipe";

/** A component from a table choice, keeping the values it already had. */
export const compFrom = (key: string, e: Element, m: Model): Comp => {
  const [k, v] = key.split(":");
  const old = e.comp as any;
  if (k === "bend") return { kind: "bend", bend: v as any, r: old.kind === "bend" && old.r > 0 ? old.r : 1.5 * (m.pipes.find(p => p.id === e.pipe)?.od ?? 0.17) };
  if (k === "tee") return { kind: "tee", tee: v as any, pad: old.kind === "tee" ? old.pad : (m.pipes.find(p => p.id === e.pipe)?.wall ?? 0.007) };
  if (k === "reducer") { const p = m.pipes.find(p => p.id === e.pipe); const smaller = m.pipes.filter(q => p && q.od < p.od).sort((a, b) => b.od - a.od)[0]; return { kind: "reducer", to: old.kind === "reducer" ? old.to : (smaller ?? p ?? m.pipes[0]).id }; }
  if (k === "rigid") return { kind: "rigid", rigid: v as any, mass: old.kind === "rigid" ? old.mass : 0 };
  return { kind: "pipe" };
};

const put = (e: Element, x: Partial<Element>) => act({ Set: { item: { Element: { ...e, ...x } } } });
const len = (d: number[]) => Math.hypot(d[0], d[1], d[2]);
const shownC = (v: number) => num(toU("coord", v), 6).replace("−", "-");

/** Moves the focus to the same column of the row above or below. */
const move = (el: HTMLElement, by: number) => {
  const td = el.closest("td"), tr = el.closest("tr"); if (!td || !tr) return;
  const col = [...tr.children].indexOf(td), rows = [...tr.parentElement!.children];
  const next = rows[rows.indexOf(tr) + by]?.children[col]?.querySelector("input,select") as HTMLElement | null;
  next?.focus(); if (next instanceof HTMLInputElement) next.select();
};
const keys = (e: KeyboardEvent) => {
  const t = e.currentTarget as HTMLElement;
  if (e.key === "Enter" && !e.ctrlKey) { e.preventDefault(); (t as HTMLInputElement).blur?.(); setTimeout(() => move(t, e.shiftKey ? -1 : 1)); }
  else if (e.key === "ArrowDown" && t instanceof HTMLInputElement) { e.preventDefault(); move(t, 1); }
  else if (e.key === "ArrowUp" && t instanceof HTMLInputElement) { e.preventDefault(); move(t, -1); }
};

export const [tableH, setTableH] = createSignal(230);

export const ElementTable = () => {
  const m = () => model()!;
  const step = () => m().settings.step || 10;
  const last = () => m().elements[m().elements.length - 1];
  const [newFrom, setNewFrom] = createSignal<number | null>(null);
  const from = () => newFrom() ?? last()?.to ?? 10;
  const nextNum = (f: number) => { const used = new Set(m().nodes.map(n => n.num)); let n = Math.ceil((f + 1) / step()) * step(); while (used.has(n)) n += step(); return n; };
  const rowOn = (e: Element) => { const s = sel(); return !!s && ((s.kind === "element" && s.id === e.id) || (s.kind === "node" && (e.from === s.id || e.to === s.id))); };
  const add = async (axis: number, v: number) => {
    if (!v) return;
    const d: [number, number, number] = [0, 0, 0]; d[axis] = fromU("coord", v);
    const base = m().elements.find(e => e.to === from()) ?? last();
    const el = { id: 0, from: from(), to: nextNum(from()), d, pipe: base?.pipe ?? m().pipes[0].id, material: base?.material ?? m().materials[0].id, cond: base?.cond ?? m().conds[0].id, comp: { kind: "pipe" }, extra: 0 };
    const o = await act({ Add: { item: { Element: el }, after: null } });
    if (o?.created.length) { setNewFrom(null); setSel({ kind: "element", id: o.created[0] }); }
  };
  const total = createMemo(() => m().elements.reduce((s, e) => s + len(e.d), 0));
  let drag = 0;
  const resize = (e: PointerEvent) => {
    const y0 = e.clientY, h0 = tableH(); drag = 1;
    const mv = (ev: PointerEvent) => setTableH(Math.max(90, Math.min(innerHeight - 260, h0 - (ev.clientY - y0))));
    const up = () => { drag = 0; removeEventListener("pointermove", mv); removeEventListener("pointerup", up); };
    addEventListener("pointermove", mv); addEventListener("pointerup", up);
  };
  return (
    <section class="etable" style={{ height: `${tableH()}px` }}>
      <div class="ehandle" onPointerDown={resize} title="Drag to resize" />
      <header><b>Elements</b><Help k="table" /><span class="muted">{m().elements.length} elements · {m().nodes.length} nodes · {num(toU("coord", total()), 5)} {unit("coord")} of pipe</span><span class="grow" />
        <span class="muted small">Lengths in {unit("coord")} · Y is up</span></header>
      <div class="escroll">
        <table class="egrid">
          <thead><tr><th class="rn">#</th><th>From</th><th>To</th><th>DX</th><th>DY</th><th>DZ</th><th>Length</th><th>Pipe</th><th>Material</th><th>Conditions</th><th>Component</th></tr></thead>
          <tbody>
            <For each={m().elements}>{(e, i) => (
              <tr classList={{ on: rowOn(e) }} onPointerDown={() => drag || setSel({ kind: "element", id: e.id })}>
                <td class="rn">{i() + 1}</td>
                <td><input class="mono n" value={e.from} onKeyDown={keys} onChange={ev => { const v = parseNum(ev.currentTarget.value); if (v === null || v < 1 || v % 1) { ev.currentTarget.value = String(e.from); return toast("Node numbers are whole numbers from 1."); } put(e, { from: v }); }} /></td>
                <td><input class="mono n" value={e.to} onKeyDown={keys} onChange={ev => { const v = parseNum(ev.currentTarget.value); if (v === null || v < 1 || v % 1) { ev.currentTarget.value = String(e.to); return toast("Node numbers are whole numbers from 1."); } put(e, { to: v }); }} /></td>
                <For each={[0, 1, 2]}>{k => <td><input class="mono" value={e.d[k] ? shownC(e.d[k]) : ""} placeholder="0" onKeyDown={keys}
                  onChange={ev => { const t = ev.currentTarget.value.trim(); const v = t === "" ? 0 : parseNum(t); if (v === null) { ev.currentTarget.value = e.d[k] ? shownC(e.d[k]) : ""; return; } const d = [...e.d] as [number, number, number]; d[k] = fromU("coord", v); put(e, { d }); }} /></td>}</For>
                <td class="r mono muted">{num(toU("coord", len(e.d)), 5)}</td>
                <td><select onKeyDown={keys} onChange={ev => put(e, { pipe: +ev.currentTarget.value })}><For each={m().pipes}>{p => <option value={p.id} selected={p.id === e.pipe}>{p.name}</option>}</For></select></td>
                <td><select onKeyDown={keys} onChange={ev => put(e, { material: +ev.currentTarget.value })}><For each={m().materials}>{p => <option value={p.id} selected={p.id === e.material}>{p.name}</option>}</For></select></td>
                <td><select onKeyDown={keys} onChange={ev => put(e, { cond: +ev.currentTarget.value })}><For each={m().conds}>{p => <option value={p.id} selected={p.id === e.cond}>{p.name}</option>}</For></select></td>
                <td><select onKeyDown={keys} onChange={ev => put(e, { comp: compFrom(ev.currentTarget.value, e, m()) })}><For each={COMPS}>{c => <option value={c[0]} selected={c[0] === compKey(e.comp)}>{c[1]}</option>}</For></select></td>
              </tr>
            )}</For>
            <tr class="newrow">
              <td class="rn">+</td>
              <td><input class="mono n" value={from()} title="The new element starts here" onKeyDown={keys} onChange={ev => { const v = parseNum(ev.currentTarget.value); setNewFrom(v && v >= 1 && v % 1 === 0 ? v : null); }} /></td>
              <td class="mono muted">{nextNum(from())}</td>
              <For each={[0, 1, 2]}>{k => <td><input class="mono" placeholder={["DX", "DY", "DZ"][k]} onKeyDown={keys} onChange={ev => { const v = parseNum(ev.currentTarget.value); ev.currentTarget.value = ""; if (v) add(k, v); }} /></td>}</For>
              <td colSpan={5} class="muted small">Type a length in DX, DY or DZ to add an element from node {from()}.</td>
            </tr>
          </tbody>
        </table>
        <Show when={!m().elements.length}><div class="etip"><b>Start here.</b> Type the first element's length in DX (east), DY (up) or DZ (south) in the row above. Node 10 is at the origin.</div></Show>
      </div>
    </section>
  );
};
