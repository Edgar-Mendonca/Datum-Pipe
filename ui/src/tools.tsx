// Edit tools: the expansion loop wizard and copy or mirror a section of the table.
import { For, createSignal } from "solid-js";
import { num, parseNum } from "./api";
import { openDialog, act, model, sel, setSel, toast } from "./store";
import { Dialog } from "./ui";
import { toU, fromU, unit } from "./units";

const selected = () => { const s = sel(), m = model(); return s?.kind === "element" && m ? m.elements.find(e => e.id === s.id) : undefined; };

/** Model › Expansion Loop: the selected straight element becomes a loop in its middle. */
export const loopWizard = () => {
  const e = selected();
  if (!e) return toast("Select the straight element to put the loop in (a row of the table, or the pipe in the 3D view).");
  const m = model()!, len = Math.hypot(...e.d);
  const horizontal = Math.abs(e.d[1]) < 1e-9 * len;
  const [w, setW] = createSignal(Math.min(len / 3, 4)), [h, setH] = createSignal(Math.min(len / 3, 4)), [up, setUp] = createSignal(!horizontal);
  openDialog(() => {
    const field = (l: string, v: () => number, set: (x: number) => void) => <><label>{l}<span class="unit"> {unit("coord")}</span></label>
      <input class="mono" value={num(toU("coord", v()), 5)} onChange={ev => { const x = parseNum(ev.currentTarget.value); if (x && x > 0) set(fromU("coord", x)); else ev.currentTarget.value = num(toU("coord", v()), 5); }} /></>;
    // A small picture of the loop, to scale.
    const pic = () => { const W = 260, H = 110, s = Math.min((W - 20) / len, (H - 20) / Math.max(h(), 1e-6)); const a = (len - w()) / 2;
      const x0 = (W - len * s) / 2, y0 = H - 12; const pts = [[0, 0], [a, 0], [a, h()], [a + w(), h()], [a + w(), 0], [len, 0]];
      return <svg width={W} height={H} class="looppic"><path d={pts.map((p, i) => `${i ? "L" : "M"}${x0 + p[0] * s} ${y0 - p[1] * s}`).join("")} fill="none" stroke="var(--blue)" stroke-width="3" stroke-linejoin="round" /><text x={4} y={14} font-size="10" fill="var(--muted)">{e.from} → {e.to}</text></svg>; };
    return (
      <Dialog title="Expansion loop" okText="Make the loop" help="edit" ok={async () => { const o = await act({ Loop: { id: e.id, width: w(), height: h(), up: up() } }); if (!o) return false; toast("Loop made: four long-radius bends. Analyze (F5) to see what it takes up.", true); setSel({ kind: "element", id: e.id }); }}>
        <p class="muted">Element {e.from}–{e.to} ({num(toU("coord", len), 5)} {unit("coord")}) becomes a loop in its middle, with long-radius bends at its four corners. Node numbers are added between {e.from} and {e.to}.</p>
        <div class="form2">{field("Width (along the pipe)", w, setW)}{field("Height (out from the pipe)", h, setH)}</div>
        <p><label class="chk"><input type="radio" name="lp" checked={!up()} disabled={!horizontal} onChange={() => setUp(false)} />Sideways, in the horizontal plane</label>
          <label class="chk"><input type="radio" name="lp" checked={up()} onChange={() => setUp(true)} />Upwards</label></p>
        {pic()}
        <p class="muted small">{m.materials.find(x => x.id === e.material)?.name}; the loop keeps the element's pipe, material and line conditions.</p>
      </Dialog>
    );
  });
};

/** Model › Copy Section: copy rows of the table, optionally mirrored, onto a node. */
export const copySection = (mirrorFirst = false) => {
  const m = model(); if (!m || !m.elements.length) return toast("There is nothing to copy yet.");
  const cur = selected();
  const idx = cur ? m.elements.findIndex(e => e.id === cur.id) : 0;
  const [from, setFrom] = createSignal(idx + 1), [to, setTo] = createSignal(m.elements.length), [at, setAt] = createSignal(m.elements[m.elements.length - 1].to), [mir, setMir] = createSignal(mirrorFirst ? 0 : -1);
  openDialog(() => (
    <Dialog title={mirrorFirst ? "Mirror a section" : "Copy a section"} okText="Copy" help="edit" ok={async () => {
      const ids = m.elements.slice(Math.max(0, from() - 1), Math.max(from(), to())).map(e => e.id);
      const o = await act({ Duplicate: { ids, at: at(), mirror: mir() >= 0 ? mir() : null } });
      if (!o) return false;
      toast(`Copied ${ids.length} elements onto node ${at()}.`, true);
    }}>
      <p class="muted">Copies table rows {from()} to {to()} with their components and supports, and joins the copy's first node to the node you choose. New nodes are numbered after the largest.</p>
      <div class="form2">
        <label>From row</label><input class="mono" value={from()} onChange={e => setFrom(Math.max(1, Math.min(m.elements.length, parseNum(e.currentTarget.value) ?? 1)))} />
        <label>To row</label><input class="mono" value={to()} onChange={e => setTo(Math.max(1, Math.min(m.elements.length, parseNum(e.currentTarget.value) ?? 1)))} />
        <label>Join the copy to node</label><select onChange={e => setAt(+e.currentTarget.value)}><For each={m.nodes}>{n => <option value={n.num} selected={n.num === at()}>{n.num}</option>}</For></select>
        <label>Mirror</label><select onChange={e => setMir(+e.currentTarget.value)}><For each={[[-1, "No, copy as it is"], [0, "Across X (east becomes west)"], [1, "Across Y (up becomes down)"], [2, "Across Z (south becomes north)"]] as [number, string][]}>{([v, l]) => <option value={v} selected={v === mir()}>{l}</option>}</For></select>
      </div>
    </Dialog>
  ));
};
