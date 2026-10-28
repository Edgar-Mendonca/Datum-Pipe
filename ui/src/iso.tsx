// The isometric drawing (engine: crates/core/src/iso.rs): each sheet drawn as SVG at the size of
// the page, with pipe lines, bends, tees, valves, flanges, reducers, support and nozzle symbols, node
// numbers, a dimension along every element, a north arrow, continuation notes, and optional result
// tags (the highest stress ratio of each element, support loads). It prints with the report and
// saves as SVG.
import { For, Show, createSignal, createResource, createMemo, JSX } from "solid-js";
import { Model, num } from "./api";
import { out, model, call, setBackstage, toast, setView } from "./store";
import { G, S, B } from "./ribbon";
import { Help } from "./help";
import { results, stale, curCase } from "./rstate";
import { fmt, toU, unit } from "./units";
import { setIsoPages } from "./report";
import { TitleBlock, setPs } from "./print";

type P2 = [number, number];
interface Line { element: number; pts: P2[]; kind: string }
interface Dim { element: number; a: P2; b: P2; off: P2; length: number; dir: string }
interface Sym { p: P2; along: P2; kind: string; node: number; tag: string; id: number }
interface Sheet { n: number; lines: Line[]; labels: { p: P2; num: number }[]; dims: Dim[]; syms: Sym[]; cont: [number, number][]; bbox: [number, number, number, number] }
interface Iso { sheets: Sheet[] }

export const [isoOpts, setIsoOpts] = createSignal({ dims: true, labels: true, ratios: true, loads: false, perSheet: 40 });
export const [sheetNo, setSheetNo] = createSignal(1);
const [iso] = createResource(() => [out(), isoOpts().perSheet] as const, ([o, per]) => o ? call<Iso>("isometric", { perSheet: per }) : Promise.resolve(undefined));

/** A length as the drawing writes it: mm in SI, feet and inches (to ⅛") in US. */
export const isoLen = (m: number) => {
  if (model()?.units !== "US") return `${Math.round(m * 1000)}`;
  const inch = m / 0.0254, ft = Math.floor(inch / 12 + 1e-9), rest = Math.round((inch - ft * 12) * 8) / 8;
  const whole = Math.floor(rest), frac = rest - whole;
  const f8 = Math.round(frac * 8), fr = f8 ? ` ${f8 % 2 ? `${f8}/8` : f8 % 4 ? `${f8 / 2}/4` : "1/2"}` : "";
  return `${ft}'-${whole}${fr}"`;
};

