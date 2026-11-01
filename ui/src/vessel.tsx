// The Vessels view: pressure vessels in the same model as the piping. Each vessel is its design
// data and a row of parts (heads, cylinders, cones) from the bottom (or the left) to the top; the
// engine checks them after every edit, and this page shows the drawing, the thicknesses, the MAWP,
// the test pressure and the weights.
import { For, Show, createSignal, JSX } from "solid-js";
import { Vessel, VPart, VesselCheck, VesselResult, invoke, num } from "./api";
import { model, act, toast, onEditHook, onLoadHook } from "./store";
import { Help } from "./help";
import { F, T } from "./props";
import { Q, toU, fromU, unit, parseQ } from "./units";
import { Badge } from "./results";

export const [vchecks, setVchecks] = createSignal<VesselCheck[]>([]);
const refresh = () => {
  if (!(model()?.vessels ?? []).length) { setVchecks([]); return; }
  invoke<VesselCheck[]>("vessel_check").then(setVchecks).catch(() => setVchecks([]));
};
onEditHook(refresh);
onLoadHook(refresh);

export const [vesselNo, setVesselNo] = createSignal(0);
const list = () => model()?.vessels ?? [];
const setList = (l: Vessel[]) => act({ SetVessels: { list: l } });
const putV = (i: number, x: Partial<Vessel>) => setList(list().map((v, k) => k === i ? { ...v, ...x } : v));

/** A new vessel with the usual parts: two 2:1 heads and a shell (example values to change). */
const fresh = (vertical: boolean, n: number): Vessel => {
  const head = (name: string): VPart => ({ name, kind: "ellipsoidal", d: 1.5, len: 0, t: 0.014, e: 1 });
  return { name: `V-${n}`, vertical, p: 10e5, temp: 273.15 + 150, s: 138e6, s_cold: 138e6, sy: 260e6, corrosion: 0.003, under: 0, rho: 7850, liquid: 1000, level: 0, extra: 0,
    parts: [head(vertical ? "Bottom head" : "Left head"), { name: "Shell", kind: "cylinder", d: 1.5, len: 4, t: 0.014, e: 1 }, head(vertical ? "Top head" : "Right head")] };
};

const KINDS: [string, string][] = [["cylinder", "Cylinder"], ["ellipsoidal", "2:1 ellipsoidal head"], ["torispherical", "Torispherical head"], ["hemispherical", "Hemispherical head"], ["cone", "Cone"]];
const kindOf = (p: VPart, k: string): VPart => {
  const base = { name: p.name, d: p.d, len: p.len, t: p.t, e: p.e };
  if (k === "torispherical") return { ...base, kind: "torispherical", crown: 1, knuckle: 0.06 };
  if (k === "cone") return { ...base, kind: "cone", d_end: p.d * 0.7, len: p.len || p.d * 0.5 };
  return { ...base, kind: k as any };
};

/** A number cell in display units. */
const cell = (q: Q, v: number, f: (x: number) => void, w = 70) => <input class="mono" style={{ width: `${w}px` }} value={num(toU(q, v), 6)} onChange={e => { const x = parseQ(e.currentTarget.value || "0", q); if (typeof x === "string") { e.currentTarget.value = num(toU(q, v), 6); return toast(x); } f(fromU(q, x)); }} />;

const v4 = (q: Q, x: number) => num(toU(q, x), 4);

