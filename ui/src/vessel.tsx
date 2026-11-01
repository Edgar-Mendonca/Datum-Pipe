// The Vessels view: pressure vessels in the same model as the piping. Each vessel is its design
// data and a row of parts (heads, cylinders, cones) from the bottom (or the left) to the top; the
// engine checks them after every edit, and this page shows the drawing, the thicknesses, the MAWP,
// the test pressure and the weights.
import { For, Show, createSignal, createResource, JSX } from "solid-js";
import { Vessel, VPart, VNozzle, BodyFlange, VesselCheck, VesselResult, Library, invoke, num } from "./api";
import { model, act, toast, onEditHook, onLoadHook } from "./store";
import { Help } from "./help";
import { F, T, library } from "./props";
import { Q, toU, fromU, unit, parseQ } from "./units";
import { Badge } from "./results";
import { results, stale } from "./rstate";

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
    const zlow = p.v.vertical && p.v.skirt ? -p.v.skirt.h : Math.min(...zs);
    return { z0: Math.min(zlow, ...zs), z1: Math.max(...zs), r: Math.max(...rs, p.v.vertical && p.v.skirt ? p.v.skirt.ring_od / 2 : 0) };
  };
  // Vertical: z up the page; horizontal: z across it.
  const sc = () => { const b = box(); const len = b.z1 - b.z0 || 1, wid = 2 * b.r || 1; return p.v.vertical ? Math.min((H - 90) / len, (W - 120) / wid) : Math.min((W - 110) / len, (H - 90) / wid); };
  const at = (r: number, z: number): [number, number] => {
    const b = box(), s = sc();
    return p.v.vertical ? [W / 2 + r * s, H - 45 - (z - b.z0) * s] : [55 + (z - b.z0) * s, H / 2 - r * s];
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
      <Show when={p.v.vertical && p.v.skirt}>{_ => { const k = p.v.skirt!; const shell = p.v.parts.find(x => x.kind === "cylinder"); const ro = shell ? shell.d / 2 + shell.t : box().r; const [xa, ya] = at(-ro, 0), [xb] = at(ro, 0); const [, yb] = at(0, -k.h); const [xr0] = at(-k.ring_od / 2, 0), [xr1] = at(k.ring_od / 2, 0);
        return <g><path d={`M${xa} ${ya}V${yb}M${xb} ${ya}V${yb}`} stroke="var(--text)" stroke-width="1.6" fill="none" /><rect x={xr0} y={yb - 3} width={xr1 - xr0} height={4} fill="var(--panel2)" stroke="var(--text)" stroke-width="1" /><line x1={xr0 - 20} y1={yb + 1} x2={xr1 + 20} y2={yb + 1} stroke="var(--muted)" stroke-width="1" stroke-dasharray="4 3" /></g>; }}</Show>
      <Show when={!p.v.vertical && p.v.saddles}>{_ => { const sd = p.v.saddles!; const b = box(); const t0 = p.r.parts[0]?.z[1] ?? 0, t1 = p.r.parts[p.r.parts.length - 1]?.z[0] ?? 0; const s = sc();
        return <For each={[t0 + sd.a, t1 - sd.a]}>{z => { const [x, y] = at(-b.r, z); const w = Math.max(sd.b * s, 6); const half = b.r * Math.sin(Math.min(sd.theta, 180) * Math.PI / 360) * s; return <path d={`M${x - half} ${y - 4}L${x - half * 0.8} ${y + 22}L${x + half * 0.8} ${y + 22}L${x + half} ${y - 4}M${x - w / 2} ${y + 22}h${w}`} fill="var(--panel2)" stroke="var(--text)" stroke-width="1.2" opacity="0.9" />; }}</For>; }}</Show>
      <For each={p.r.nozzles ?? []}>{(z, k) => { const nz = (p.v.nozzles ?? [])[k()]; if (!nz) return null; const part = p.v.parts[nz.part]; const onShell = part && (part.kind === "cylinder" || part.kind === "cone");
        const side = Math.cos((nz.angle * Math.PI) / 180) >= -1e-9 ? 1 : -1; const s = sc(); const L = Math.max(nz.proj, 0.05) * s, w = nz.od * s;
        const [x0, y0] = onShell ? at(side * z.at[0], z.at[1]) : at(z.at[0], z.at[1]);
        const dirx = onShell ? (p.v.vertical ? side : 0) : (p.v.vertical ? 0 : (nz.part === 0 ? -1 : 1)), diry = onShell ? (p.v.vertical ? 0 : -side) : (p.v.vertical ? (nz.part === 0 ? 1 : -1) : 0);
        const nx = -diry, ny = dirx;
        const q = (a: number, b: number) => `${(x0 + dirx * a + nx * b).toFixed(1)} ${(y0 + diry * a + ny * b).toFixed(1)}`;
        return <g><path d={`M${q(0, -w / 2)}L${q(L, -w / 2)}L${q(L, w / 2)}L${q(0, w / 2)}`} fill="var(--bg2)" stroke={z.pass ? "var(--text)" : "var(--crit)"} stroke-width="1.3" /><path d={`M${q(L, -w / 2 - 3)}L${q(L, w / 2 + 3)}`} stroke={z.pass ? "var(--text)" : "var(--crit)"} stroke-width="2.4" /><text x={x0 + dirx * (L + 6) + (dirx === 0 ? 4 : 0)} y={y0 + diry * (L + 6) + (diry === 0 ? -4 : 4)} font-size="10" text-anchor={dirx < 0 ? "end" : "start"} fill="var(--muted)">{nz.name.split(" ")[0]}</text></g>; }}</For>
      <For each={p.r.parts}>{(x, i) => { const b = box(); const zm = (x.z[0] + x.z[1]) / 2; const [tx, ty] = p.v.vertical ? at(b.r, zm) : at(b.r, zm); return <text x={p.v.vertical ? tx + 8 : tx} y={p.v.vertical ? ty + 4 : ty - 30 - (i() % 2) * 12} font-size="11" text-anchor={p.v.vertical ? "start" : "middle"} fill={x.pass ? "var(--text)" : "var(--crit)"}>{x.name}{p.r.governs === i() ? " ◂" : ""}</text>; }}</For>
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
    <Show when={(p.r.nozzles ?? []).length}>
      <h4>Nozzles</h4>
      <table class="tbl rtable"><thead><tr><th>Nozzle</th><th class="r">d {unit("len")}</th><th class="r">tr</th><th class="r">trn</th><th class="r">Required {unit("area")}</th><th class="r">A1 shell</th><th class="r">A2 neck</th><th class="r">Welds</th><th class="r">A5 pad</th><th class="r">Available</th><th class="r">Neck min. {unit("len")}</th><th class="r">Neck</th><th>Result</th></tr></thead><tbody>
        <For each={p.r.nozzles}>{z => <tr><td>{z.name}</td><td class="r">{v4("len", z.d)}</td><td class="r">{v4("len", z.tr)}</td><td class="r">{v4("len", z.trn)}</td><td class="r">{v4("area", z.a_req)}</td><td class="r">{v4("area", Math.max(0, z.a1))}</td><td class="r">{v4("area", z.a2)}</td><td class="r">{v4("area", z.a41 + z.a42)}</td><td class="r">{v4("area", z.a5)}</td><td class="r" classList={{ bad: !z.area_pass }}>{z.small ? "small opening" : v4("area", z.a_avail)}</td><td class="r">{v4("len", z.t_neck_req)}</td><td class="r" classList={{ bad: !z.neck_pass }}>{v4("len", z.t_neck)}</td><td><Badge pass={z.pass} /></td></tr>}</For>
      </tbody></table>
      <p class="muted small">Area replacement: the opening's d × tr (tr the wall the part needs with E = 1) made up by the shell's excess (A1), the neck's (A2), the welds and the pad, within the reinforcement limit (pad up to 2 d). The neck must be at least the larger of its pressure thickness and the smaller of the shell's need and the standard-wall pipe's 87.5 %, with corrosion. Pipe necks count 87.5 % of their wall.</p>
    </Show>
    <Show when={(p.r.external ?? []).length}>
      <h4>External pressure ({v4("press", p.v.pe ?? 0)} {unit("press")})</h4>
      <table class="tbl rtable"><thead><tr><th>Part</th><th class="r">Design length / radius {unit("coord")}</th><th class="r">t {unit("len")}</th><th class="r">Elastic collapse {unit("press")}</th><th class="r">Yield limit</th><th class="r">Allowable</th><th class="r">Ratio</th><th>Result</th></tr></thead><tbody>
        <For each={p.r.external}>{x => <tr><td>{x.name}</td><td class="r">{v4("coord", x.length)}</td><td class="r">{v4("len", x.t)}</td><td class="r">{v4("press", x.elastic)}</td><td class="r">{v4("press", x.yield_p)}</td><td class="r">{v4("press", x.allow)}</td><td class="r">{num(x.ratio, 3)}</td><td><Badge pass={x.pass} /></td></tr>}</For>
        <Show when={p.r.rings}>{_ => { const g = p.r.rings!; return <tr><td>Stiffening rings: moment of inertia needed / given (m⁴)</td><td colSpan={4} /><td class="r">{g.value.toExponential(3)} / {g.allow.toExponential(3)}</td><td class="r">{num(g.ratio, 3)}</td><td><Badge pass={g.pass} /></td></tr>; }}</Show>
      </tbody></table>
      <p class="muted small">Classical shell buckling: a cylinder between stiffeners collapses at 2.42 E (t/D)^2.5 / ((1 − ν²)^0.75 (L/D − 0.45 √(t/D))) (never below the long-tube 2E/(1 − ν²)(t/D)³), L the shell between the heads plus a third of each head's depth, or the ring spacing; heads as spheres of their crown radius (0.9 D for 2:1), 0.1875 E (t/R)² with a knockdown for imperfections; each capped by yield and divided by the design factor ({num(p.v.ext_fs ?? 3, 3)}). A ring needs I ≥ factor × pe R³ Ls / (3 E).</p>
    </Show>
    <Show when={(p.r.flanges ?? []).length}>
      <h4>Bolted flanges and covers</h4>
      <table class="tbl rtable"><thead><tr><th>Flange</th><th class="r">G {unit("len")}</th><th class="r">Wm1 {unit("force")}</th><th class="r">Wm2</th><th class="r">Bolt area needed {unit("area")}</th><th class="r">Bolt area</th><th class="r">Operating</th><th class="r">Seating</th><th class="r">Allowable</th><th>Result</th></tr></thead><tbody>
        <For each={p.r.flanges}>{(f, k) => { const blind = (p.v.flanges ?? [])[k()]?.blind; const q: Q = blind ? "len" : "stress"; return <tr><td>{f.name}{blind ? " (cover, thickness)" : " (ring, stress)"}</td><td class="r">{v4("len", f.g)}</td><td class="r">{v4("force", f.wm1)}</td><td class="r">{v4("force", f.wm2)}</td><td class="r">{v4("area", f.am)}</td><td class="r" classList={{ bad: !f.bolts_pass }}>{v4("area", f.ab)}</td><td class="r">{v4(q, f.op)}</td><td class="r">{v4(q, f.seat)}</td><td class="r">{blind ? `${v4("len", f.op_allow)} given` : `${v4("stress", f.op_allow)} / ${v4("stress", f.seat_allow)}`}</td><td><Badge pass={f.pass} /></td></tr>; }}</For>
      </tbody></table>
      <p class="muted small">The bolted flange method: the gasket's effective width b and diameter G, the operating bolt load Wm1 = H + Hp (pressure end force and gasket compression m P) and the seating load Wm2 = π b G y; the bolts must give the larger area. A ring flange's tangential stress Y Mo / (t² B) from the moments of the end force, the face force and the gasket load about the bolt circle; a blind cover's thickness G √(0.3 P/S + 1.9 W hG/(S G³)). The gasket factors m and y and the allowables are your inputs.</p>
    </Show>
    <Show when={(results()?.vessel_links ?? []).some(x => x.vessel === p.v.name)}>
      <h4>Piping loads on the nozzles{stale() ? " (the model changed: analyze again)" : ""}</h4>
      <table class="tbl rtable"><thead><tr><th>Nozzle</th><th>Case</th><th class="r">Axial {unit("force")}</th><th class="r">Torsion {unit("moment")}</th><th class="r">Bending</th><th class="r">Neck stress {unit("stress")}</th><th class="r">1.5 S</th><th class="r">Ratio</th><th>Result</th></tr></thead><tbody>
        <For each={(results()?.vessel_links ?? []).filter(x => x.vessel === p.v.name)}>{x => <tr><td>{x.nozzle}</td><td>{x.case}</td><td class="r">{v4("force", x.local[0])}</td><td class="r">{v4("moment", x.local[3])}</td><td class="r">{v4("moment", Math.hypot(x.local[4], x.local[5]))}</td><td class="r">{v4("stress", x.stress)}</td><td class="r">{v4("stress", x.allow)}</td><td class="r">{num(x.ratio, 3)}</td><td><Badge pass={x.pass} /></td></tr>}</For>
      </tbody></table>
      <p class="muted small">From the piping analysis: the loads at the linked nozzle points, on the neck's corroded section (pressure P d/4t, axial force, bending and torsion by the maximum shear stress) against 1.5 S. The local stresses in the vessel wall are for the vessel engineer: the loads above in the nozzle's axes are what they need.</p>
    </Show>
    <Show when={(p.r.lifting ?? []).length + (p.r.legs ?? []).length}>
      <h4>Lifting and legs</h4>
      <table class="tbl rtable"><thead><tr><th>Check</th><th class="r">Stress {unit("stress")}</th><th class="r">Allowable</th><th class="r">Ratio</th><th>Result</th></tr></thead><tbody>
        <For each={[...(p.r.lifting ?? []), ...(p.r.legs ?? [])]}>{x => <tr><td>{x.what}</td><td class="r">{x.what.includes("combined") ? num(x.value, 3) : v4("stress", x.value)}</td><td class="r">{x.what.includes("combined") ? "1" : v4("stress", x.allow)}</td><td class="r">{num(x.ratio, 3)}</td><td><Badge pass={x.pass} /></td></tr>}</For>
      </tbody></table>
      <p class="muted small">Lifting: the empty weight × the impact factor, shared by the lugs along the sling (or by two trunnions); allowables 0.6 Sy in tension, 0.4 Sy in shear and on the welds, 0.9 Sy in bearing, 0.66 Sy in bending. Legs: each carries the weight over the legs plus 4M/(n D) from the overturning moment, against Euler buckling of a cantilever leg (K = 2) with a factor of 2 and 0.6 Sy, plus the base shear's bending.</p>
    </Show>
    <Show when={p.r.support}>{_ => { const su = p.r.support!; return <>
      <h4>Wind, seismic and the skirt</h4>
      <p class="muted small">Base shear: wind {v4("force", su.shear_wind)}, seismic {v4("force", su.shear_seis)} {unit("force")}; base moment: wind {v4("moment", su.moment_wind)}, seismic {v4("moment", su.moment_seis)} {unit("moment")} (the larger is used). Weight operating {v4("force", su.weight_op)}, empty {v4("force", su.weight_empty)} {unit("force")}.</p>
      <table class="tbl rtable"><thead><tr><th>Section</th><th class="r">Moment, wind {unit("moment")}</th><th class="r">Moment, seismic</th><th class="r">Weight above {unit("force")}</th><th class="r">Tension {unit("stress")}</th><th class="r">Allowable</th><th class="r">Compression</th><th class="r">Allowable</th><th>Result</th></tr></thead><tbody>
        <For each={su.sections}>{x => <tr><td>{x.name}</td><td class="r">{v4("moment", x.m_wind)}</td><td class="r">{v4("moment", x.m_seis)}</td><td class="r">{v4("force", x.weight)}</td><td class="r">{v4("stress", x.tension)}</td><td class="r">{v4("stress", x.allow_t)}</td><td class="r">{v4("stress", x.compression)}</td><td class="r">{v4("stress", x.allow_c)}</td><td><Badge pass={x.pass} /></td></tr>}</For>
        <Show when={su.bolt_allow > 0}><tr><td>Anchor bolts ({v4("force", su.bolt_load)} {unit("force")} each)</td><td colSpan={3} /><td class="r">{v4("stress", su.bolt_stress)}</td><td class="r">{v4("stress", su.bolt_allow)}</td><td colSpan={2} /><td><Badge pass={su.bolt_stress <= su.bolt_allow * (1 + 1e-9)} /></td></tr></Show>
        <Show when={su.bearing_allow > 0}><tr><td>Foundation bearing</td><td colSpan={5} /><td class="r">{v4("stress", su.bearing)}</td><td class="r">{v4("stress", su.bearing_allow)}</td><td><Badge pass={su.bearing <= su.bearing_allow * (1 + 1e-9)} /></td></tr></Show>
        <Show when={su.ring_t > 0}><tr><td>Base ring thickness {unit("len")}</td><td colSpan={3} /><td class="r">{v4("len", su.ring_t_req)} needed</td><td class="r">{v4("len", su.ring_t)}</td><td colSpan={2} /><td><Badge pass={su.ring_t >= su.ring_t_req - 1e-12} /></td></tr></Show>
      </tbody></table>
      <p class="muted small">Statics of the vessel as a cantilever from its base: wind = the site's pressure (Analysis settings) × Cd × the outside diameter × the factor for ladders and platforms; seismic = the coefficient × the mass at each height. Shell courses: pressure plus bending less weight in tension; bending and weight in compression against min(S, 0.0625 E t / R) (axial buckling with a knockdown). The skirt carries no pressure. Bolts 4M/(N BC) − W/N; bearing W/A + M/Z; ring l √(3 σ / S).</p>
    </>; }}</Show>
    <Show when={p.r.saddles}>{_ => { const sd = p.r.saddles!; return <>
      <h4>Saddles</h4>
      <p class="muted small">Load on each saddle {v4("force", sd.q)} {unit("force")}; moment at the saddles {v4("moment", sd.m1)}, at mid-span {v4("moment", sd.m2)} {unit("moment")}; constants K1 {num(sd.k[0], 3)}, K1* {num(sd.k[1], 3)}, K2 {num(sd.k[2], 3)}, K3 {num(sd.k[3], 3)}, K5 {num(sd.k[4], 3)}, K6 {num(sd.k[5], 4)}.</p>
      <table class="tbl rtable"><thead><tr><th>Stress</th><th class="r">Stress {unit("stress")}</th><th class="r">Allowable</th><th class="r">Ratio</th><th>Result</th></tr></thead><tbody>
        <For each={sd.rows}>{x => <tr><td>{x.what}</td><td class="r">{v4("stress", x.stress)}</td><td class="r">{v4("stress", x.allow)}</td><td class="r">{num(x.ratio, 3)}</td><td><Badge pass={x.pass} /></td></tr>}</For>
      </tbody></table>
      <p class="muted small">Zick's method (1951): the vessel as a beam on two saddles carrying the heavier of its operating weight and its weight full of water, the shell at the design pressure, hot and corroded. Allowables: S for the longitudinal stresses, 0.8 S for shear, 1.25 S at the horn, 0.5 Sy at the bottom.</p>
    </>; }}</Show>
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

const NozzlesTable = (p: { i: number; v: Vessel }) => {
  const [lib] = createResource(library);
  const list = () => p.v.nozzles ?? [];
  const set = (l: VNozzle[]) => putV(p.i, { nozzles: l });
  const put = (j: number, x: Partial<VNozzle>) => set(list().map((q, k) => k === j ? { ...q, ...x } : q));
  const sizes = () => (lib() as Library | undefined)?.sizes ?? [];
  const schedOf = (z: VNozzle) => sizes().find(s => s.nps === z.nps)?.schedules.find(sc => Math.abs(sc[1] - z.t) < 1e-6)?.[0] ?? "";
  const setSize = (j: number, nps: string, sched: string) => { const s = sizes().find(x => x.nps === nps); const sc = s?.schedules.find(x => x[0] === sched) ?? s?.schedules.find(x => x[0] === "80") ?? s?.schedules[0]; if (s && sc) put(j, { nps, od: s.od, t: sc[1], pipe: true }); };
  const add = () => { const shell = Math.max(0, p.v.parts.findIndex(x => x.kind === "cylinder")); set([...list(), { name: `N${list().length + 1}`, part: shell, pos: (p.v.parts[shell]?.len ?? 2) / 2, angle: 0, nps: "4", od: 0.1143, t: 0.00856, pipe: true, leg: 0.008, pad_od: 0, pad_t: 0, pad_leg: 0, proj: 0.2 }]); };
  return (
    <>
      <table class="tbl vparts"><thead><tr><th>Nozzle</th><th>On</th><th>At ({unit("coord")})</th><th>Angle °</th><th>Size</th><th>Neck OD × t ({unit("len")})</th><th>Weld leg</th><th>Pad OD × t</th><th>Piping</th><th /></tr></thead><tbody>
        <For each={list()}>{(z, j) => <tr>
          <td><input style={{ width: "90px" }} value={z.name} onChange={e => put(j(), { name: e.currentTarget.value })} /></td>
          <td><select style={{ width: "110px" }} onChange={e => put(j(), { part: +e.currentTarget.value })}><For each={p.v.parts}>{(x, k) => <option value={k()} selected={k() === z.part}>{x.name}</option>}</For></select></td>
          <td title="Along the axis from the part's start (shells, cones), or the offset from the axis (heads)">{cell("coord", z.pos, pos => put(j(), { pos }), 64)}</td>
          <td><input class="mono" style={{ width: "44px" }} value={z.angle} onChange={e => put(j(), { angle: +e.currentTarget.value || 0 })} /></td>
          <td class="nowrap"><Show when={sizes().length} fallback={<span class="muted">…</span>}><select style={{ width: "62px" }} value={z.pipe ? z.nps : ""} onChange={e => setSize(j(), e.currentTarget.value, schedOf(z) || "80")}><option value="" selected={!z.pipe}>plate</option><For each={sizes()}>{s => <option value={s.nps} selected={z.pipe && s.nps === z.nps}>{s.nps}"</option>}</For></select>
            <Show when={z.pipe}><select style={{ width: "62px" }} value={schedOf(z)} onChange={e => setSize(j(), z.nps, e.currentTarget.value)}><For each={sizes().find(s => s.nps === z.nps)?.schedules ?? []}>{sc => <option value={sc[0]} selected={sc[0] === schedOf(z)}>{sc[0]}</option>}</For></select></Show></Show></td>
          <td class="nowrap">{cell("len", z.od, od => od > 2 * z.t ? put(j(), { od, pipe: false, nps: "" }) : toast("The outside diameter must be over twice the wall."), 58)} × {cell("len", z.t, t => t > 0 && 2 * t < z.od ? put(j(), { t, pipe: false, nps: "" }) : toast("The wall must be above 0 and under half the diameter."), 46)}</td>
          <td>{cell("len", z.leg, leg => leg >= 0 && put(j(), { leg }), 40)}</td>
          <td class="nowrap">{cell("len", z.pad_od, pad_od => pad_od >= 0 && put(j(), { pad_od, pad_leg: z.pad_leg || z.leg }), 52)} × {cell("len", z.pad_t, pad_t => pad_t >= 0 && put(j(), { pad_t }), 40)}</td>
          <td><select style={{ width: "100px" }} title="The piping model's nozzle point this nozzle carries" value={String(z.link ?? 0)} onChange={e => put(j(), { link: +e.currentTarget.value })}><option value="0">—</option><For each={model()?.nozzles ?? []}>{n => <option value={String(n.id)}>{n.name}</option>}</For></select></td>
          <td><button class="x" title="Remove this nozzle" onClick={() => set(list().filter((_, k) => k !== j()))}>×</button></td>
        </tr>}</For>
      </tbody></table>
      <p class="ibtns"><button class="btn sm" onClick={add}>Add a nozzle</button></p>
      <p class="muted small">Pick a pipe size and schedule for the neck (its wall counts at 87.5 %), or type the outside diameter and wall of a rolled-plate neck. Pad OD 0 = no pad.</p>
    </>
  );
};

const FlangesTable = (p: { i: number; v: Vessel }) => {
  const list = () => p.v.flanges ?? [];
  const set = (l: BodyFlange[]) => putV(p.i, { flanges: l });
  const put = (j: number, x: Partial<BodyFlange>) => set(list().map((q, k) => k === j ? { ...q, ...x } : q));
  const n = (j: number, k: keyof BodyFlange, q: Q, w = 54) => cell(q, list()[j][k] as number, x => x >= 0 ? put(j, { [k]: x } as any) : toast("Sizes and stresses are 0 or more."), w);
  return (
    <>
      <Show when={list().length}>
        <table class="tbl vparts"><thead><tr><th>Flange</th><th>Kind</th><th>A × B × C × t ({unit("len")})</th><th>Gasket OD × ID</th><th>m</th><th>y ({unit("stress")})</th><th>Bolts × root area ({unit("area")})</th><th>S bolts amb./des.</th><th>S flange amb./des.</th><th /></tr></thead><tbody>
          <For each={list()}>{(f, j) => <tr>
            <td><input style={{ width: "80px" }} value={f.name} onChange={e => put(j(), { name: e.currentTarget.value })} /></td>
            <td><select style={{ width: "110px" }} value={f.blind ? "blind" : "ring"} onChange={e => put(j(), { blind: e.currentTarget.value === "blind", b: e.currentTarget.value === "blind" ? 0 : (f.b || f.g_id * 0.9) })}><option value="blind" selected={f.blind}>Blind cover</option><option value="ring" selected={!f.blind}>Ring flange</option></select></td>
            <td class="nowrap">{n(j(), "a", "len")}×<Show when={!f.blind} fallback={<span class="muted"> — </span>}>{n(j(), "b", "len")}</Show>×{n(j(), "c", "len")}×{n(j(), "t", "len", 40)}</td>
            <td class="nowrap">{n(j(), "g_od", "len")}×{n(j(), "g_id", "len")}</td>
            <td><input class="mono" style={{ width: "36px" }} value={f.m} onChange={e => { const x = +e.currentTarget.value; if (x >= 0) put(j(), { m: x }); }} /></td>
            <td>{n(j(), "y", "stress", 44)}</td>
            <td class="nowrap"><input class="mono" style={{ width: "32px" }} value={f.bolts} onChange={e => { const x = Math.round(+e.currentTarget.value); if (x > 0) put(j(), { bolts: x }); }} />×{n(j(), "bolt_area", "area", 50)}</td>
            <td class="nowrap">{n(j(), "sa", "stress", 40)}/{n(j(), "sb", "stress", 40)}</td>
            <td class="nowrap">{n(j(), "sfa", "stress", 40)}/{n(j(), "sfo", "stress", 40)}</td>
            <td><button class="x" title="Remove this flange" onClick={() => set(list().filter((_, k) => k !== j()))}>×</button></td>
          </tr>}</For>
        </tbody></table>
      </Show>
      <p class="ibtns"><button class="btn sm" onClick={() => set([...list(), { name: `F${list().length + 1}`, blind: true, a: 0.775, b: 0, c: 0.686, t: 0.05, g_od: 0.584, g_id: 0.54, m: 2, y: 11e6, bolts: 24, bolt_area: 599e-6, sa: 172e6, sb: 172e6, sfa: 138e6, sfo: 138e6 }])}>Add a flange or cover</button></p>
      <p class="muted small">Blind covers and ring (slip-on, lap-joint) flanges at the vessel's design pressure. Example values: take the dimensions from the flange standard and the gasket factors m and y from the gasket maker.</p>
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
            <h4>Nozzles</h4>
            <NozzlesTable i={i()} v={v()!} />
            <h4>External pressure</h4>
            <div class="two"><F l="External design pressure (0 = none)" q="press" v={v()!.pe ?? 0} set={pe => pe >= 0 && putV(i(), { pe })} /><div class="iprow"><label>Design factor on collapse</label><input class="mono" value={v()!.ext_fs ?? 3} onChange={e => { const x = +e.currentTarget.value; if (x >= 1) putV(i(), { ext_fs: x }); else toast("The factor is 1 or more."); }} /></div></div>
            <div class="two"><F l="Stiffening ring spacing (0 = none)" q="coord" v={v()!.ring_spacing ?? 0} set={ring_spacing => ring_spacing >= 0 && putV(i(), { ring_spacing })} /><div class="iprow"><label>Ring moment of inertia (m⁴)</label><input class="mono" value={v()!.ring_i ?? 0} onChange={e => { const x = +e.currentTarget.value; if (x >= 0) putV(i(), { ring_i: x }); }} /></div></div>
            <p class="muted small">Full vacuum is 1.013 bar outside. The elastic modulus is under Wind, seismic and skirt (vertical vessels) and 195 GPa by default.</p>
            <h4>Bolted flanges and covers</h4>
            <FlangesTable i={i()} v={v()!} />
            <h4>Lifting</h4>
            <label class="chk"><input type="checkbox" checked={!!v()!.lifting} onChange={e => putV(i(), { lifting: e.currentTarget.checked ? { lugs: 2, t: 0.03, w: 0.25, h: 0.12, r: 0.075, hole: 0.055, pin: 0.05, weld: 0.012, sling: 30, tr_od: 0, tr_t: 0, tr_arm: 0, impact: 1.5, sy: 250e6 } : null })} />Lifting lugs and trunnions</label>
            <Show when={v()!.lifting}>{_ => { const l = () => v()!.lifting!; const pl = (x: Partial<NonNullable<Vessel["lifting"]>>) => putV(i(), { lifting: { ...l(), ...x } }); const num1 = (lab: string, k: keyof NonNullable<Vessel["lifting"]>) => <div class="iprow"><label>{lab}</label><input class="mono" value={l()[k] as number} onChange={e => { const x = +e.currentTarget.value; if (x >= 0) pl({ [k]: x } as any); }} /></div>;
              return <>
                <div class="two">{num1("Number of lugs", "lugs")}{num1("Sling angle from vertical °", "sling")}</div>
                <div class="two"><F l="Lug thickness" q="len" v={l().t} set={t => pl({ t })} /><F l="Lug width at the base" q="len" v={l().w} set={w => pl({ w })} /></div>
                <div class="two"><F l="Hole height above the base" q="len" v={l().h} set={h => pl({ h })} /><F l="Metal radius round the hole" q="len" v={l().r} set={r => pl({ r })} /></div>
                <div class="two"><F l="Hole diameter" q="len" v={l().hole} set={hole => pl({ hole })} /><F l="Pin diameter" q="len" v={l().pin} set={pin => pl({ pin })} /></div>
                <div class="two"><F l="Base fillet weld leg" q="len" v={l().weld} set={weld => pl({ weld })} /><F l="Yield strength (lugs, trunnions)" q="stress" v={l().sy} set={sy => sy > 0 && pl({ sy })} /></div>
                <div class="two"><F l="Trunnion OD (0 = none)" q="len" v={l().tr_od} set={tr_od => pl({ tr_od })} /><F l="Trunnion wall" q="len" v={l().tr_t} set={tr_t => pl({ tr_t })} /></div>
                <div class="two"><F l="Trunnion arm to the load" q="len" v={l().tr_arm} set={tr_arm => pl({ tr_arm })} />{num1("Impact factor", "impact")}</div>
              </>; }}</Show>
            <Show when={v()!.vertical}>
              <h4>Wind, seismic and skirt</h4>
              <div class="two"><div class="iprow"><label>Wind shape factor Cd</label><input class="mono" value={v()!.wind_cd ?? 0.7} onChange={e => { const x = +e.currentTarget.value; if (x >= 0) putV(i(), { wind_cd: x }); }} /></div><div class="iprow"><label>Diameter factor (ladders, platforms)</label><input class="mono" value={v()!.wind_dfac ?? 1.2} onChange={e => { const x = +e.currentTarget.value; if (x >= 1) putV(i(), { wind_dfac: x }); else toast("The factor is 1 or more."); }} /></div></div>
              <div class="two"><div class="iprow"><label>Seismic coefficient (g)</label><input class="mono" value={v()!.seis ?? 0} onChange={e => { const x = +e.currentTarget.value; if (x >= 0 && x <= 5) putV(i(), { seis: x }); }} /></div><F l="Base elevation (for the wind profile)" q="coord" v={v()!.base_el ?? 0} set={base_el => putV(i(), { base_el })} /></div>
              <div class="two"><F l="Elastic modulus (buckling)" q="modulus" v={v()!.emod ?? 195e9} set={emod => emod > 0 && putV(i(), { emod })} /><div /></div>
              <p class="muted small">The wind pressure (or profile) is the site's, in Analysis settings (select nothing on the Model view).</p>
              <label class="chk"><input type="checkbox" checked={!!v()!.legs} disabled={!!v()!.skirt} onChange={e => putV(i(), { legs: e.currentTarget.checked ? { n: 4, len: 1.5, area: 5.42e-3, z: 1.89e-4, rg: 0.0559, circle: (v()!.parts.find(x => x.kind === "cylinder")?.d ?? 1.5) * 0.9, sy: 240e6 } : null })} />On legs</label>
              <Show when={v()!.legs}>{_ => { const g = () => v()!.legs!; const pg = (x: Partial<NonNullable<Vessel["legs"]>>) => putV(i(), { legs: { ...g(), ...x } });
                return <>
                  <div class="two"><div class="iprow"><label>Number of legs</label><input class="mono" value={g().n} onChange={e => { const x = Math.round(+e.currentTarget.value); if (x >= 3) pg({ n: x }); else toast("Three legs or more."); }} /></div><F l="Leg length (base to tangent)" q="coord" v={g().len} set={len => len > 0 && pg({ len })} /></div>
                  <div class="two"><F l="Leg circle" q="len" v={g().circle} set={circle => circle > 0 && pg({ circle })} /><F l="Leg yield strength" q="stress" v={g().sy} set={sy => sy > 0 && pg({ sy })} /></div>
                  <div class="two"><F l="Leg area" q="area" v={g().area} set={area => area > 0 && pg({ area })} /><F l="Radius of gyration" q="len" v={g().rg} set={rg => rg > 0 && pg({ rg })} /></div>
                  <div class="iprow"><label>Leg section modulus (m³)</label><input class="mono" value={g().z} onChange={e => { const x = +e.currentTarget.value; if (x > 0) pg({ z: x }); }} /></div>
                  <p class="muted small">Example values: four 6" Sch 80 pipe legs.</p>
                </>; }}</Show>
              <label class="chk"><input type="checkbox" checked={!!v()!.skirt} disabled={!!v()!.legs} onChange={e => { const shell = v()!.parts.find(x => x.kind === "cylinder"); const od = shell ? shell.d + 2 * shell.t : 1.5; putV(i(), { skirt: e.currentTarget.checked ? { h: 2, t: 0.012, ring_od: od + 0.4, ring_id: od - 0.25, ring_t: 0.025, bolts: 8, bolt_d: 0.0237, bc: od + 0.2, bolt_s: 140e6, bearing: 7e6 } : null }); }} />On a skirt</label>
              <Show when={v()!.skirt}>{_ => { const k = () => v()!.skirt!; const pk = (x: Partial<NonNullable<Vessel["skirt"]>>) => putV(i(), { skirt: { ...k(), ...x } });
                return <>
                  <div class="two"><F l="Skirt height (base to tangent)" q="coord" v={k().h} set={h => h > 0 && pk({ h })} /><F l="Skirt thickness" q="len" v={k().t} set={t => t > 0 && pk({ t })} /></div>
                  <div class="two"><F l="Base ring outside diameter" q="len" v={k().ring_od} set={ring_od => ring_od > k().ring_id && pk({ ring_od })} /><F l="Base ring inside diameter" q="len" v={k().ring_id} set={ring_id => ring_id >= 0 && ring_id < k().ring_od && pk({ ring_id })} /></div>
                  <div class="two"><F l="Base ring thickness" q="len" v={k().ring_t} set={ring_t => ring_t >= 0 && pk({ ring_t })} /><div class="iprow"><label>Anchor bolts</label><input class="mono" value={k().bolts} onChange={e => { const x = Math.round(+e.currentTarget.value); if (x >= 0) pk({ bolts: x }); }} /></div></div>
                  <div class="two"><F l="Bolt root diameter" q="len" v={k().bolt_d} set={bolt_d => bolt_d >= 0 && pk({ bolt_d })} /><F l="Bolt circle" q="len" v={k().bc} set={bc => bc >= 0 && pk({ bc })} /></div>
                  <div class="two"><F l="Bolt allowable stress" q="stress" v={k().bolt_s} set={bolt_s => bolt_s >= 0 && pk({ bolt_s })} /><F l="Foundation allowable bearing" q="stress" v={k().bearing} set={bearing => bearing >= 0 && pk({ bearing })} /></div>
                </>; }}</Show>
            </Show>
            <Show when={!v()!.vertical}>
              <h4>Saddles</h4>
              <label class="chk"><input type="checkbox" checked={!!v()!.saddles} onChange={e => putV(i(), { saddles: e.currentTarget.checked ? { a: Math.min(0.6, (v()!.parts.find(p => p.kind === "cylinder")?.d ?? 1.2) / 4), theta: 120, b: 0.3, welded: true, rings: false } : null })} />On two saddles (Zick's method)</label>
              <Show when={v()!.saddles}>{_ => { const sd = () => v()!.saddles!; const ps = (x: Partial<NonNullable<Vessel["saddles"]>>) => putV(i(), { saddles: { ...sd(), ...x } });
                return <>
                  <div class="two"><F l="Tangent line to saddle centre (A)" q="coord" v={sd().a} set={a => a >= 0 && ps({ a })} /><F l="Saddle width" q="len" v={sd().b} set={b => b > 0 && ps({ b })} /></div>
                  <div class="two"><div class="iprow"><label>Contact angle °</label><input class="mono" value={sd().theta} onChange={e => { const x = +e.currentTarget.value; if (x >= 120 && x <= 180) ps({ theta: x }); else { e.currentTarget.value = String(sd().theta); toast("The contact angle is from 120° to 180°."); } }} /></div><div /></div>
                  <label class="chk"><input type="checkbox" checked={sd().welded} onChange={e => ps({ welded: e.currentTarget.checked })} />Saddles welded to the shell</label>
                  <label class="chk"><input type="checkbox" checked={sd().rings} onChange={e => ps({ rings: e.currentTarget.checked })} />Stiffening rings at the saddles (rings not checked)</label>
                  <p class="muted small">A up to R/2 lets the heads stiffen the shell at the saddles.</p>
                </>; }}</Show>
            </Show>
            <p class="ibtns"><button class="btn sm" onClick={() => { setList(list().filter((_, k) => k !== i())); setVesselNo(Math.max(0, i() - 1)); }}>Delete this vessel</button></p>
          </div>
          <div class="vright">
            <Show when={chk()?.result} fallback={<p class="warnline">{chk()?.error ?? "Checking…"}</p>}>{_ => <VesselDrawing v={v()!} r={chk()!.result!} w={v()!.vertical ? 320 : 560} h={v()!.vertical ? 480 : 320} />}</Show>
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