/** One sheet drawn to fit w × h pixels. */
export const IsoSheet = (p: { sh: Sheet; w: number; h: number; m: Model; print?: boolean }) => {
  const pad = 70;
  const bw = Math.max(p.sh.bbox[2] - p.sh.bbox[0], 1e-6), bh = Math.max(p.sh.bbox[3] - p.sh.bbox[1], 1e-6);
  const s = Math.min((p.w - 2 * pad) / bw, (p.h - 2 * pad) / bh);
  const ox = (p.w - bw * s) / 2 - p.sh.bbox[0] * s, oy = (p.h - bh * s) / 2 - p.sh.bbox[1] * s;
  const X = (q: P2): P2 => [ox + q[0] * s, oy + q[1] * s];
  const res = () => (results() && !stale() ? results() : null);
  // The highest ratio of each element over the case shown (or over every checked case).
  const ratioOf = createMemo(() => {
    const r = res(); const map = new Map<number, number>(); if (!r) return map;
    const c = curCase(); const cs = c && c.stresses.length ? [c] : r.cases.filter(x => x.stresses.length);
    for (const x of cs) for (const st of x.stresses) { const el = r.geom.segs[st.seg]?.element; if (el !== undefined) map.set(el, Math.max(map.get(el) ?? 0, st.ratio)); }
    return map;
  });
  // One tag an element: on its longest line on the sheet.
  const tagLine = createMemo(() => {
    const best = new Map<number, [Line, number]>();
    for (const l of p.sh.lines) { const a = l.pts[0], b = l.pts[l.pts.length - 1], d = Math.hypot(b[0] - a[0], b[1] - a[1]); const cur = best.get(l.element); if (!cur || d > cur[1]) best.set(l.element, [l, d]); }
    return new Set([...best.values()].map(x => x[0]));
  });
  const ink = "#1B2733", acc = "#4D7C0F", sup = "#1F7A4D";
  const sym = (y: Sym): JSX.Element => {
    const [x0, y0] = X(y.p), a = y.along, n: P2 = [-a[1], a[0]], k = 9;
    const L = (pts: P2[], extra: Record<string, any> = {}) => <path d={pts.map((q, i) => `${i ? "L" : "M"}${q[0]} ${q[1]}`).join("")} fill="none" stroke={sup} stroke-width="1.6" {...extra} />;
    switch (y.kind) {
      case "anchor": return <g><rect x={x0 - k} y={y0 - k} width={2 * k} height={2 * k} fill="none" stroke={sup} stroke-width="1.8" /><path d={`M${x0 - k} ${y0 - k}L${x0 + k} ${y0 + k}M${x0 + k} ${y0 - k}L${x0 - k} ${y0 + k}`} stroke={sup} stroke-width="1.6" /></g>;
      case "rest": return <g>{L([[x0, y0 + 4], [x0 - 7, y0 + 16], [x0 + 7, y0 + 16], [x0, y0 + 4]])}{L([[x0 - 11, y0 + 19], [x0 + 11, y0 + 19]])}</g>;
      case "guide": return <g>{[-1, 1].map(sg => L([[x0 + n[0] * 8 * sg - a[0] * 8, y0 + n[1] * 8 * sg - a[1] * 8], [x0 + n[0] * 8 * sg + a[0] * 8, y0 + n[1] * 8 * sg + a[1] * 8]], { "stroke-width": 2.4 }))}</g>;
      case "linestop": return <g>{[-1, 1].map(sg => L([[x0 + a[0] * 6 * sg - n[0] * 9, y0 + a[1] * 6 * sg - n[1] * 9], [x0 + a[0] * 6 * sg + n[0] * 9, y0 + a[1] * 6 * sg + n[1] * 9]], { "stroke-width": 2.4 }))}</g>;
      case "restraint": return <g>{L([[x0, y0 - 14], [x0, y0 + 14]])}{L([[x0 - 4, y0 - 9], [x0, y0 - 14], [x0 + 4, y0 - 9]])}{L([[x0 - 4, y0 + 9], [x0, y0 + 14], [x0 + 4, y0 + 9]])}</g>;
      case "spring": return <g>{L([[x0, y0 + 4], [x0 - 5, y0 + 7], [x0 + 5, y0 + 10], [x0 - 5, y0 + 13], [x0 + 5, y0 + 16], [x0, y0 + 19]])}{L([[x0 - 9, y0 + 21], [x0 + 9, y0 + 21]])}</g>;
      case "rotspring": return <circle cx={x0} cy={y0} r={11} fill="none" stroke={sup} stroke-width="1.6" stroke-dasharray="3 2" />;
      case "constant": return <g>{L([[x0, y0 - 4], [x0, y0 - 16]])}<rect x={x0 - 8} y={y0 - 28} width={16} height={12} fill="none" stroke={sup} stroke-width="1.6" /></g>;
      case "nozzle": return <g><rect x={x0 - a[0] * 30 - 16} y={y0 - a[1] * 30 - 12} width={32} height={24} fill="#FBEFD5" stroke="#B7791F" stroke-width="1.4" /><text x={x0 - a[0] * 30} y={y0 - a[1] * 30 + 4} text-anchor="middle" font-size="8.5" fill="#8A5A00">NOZ</text></g>;
      case "tee": return <circle cx={x0} cy={y0} r={3.6} fill={ink} />;
      default: return null;
    }
  };
  const mid = (l: Line): [P2, P2] => { const i = Math.floor((l.pts.length - 1) / 2); const a = X(l.pts[i]), b = X(l.pts[Math.min(l.pts.length - 1, i + 1)]); const d: P2 = [b[0] - a[0], b[1] - a[1]]; const n = Math.hypot(d[0], d[1]) || 1; return [[(a[0] + b[0]) / 2, (a[1] + b[1]) / 2], [d[0] / n, d[1] / n]]; };
  const angle = (a: P2, b: P2) => { let t = Math.atan2(b[1] - a[1], b[0] - a[0]) * 180 / Math.PI; if (t > 90) t -= 180; if (t < -90) t += 180; return t; };
  return (
    <svg class="isosvg" width={p.w} height={p.h} viewBox={`0 0 ${p.w} ${p.h}`} xmlns="http://www.w3.org/2000/svg" font-family="IBM Plex Sans, Segoe UI, sans-serif">
      <rect x={0.5} y={0.5} width={p.w - 1} height={p.h - 1} fill="#fff" stroke={ink} stroke-width="1" />
      {/* north arrow: north is up and to the right */}
      <g transform={`translate(${p.w - 46} 40)`}><path d="M-14 8L14 -8M14 -8l-9 0.5M14 -8l-4 7.5" stroke={ink} stroke-width="1.6" fill="none" /><text x={18} y={-8} font-size="12" font-weight="600" fill={ink}>N</text></g>
      <For each={p.sh.lines}>{l => { const pts = l.pts.map(X); const d = pts.map((q, i) => `${i ? "L" : "M"}${q[0].toFixed(2)} ${q[1].toFixed(2)}`).join(""); const r = ratioOf().get(l.element);
        const [c, t] = mid(l); const n: P2 = [-t[1], t[0]];
        return <g>
          <path d={d} fill="none" stroke={ink} stroke-width={l.kind === "rigid" ? 4.5 : 2.4} stroke-linejoin="round" stroke-linecap="round" />
          <Show when={l.kind === "valve"}><path d={`M${c[0] - t[0] * 10 + n[0] * 7} ${c[1] - t[1] * 10 + n[1] * 7}L${c[0] + t[0] * 10 - n[0] * 7} ${c[1] + t[1] * 10 - n[1] * 7}L${c[0] + t[0] * 10 + n[0] * 7} ${c[1] + t[1] * 10 + n[1] * 7}L${c[0] - t[0] * 10 - n[0] * 7} ${c[1] - t[1] * 10 - n[1] * 7}Z`} fill="#fff" stroke={ink} stroke-width="1.6" /></Show>
          <Show when={l.kind === "flange"}>{[pts[0], pts[pts.length - 1]].map(q => <path d={`M${q[0] - n[0] * 8} ${q[1] - n[1] * 8}L${q[0] + n[0] * 8} ${q[1] + n[1] * 8}`} stroke={ink} stroke-width="2.2" />)}</Show>
          <Show when={l.kind === "reducer"}><path d={`M${c[0] - t[0] * 8 + n[0] * 7} ${c[1] - t[1] * 8 + n[1] * 7}L${c[0] + t[0] * 8 + n[0] * 4} ${c[1] + t[1] * 8 + n[1] * 4}L${c[0] + t[0] * 8 - n[0] * 4} ${c[1] + t[1] * 8 - n[1] * 4}L${c[0] - t[0] * 8 - n[0] * 7} ${c[1] - t[1] * 8 - n[1] * 7}Z`} fill="#fff" stroke={ink} stroke-width="1.6" /></Show>
          <Show when={isoOpts().ratios && r !== undefined && tagLine().has(l)}><g transform={`translate(${c[0] + n[0] * -16} ${c[1] + n[1] * -16})`}><rect x={-15} y={-8} width={30} height={14} rx={2} fill={r! > 1 ? "#FBE3E0" : "#E4F0CF"} stroke={r! > 1 ? "#C0392B" : acc} stroke-width="0.8" /><text y={3} text-anchor="middle" font-size="8.5" font-family="IBM Plex Mono, monospace" fill={r! > 1 ? "#9B2C2C" : "#3F6212"}>{num(r!, 2)}</text></g></Show>
        </g>; }}</For>
      <Show when={isoOpts().dims}><For each={p.sh.dims}>{d => { const o = 20; const a = X(d.a), b = X(d.b); const a2: P2 = [a[0] + d.off[0] * o, a[1] + d.off[1] * o], b2: P2 = [b[0] + d.off[0] * o, b[1] + d.off[1] * o];
        const len = Math.hypot(b[0] - a[0], b[1] - a[1]); if (len < 16) return null;
        const c: P2 = [(a2[0] + b2[0]) / 2 + d.off[0] * 7, (a2[1] + b2[1]) / 2 + d.off[1] * 7];
        return <g class="dim"><path d={`M${a[0] + d.off[0] * 4} ${a[1] + d.off[1] * 4}L${a2[0] + d.off[0] * 3} ${a2[1] + d.off[1] * 3}M${b[0] + d.off[0] * 4} ${b[1] + d.off[1] * 4}L${b2[0] + d.off[0] * 3} ${b2[1] + d.off[1] * 3}M${a2[0]} ${a2[1]}L${b2[0]} ${b2[1]}`} stroke="#55606E" stroke-width="0.7" fill="none" />
          <circle cx={a2[0]} cy={a2[1]} r={1.6} fill="#55606E" /><circle cx={b2[0]} cy={b2[1]} r={1.6} fill="#55606E" />
          <text x={c[0]} y={c[1]} transform={`rotate(${angle(a2, b2)} ${c[0]} ${c[1]})`} text-anchor="middle" dominant-baseline="middle" font-size="9" font-family="IBM Plex Mono, monospace" fill="#3B4859" paint-order="stroke" stroke="#fff" stroke-width="3">{isoLen(d.length)}</text></g>; }}</For></Show>
      <For each={p.sh.syms}>{y => sym(y)}</For>
      <Show when={isoOpts().labels}><For each={p.sh.labels}>{l => { const [x, y] = X(l.p); return <text x={x + 6} y={y - 6} font-size="10" font-weight="600" font-family="IBM Plex Mono, monospace" fill={acc} paint-order="stroke" stroke="#fff" stroke-width="3">{l.num}</text>; }}</For></Show>
      <For each={p.sh.syms.filter(y => y.tag && y.kind !== "tee")}>{y => { const [x, yy] = X(y.p); return <text x={x + 12} y={yy + 22} font-size="8.5" fill={sup}>{y.tag}</text>; }}</For>
      <Show when={isoOpts().loads && res()}>{_ => { const r = res()!; const c = r.cases.find(x => x.kind === "ope") ?? r.cases[0];
        return <For each={p.sh.syms.filter(y => !["tee", "nozzle"].includes(y.kind))}>{y => { const rx = c.reactions.find(x => x.support === y.id); if (!rx) return null; const [x, yy] = X(y.p); const big = [0, 1, 2].filter(k => Math.abs(rx.f[k]) > 1e-6);
          return <text x={x + 12} y={yy + 33} font-size="8" font-family="IBM Plex Mono, monospace" fill="#3B4859">{big.map(k => `F${"XYZ"[k]} ${num(toU("force", rx.f[k]), 3)}`).join(" ")} {unit("force")}</text>; }}</For>; }}</Show>
      <For each={p.sh.cont}>{([n, o]) => { const l = p.sh.labels.find(x => x.num === n); if (!l) return null; const [x, y] = X(l.p); return <text x={x + 6} y={y + 14} font-size="8.5" font-style="italic" fill="#8A5A00">Continued on sheet {o}</text>; }}</For>
      <text x={14} y={p.h - 12} font-size="8.5" fill="#55606E">Dimensions in {model()?.units === "US" ? "feet and inches" : "mm"}, node to node. Not to scale. {p.print ? "" : `Sheet ${p.sh.n}.`}</text>
    </svg>
  );
};