/** The vessel drawn from its outline: the outside and inside surfaces, tangent lines, the liquid. */
export const VesselDrawing = (p: { v: Vessel; r: VesselResult; w?: number; h?: number }) => {
  const W = p.w ?? 300, H = p.h ?? 440;
  const pts = () => p.r.outline;
  const box = () => {
    const zs = pts().map(q => q[1]), rs = pts().map(q => q[0]);
    return { z0: Math.min(...zs), z1: Math.max(...zs), r: Math.max(...rs) };
  };
  // Vertical: z up the page; horizontal: z across it.
  const sc = () => { const b = box(); const len = b.z1 - b.z0 || 1, wid = 2 * b.r || 1; return p.v.vertical ? Math.min((H - 40) / len, (W - 90) / wid) : Math.min((W - 40) / len, (H - 90) / wid); };
  const at = (r: number, z: number): [number, number] => {
    const b = box(), s = sc();
    return p.v.vertical ? [W / 2 + r * s, H - 20 - (z - b.z0) * s] : [20 + (z - b.z0) * s, H / 2 - r * s];
  };
  const path = (k: 0 | 2) => { const o = pts(); const side = (sg: number) => o.map(q => at(sg * q[k], q[1])); const a = side(1), b = side(-1).reverse(); return "M" + [...a, ...b].map(x => `${x[0].toFixed(1)} ${x[1].toFixed(1)}`).join("L") + "Z"; };
  const tangents = () => p.r.parts.flatMap((x, i) => i === 0 ? [] : [x.z[0]]);
  const level = () => p.v.vertical ? p.v.level : null;
  return (
    <svg class="vdraw" width={W} height={H} viewBox={`0 0 ${W} ${H}`}>
      <path d={path(0)} fill="var(--bg2)" stroke="var(--text)" stroke-width="1.6" />
      <path d={path(2)} fill="var(--panel)" stroke="var(--muted)" stroke-width="0.8" />
      <Show when={level() !== null && p.v.level > 0 && p.v.liquid > 0}>{_ => { const b = box(); const y = Math.min(p.v.level, b.z1); const [x0, y0] = at(-b.r, y), [x1] = at(b.r, y); return <line x1={x0} y1={y0} x2={x1} y2={y0} stroke="var(--blue)" stroke-width="1.4" stroke-dasharray="6 3" />; }}</Show>
      <For each={tangents()}>{z => { const b = box(); const [x0, y0] = at(-b.r - 0.06 / sc() * 40, z), [x1, y1] = at(b.r + 0.06 / sc() * 40, z); return <line x1={x0} y1={y0} x2={x1} y2={y1} stroke="var(--muted)" stroke-width="0.8" stroke-dasharray="2 3" />; }}</For>
      <For each={p.r.parts}>{(x, i) => { const b = box(); const zm = (x.z[0] + x.z[1]) / 2; const [tx, ty] = p.v.vertical ? at(b.r, zm) : at(0, zm); return <text x={p.v.vertical ? tx + 8 : tx} y={p.v.vertical ? ty + 4 : ty} font-size="11" text-anchor={p.v.vertical ? "start" : "middle"} fill={x.pass ? "var(--text)" : "var(--crit)"}>{x.name}{p.r.governs === i() ? " ◂" : ""}</text>; }}</For>
    </svg>
  );
};

/** The results tables of one vessel (also used in the report). */
export const VesselTables = (p: { v: Vessel; r: VesselResult }) => (
  <>
    <div class="vsum">
      <div><small>MAWP at the top, hot and corroded</small><b>{v4("press", p.r.mawp)} {unit("press")}</b><small class="muted">{p.r.governs !== null ? `set by ${p.r.parts[p.r.governs].name}` : ""}</small></div>
      <div><small>MAP, new and cold</small><b>{v4("press", p.r.map)} {unit("press")}</b></div>
      <div><small>Hydrotest pressure at the top</small><b>{v4("press", p.r.test)} {unit("press")}</b><small class="muted">1.3 × MAWP × S cold / S hot</small></div>
      <div><small>Volume</small><b>{v4("vol", p.r.volume)} {unit("vol")}</b></div>
      <div><small>Weight empty · operating · full of water</small><b>{v4("mass", p.r.empty)} · {v4("mass", p.r.operating)} · {v4("mass", p.r.test_weight)} {unit("mass")}</b></div>
    </div>
    <table class="tbl rtable"><thead><tr><th>Part</th><th class="r">Design P {unit("press")}</th><th class="r">Static head</th><th class="r">t for pressure {unit("len")}</th><th class="r">t + corrosion</th><th class="r">Nominal</th><th class="r">MAWP</th><th class="r">MAWP at top</th><th class="r">MAP</th><th class="r">Test stress {unit("stress")}</th><th class="r">0.9 Sy</th><th>Result</th></tr></thead><tbody>
      <For each={p.r.parts}>{x => <tr><td title={x.formula}>{x.name}</td><td class="r">{v4("press", x.p)}</td><td class="r">{v4("press", x.static_head)}</td><td class="r">{v4("len", x.t_req)}</td><td class="r">{v4("len", x.t_min)}</td><td class="r">{v4("len", x.t_nom)}</td><td class="r">{v4("press", x.mawp)}</td><td class="r">{v4("press", x.mawp_top)}</td><td class="r">{v4("press", x.map)}</td><td class="r">{v4("stress", x.test_stress)}</td><td class="r">{p.v.sy > 0 ? v4("stress", x.test_allow) : "—"}</td><td><Badge pass={x.pass} /></td></tr>}</For>
    </tbody></table>
    <p class="muted small mono">{[...new Set(p.r.parts.map(x => x.formula))].map(f => `${p.r.parts.filter(x => x.formula === f).map(x => x.name).join(", ")}: ${f}`).join(" · ")}</p>
    <p class="muted small">Thin-shell membrane equations on the corroded inside dimensions, with the liquid's static head at each part's lowest point. MAWP at the top is the lowest of the parts' MAWP less their static head. The allowable stresses, joint efficiencies and yield strength are your inputs: check them against your code edition and the material certificates.</p>
  </>
);