export const IsoRibbon = () => <>
  <G l="Isometric" h="iso"><B big i="iso" l="Isometric" run={() => setView("iso")} /><B big i="print" l="Print" run={() => { setPs("iso", true); setBackstage("print"); }} /><S><B i="export" l="Save as SVG" run={saveSvg} /></S></G>
  <G l="Show" h="iso"><S>
    <label class="chk"><input type="checkbox" checked={isoOpts().dims} onChange={e => setIsoOpts({ ...isoOpts(), dims: e.currentTarget.checked })} />Dimensions</label>
    <label class="chk"><input type="checkbox" checked={isoOpts().labels} onChange={e => setIsoOpts({ ...isoOpts(), labels: e.currentTarget.checked })} />Node numbers</label>
  </S><S>
    <label class="chk"><input type="checkbox" checked={isoOpts().ratios} onChange={e => setIsoOpts({ ...isoOpts(), ratios: e.currentTarget.checked })} />Stress ratio tags</label>
    <label class="chk"><input type="checkbox" checked={isoOpts().loads} onChange={e => setIsoOpts({ ...isoOpts(), loads: e.currentTarget.checked })} />Support load tags</label>
  </S></G>
  <G l="Sheets" h="iso"><S><span class="inline">Elements a sheet <select onChange={e => { setIsoOpts({ ...isoOpts(), perSheet: +e.currentTarget.value }); setSheetNo(1); }}><For each={[[0, "All on one"], [20, "20"], [40, "40"], [80, "80"]] as [number, string][]}>{([v, l]) => <option value={v} selected={v === isoOpts().perSheet}>{l}</option>}</For></select></span></S></G>