const PartsTable = (p: { i: number; v: Vessel }) => {
  const put = (j: number, x: Partial<VPart>) => putV(p.i, { parts: p.v.parts.map((q, k) => k === j ? ({ ...q, ...x } as VPart) : q) });
  const move = (j: number, d: number) => { const a = [...p.v.parts]; const k = j + d; if (k < 0 || k >= a.length) return; [a[j], a[k]] = [a[k], a[j]]; putV(p.i, { parts: a }); };
  // A new shell course or cone goes before the far head (or at the end), at the diameter it meets.
  const add = (kind: string) => {
    const parts = [...p.v.parts];
    const isShell = (x?: VPart) => !!x && (x.kind === "cylinder" || x.kind === "cone");
    const lastHead = parts.length > 0 && !isShell(parts[parts.length - 1]);
    const ref = lastHead ? parts[parts.length - 2] ?? parts[parts.length - 1] : parts[parts.length - 1];
    const d = ref ? (ref.kind === "cone" ? ref.d_end : ref.d) : 1.5;
    const np = kindOf({ name: kind === "cone" ? "Cone" : `Shell ${parts.filter(isShell).length + 1}`, d, len: 2, t: ref?.t ?? 0.012, e: 1, kind: "cylinder" }, kind);
    parts.splice(lastHead ? parts.length - 1 : parts.length, 0, np);
    putV(p.i, { parts });
  };
  return (
    <>
      <table class="tbl vparts"><thead><tr><th>{p.v.vertical ? "Bottom to top" : "Left to right"}</th><th>Kind</th><th>Inside D ({unit("len")})</th><th>Length ({unit("coord")})</th><th>Thickness ({unit("len")})</th><th>E</th><th>More</th><th /></tr></thead><tbody>
        <For each={p.v.parts}>{(q, j) => <tr>
          <td><input style={{ width: "100px" }} value={q.name} onChange={e => put(j(), { name: e.currentTarget.value })} /></td>
          <td><select style={{ width: "150px" }} onChange={e => putV(p.i, { parts: p.v.parts.map((x, k) => k === j() ? kindOf(x, e.currentTarget.value) : x) })}><For each={KINDS}>{k => <option value={k[0]} selected={k[0] === q.kind}>{k[1]}</option>}</For></select></td>
          <td>{cell("len", q.d, d => d > 0 ? put(j(), { d }) : toast("The diameter must be above 0."))}</td>
          <td><Show when={q.kind === "cylinder" || q.kind === "cone"} fallback={<span class="muted">—</span>}>{cell("coord", q.len, len => len > 0 ? put(j(), { len }) : toast("The length must be above 0."))}</Show></td>
          <td>{cell("len", q.t, t => t > 0 ? put(j(), { t }) : toast("The thickness must be above 0."), 60)}</td>
          <td><input class="mono" style={{ width: "46px" }} value={q.e} onChange={e => { const x = +e.currentTarget.value; if (x > 0 && x <= 1) put(j(), { e: x }); else { e.currentTarget.value = String(q.e); toast("The joint efficiency is between 0 and 1."); } }} /></td>
          <td class="small">{q.kind === "torispherical" ? <span class="inline">L/D <input class="mono" style={{ width: "42px" }} value={(q as any).crown} onChange={e => put(j(), { crown: +e.currentTarget.value } as any)} /> r/D <input class="mono" style={{ width: "42px" }} value={(q as any).knuckle} onChange={e => put(j(), { knuckle: +e.currentTarget.value } as any)} /></span> : q.kind === "cone" ? <span class="inline">end D {cell("len", (q as any).d_end, d_end => put(j(), { d_end } as any), 64)}</span> : ""}</td>
          <td class="nowrap"><button class="x" title="Move down" onClick={() => move(j(), -1)}>↑</button><button class="x" title="Move up" onClick={() => move(j(), 1)}>↓</button><button class="x" title="Remove this part" onClick={() => putV(p.i, { parts: p.v.parts.filter((_, k) => k !== j()) })}>×</button></td>
        </tr>}</For>
      </tbody></table>
      <p class="ibtns"><button class="btn sm" onClick={() => add("cylinder")}>Add a shell course</button><button class="btn sm" onClick={() => add("cone")}>Add a cone</button></p>
    </>
  );
};