</>;

const saveSvg = async () => {
  const svg = document.querySelector(".isopage .isosvg")?.outerHTML;
  if (!svg) return toast("Open the Isometric view first (sidebar › Isometric).");
  const path = await call<string | null>("save_file", { name: `${model()?.name ?? "isometric"} sheet ${sheetNo()}.svg`, data: svg });
  if (path) toast(`Saved ${path}`, true);
};

export const IsoPage = () => {
  const sheets = () => iso()?.sheets ?? [];
  const sh = () => sheets().find(s => s.n === sheetNo()) ?? sheets()[0];
  return (
    <div class="page isopage">
      <h1>Isometric <Help k="iso" /><span class="muted">{sheets().length > 1 ? `${sheets().length} sheets` : ""}</span><span class="grow" />
        <button class="btn sm" onClick={saveSvg}>Save as SVG</button><button class="btn sm pri" onClick={() => { setPs("iso", true); setBackstage("print"); }}>Print or save as PDF</button></h1>
      <Show when={sheets().length > 1}><div class="ctabs"><For each={sheets()}>{s => <button classList={{ on: s.n === sh()?.n }} onClick={() => setSheetNo(s.n)}><b>Sheet {s.n}</b><small>{s.dims.length} elements</small></button>}</For></div></Show>
      <Show when={sh() && model()} fallback={<div class="emptypage"><b>Nothing to draw yet</b><span>The isometric draws itself from the element table: type the piping first.</span></div>}>
        <div class="isowrap"><IsoSheet sh={sh()!} w={1120} h={720} m={model()!} /><TitleBlock m={model()!} title={`Isometric, sheet ${sh()!.n} of ${sheets().length}`} /></div>
      </Show>
      <Show when={results() && !stale() && isoOpts().ratios}><p class="muted small">Tags show the highest stress ratio of each element in {curCase()?.stresses.length ? curCase()!.name : "any checked case"} ({fmt("stress", 0).replace("0 ", "")} stresses against their allowables).</p></Show>
    </div>
  );
};

// The isometric sheets in the printed report.
setIsoPages((m, w, h) => (iso()?.sheets ?? []).map(s => <div class="preport"><IsoSheet sh={s} w={w} h={h - 70} m={m} print /><TitleBlock m={m} title={`Isometric, sheet ${s.n} of ${iso()!.sheets.length}`} /></div>));