export const VesselPage = () => {
  const i = () => Math.min(vesselNo(), Math.max(0, list().length - 1));
  const v = () => list()[i()];
  const chk = () => vchecks()[i()];
  const addV = (vertical: boolean) => { setList([...list(), fresh(vertical, list().length + 1)]); setVesselNo(list().length); };
  return (
    <div class="page vpage">
      <h1>Vessels <Help k="vessels" /><span class="grow" /><button class="btn sm" onClick={() => addV(true)}>Add a vertical vessel</button><button class="btn sm" onClick={() => addV(false)}>Add a horizontal vessel</button></h1>
      <Show when={v()} fallback={<div class="emptypage"><b>No vessels yet</b><span>Add a vertical or a horizontal vessel: two heads and a shell to start from, with example values to change. Or open File › Vertical separator V-101.</span><p class="ibtns"><button class="btn pri" onClick={() => addV(true)}>Add a vertical vessel</button><button class="btn" onClick={() => addV(false)}>Add a horizontal vessel</button></p></div>}>
        <div class="ctabs"><For each={list()}>{(x, k) => <button classList={{ on: k() === i() }} onClick={() => setVesselNo(k())}><b>{x.name}</b><small>{x.vertical ? "vertical" : "horizontal"}</small><Badge pass={vchecks()[k()]?.result?.pass} /></button>}</For></div>
        <div class="vgrid">
          <div class="vleft">
            <h4>Design data</h4>
            <div class="two"><T l="Name" v={v()!.name} set={name => putV(i(), { name })} /><div class="iprow"><label>Orientation</label><select onChange={e => putV(i(), { vertical: e.currentTarget.value === "v" })}><option value="v" selected={v()!.vertical}>Vertical</option><option value="h" selected={!v()!.vertical}>Horizontal</option></select></div></div>
            <div class="two"><F l="Design pressure" q="press" v={v()!.p} set={p => p >= 0 ? putV(i(), { p }) : toast("The pressure must be 0 or more.")} /><F l="Design temperature" q="temp" v={v()!.temp} set={temp => putV(i(), { temp })} /></div>
            <div class="two"><F l="Allowable stress, hot" q="stress" v={v()!.s} set={s => s > 0 && putV(i(), { s })} /><F l="Allowable stress, cold" q="stress" v={v()!.s_cold} set={s_cold => s_cold > 0 && putV(i(), { s_cold })} /></div>
            <div class="two"><F l="Yield strength, cold (0 = no test check)" q="stress" v={v()!.sy} set={sy => sy >= 0 && putV(i(), { sy })} /><F l="Corrosion allowance" q="len" v={v()!.corrosion} set={corrosion => corrosion >= 0 && putV(i(), { corrosion })} /></div>
            <div class="two"><F l="Plate under-tolerance" q="len" v={v()!.under} set={under => under >= 0 && putV(i(), { under })} /><F l="Metal density" q="density" v={v()!.rho} set={rho => rho >= 0 && putV(i(), { rho })} /></div>
            <div class="two"><F l="Liquid density" q="density" v={v()!.liquid} set={liquid => liquid >= 0 && putV(i(), { liquid })} /><F l={v()!.vertical ? "Liquid level above the bottom tangent" : "Liquid level above the bottom"} q="coord" v={v()!.level} set={level => level >= 0 && putV(i(), { level })} /></div>
            <div class="two"><F l="Internals, platforms, insulation" q="mass" v={v()!.extra} set={extra => extra >= 0 && putV(i(), { extra })} /><div /></div>
            <p class="muted small">Example values: the allowable stresses, joint efficiencies and yield strength are your inputs from your code edition and material certificates.</p>
            <h4>Parts</h4>
            <PartsTable i={i()} v={v()!} />
            <p class="ibtns"><button class="btn sm" onClick={() => { setList(list().filter((_, k) => k !== i())); setVesselNo(Math.max(0, i() - 1)); }}>Delete this vessel</button></p>
          </div>
          <div class="vright">
            <Show when={chk()?.result} fallback={<p class="warnline">{chk()?.error ?? "Checking…"}</p>}>{_ => <VesselDrawing v={v()!} r={chk()!.result!} w={320} h={480} />}</Show>
          </div>
        </div>
        <Show when={chk()?.result}>{_ => <div class="vres">
          <h4>Results</h4>
          <Show when={chk()!.result!.warnings.length}><p class="warnline">{chk()!.result!.warnings.join(" ")}</p></Show>
          <div class="vtables"><VesselTables v={v()!} r={chk()!.result!} /></div>
        </div>}</Show>
      </Show>
    </div>
  );
};

/** The vessels' pages of the report. */
export const vesselReport = (): (() => JSX.Element)[] => (model()?.vessels ?? []).map((x, k) => () => {
  const c = vchecks()[k];
  return <div><h3>Vessel {x.name}</h3>
    <p class="muted small">{x.vertical ? "Vertical" : "Horizontal"}; design {v4("press", x.p)} {unit("press")} at {v4("temp", x.temp)} {unit("temp")}; S hot {v4("stress", x.s)}, cold {v4("stress", x.s_cold)} {unit("stress")}; corrosion {v4("len", x.corrosion)} {unit("len")}; liquid {v4("density", x.liquid)} {unit("density")} to {v4("coord", x.level)} {unit("coord")}.</p>
    <Show when={c?.result} fallback={<p class="warnline">{c?.error}</p>}><div class="vtop"><VesselDrawing v={x} r={c!.result!} w={220} h={300} /><div class="vtables"><VesselTables v={x} r={c!.result!} /></div></div></Show></div>;
});
