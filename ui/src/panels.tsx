// The Inspector: the selected element, node, support, nozzle or load, in display units; with
// nothing selected, the model's name, title block and analysis settings.
import { For, Show, Switch, Match, JSX } from "solid-js";
import { Element, Support, Nozzle, Load, SupKind, V3, HarmonicLoad, num } from "./api";
import { model, sel, setSel, act, toast } from "./store";
import { C, SUPPORTS } from "./cmds";
import { Help } from "./help";
import { F, T, numIn, numInQ } from "./props";
import * as P from "./props";
import * as Tools from "./tools";
import { toU, fromU, unit, fmt, Q } from "./units";
import { compName, COMPS, compKey, compFrom } from "./table";

export const SUP_NAMES: Record<string, string> = { anchor: "Anchor", rest: "Rest (+Y)", guide: "Guide", linestop: "Line stop", restraint: "Restraint", spring: "Spring", rotspring: "Rotational spring", constant: "Constant-force support", hanger: "Spring hanger" };
export const supName = (k: SupKind) => k.kind === "restraint" ? `Restraint ${dirName(k.dir)}` : SUP_NAMES[k.kind];
export const dirName = (d: V3) => { const n = Math.hypot(...d); if (n === 0) return ""; const u = d.map(v => v / n); const ax = ["X", "Y", "Z"].findIndex((_, i) => Math.abs(Math.abs(u[i]) - 1) < 1e-9); return ax >= 0 ? `${u[ax] < 0 ? "−" : ""}${"XYZ"[ax]}` : `(${u.map(v => num(v, 3)).join(", ")})`; };

const Sec = (p: { t: string; h?: string; children: JSX.Element }) => <><h4>{p.t}{p.h && <Help k={p.h} />}</h4>{p.children}</>;
const V3In = (p: { l: string; q: "force" | "moment" | "disp" | "rot" | "ratio"; v: V3; set: (v: V3) => void }) => (
  <div class="iprow"><label>{p.l}<span class="unit">{p.q === "ratio" ? "" : unit(p.q)}</span></label>
    <div class="v3"><For each={[0, 1, 2]}>{i => <input class="mono" title={"XYZ"[i]} value={num(p.q === "ratio" ? p.v[i] : toU(p.q, p.v[i]), 6).replace("−", "-")} onChange={(p.q === "ratio" ? numIn : (f: (v: number) => unknown, sh: () => string) => numInQ(p.q as Q, f, sh))(v => p.set(p.v.map((x, k) => k === i ? (p.q === "ratio" ? v : fromU(p.q, v)) : x) as V3), () => num(p.q === "ratio" ? p.v[i] : toU(p.q, p.v[i]), 6))} />}</For></div></div>
);

const ElementPanel = (p: { e: Element }) => {
  const m = () => model()!;
  const e = () => p.e;
  const put = (x: Partial<Element>) => act({ Set: { item: { Element: { ...e(), ...x } } } });
  const pipe = () => m().pipes.find(x => x.id === e().pipe);
  return <>
    <p class="iwhat"><b>{e().from} → {e().to}</b> · {fmt("coord", Math.hypot(...e().d), 5)} · {compName(e().comp)}</p>
    <Sec t="Length" h="table"><V3In l="DX, DY, DZ" q="disp" v={e().d} set={d => put({ d })} /></Sec>
    <Sec t="Component" h="components">
      <div class="iprow"><select onChange={ev => put({ comp: compFrom(ev.currentTarget.value, e(), m()) })}><For each={COMPS}>{c => <option value={c[0]} selected={c[0] === compKey(e().comp)}>{c[1]}</option>}</For></select></div>
      <Show when={e().comp.kind === "bend" && (e().comp as any).bend === "custom"}><F l="Bend radius" q="radius" v={(e().comp as any).r} set={r => put({ comp: { ...(e().comp as any), r } })} /></Show>
      <Show when={e().comp.kind === "bend"}><p class="muted small">The bend is at node {e().to}, turning into the element that leaves it.</p></Show>
      <Show when={e().comp.kind === "tee" && (e().comp as any).tee === "reinforced"}><F l="Pad thickness" q="len" v={(e().comp as any).pad} set={pad => put({ comp: { ...(e().comp as any), pad } })} /></Show>
      <Show when={e().comp.kind === "tee"}><p class="muted small">The tee is at node {e().to}, where three elements meet.</p></Show>
      <Show when={e().comp.kind === "reducer"}><div class="iprow"><label>Small end</label><select onChange={ev => put({ comp: { kind: "reducer", to: +ev.currentTarget.value } })}><For each={m().pipes}>{x => <option value={x.id} selected={x.id === (e().comp as any).to}>{x.name}</option>}</For></select></div></Show>
      <Show when={e().comp.kind === "rigid"}><F l="Weight (without contents)" q="mass" v={(e().comp as any).mass} set={mass => put({ comp: { ...(e().comp as any), mass } })} /></Show>
      <Show when={e().comp.kind === "bellows"}>{_ => { const b = () => e().comp as any; const pb = (x: any) => { if (Object.values(x).some(v => typeof v === "number" && v < 0)) return toast("The rates, area and weight must be 0 or more."); put({ comp: { ...b(), ...x } }); };
        return <>
          <div class="two"><F l="Axial rate" q="tstiff" v={b().axial} set={axial => pb({ axial })} /><F l="Lateral rate" q="tstiff" v={b().lateral} set={lateral => pb({ lateral })} /></div>
          <div class="two"><F l="Angular rate" q="rstiff" v={b().angular} set={angular => pb({ angular })} /><F l="Torsional rate" q="rstiff" v={b().torsion} set={torsion => pb({ torsion })} /></div>
          <div class="two"><F l="Effective area" q="area" v={b().area} set={area => pb({ area })} /><F l="Weight (without contents)" q="mass" v={b().mass} set={mass => pb({ mass })} /></div>
          <label class="chk"><input type="checkbox" checked={b().tied} onChange={ev => pb({ tied: ev.currentTarget.checked })} />Tied (tie rods carry the pressure thrust)</label>
          <p class="muted small">The maker's data; 0 is rigid. An area of 0 takes the pipe's outside diameter. Untied, the pressure on the effective area pushes the pipe apart: the main anchors must carry it.</p>
        </>; }}</Show>
    </Sec>
    <Sec t="Properties" h="pipes">
      <div class="iprow"><label>Pipe</label><div class="inline"><select onChange={ev => put({ pipe: +ev.currentTarget.value })}><For each={m().pipes}>{x => <option value={x.id} selected={x.id === e().pipe}>{x.name}</option>}</For></select><button class="btn sm" onClick={() => P.pipes(e().pipe)}>Edit</button></div></div>
      <div class="iprow"><label>Material</label><div class="inline"><select onChange={ev => put({ material: +ev.currentTarget.value })}><For each={m().materials}>{x => <option value={x.id} selected={x.id === e().material}>{x.name}</option>}</For></select><button class="btn sm" onClick={() => P.materials(e().material)}>Edit</button></div></div>
      <div class="iprow"><label>Line conditions</label><div class="inline"><select onChange={ev => put({ cond: +ev.currentTarget.value })}><For each={m().conds}>{x => <option value={x.id} selected={x.id === e().cond}>{x.name}</option>}</For></select><button class="btn sm" onClick={() => P.conds(e().cond)}>Edit</button></div></div>
      <F l="Extra uniform load (down)" q="wlen" v={e().extra} set={extra => put({ extra })} hint="A walkway, ice or a cable tray carried by the pipe." />
      <Show when={e().comp.kind === "pipe"}><F l="Cold spring (cut short by)" q="len" v={e().cold ?? 0} set={cold => put({ cold })} hint="The pipe is made this much short and pulled together at installation. Adds the case COLD; the expansion range is unchanged." /></Show>
      <Show when={pipe()}>{pp => <table class="vmini"><tbody><tr><td>Outside diameter</td><td class="r">{fmt("len", pp().od)}</td></tr><tr><td>Wall</td><td class="r">{fmt("len", pp().wall)}</td></tr></tbody></table>}</Show>
    </Sec>
    <p class="ibtns"><button class="btn sm" onClick={() => setSel({ kind: "node", id: e().from })}>Node {e().from}</button><button class="btn sm" onClick={() => setSel({ kind: "node", id: e().to })}>Node {e().to}</button><button class="btn sm" onClick={C.split}>Insert node</button><Show when={e().comp.kind === "pipe"}><button class="btn sm" onClick={() => Tools.loopWizard()}>Expansion loop</button></Show><button class="btn sm" onClick={C.del}>Delete</button></p>
  </>;
};

const NodePanel = (p: { num: number }) => {
  const m = () => model()!;
  const n = () => m().nodes.find(x => x.num === p.num);
  const on = <X extends { node: number }>(l: X[]) => l.filter(x => x.node === p.num);
  const root = () => m().elements.find(e => e.from === p.num || e.to === p.num) && m().nodes[0]?.num === p.num;
  return <Show when={n()}>{nn => <>
    <p class="iwhat"><b>Node {p.num}</b></p>
    <Sec t="Position" h="table">
      <table class="vmini"><tbody><For each={["X", "Y", "Z"]}>{(a, i) => <tr><td>{a}</td><td class="r">{fmt("coord", [nn().x, nn().y, nn().z][i()], 6)}</td></tr>}</For></tbody></table>
      <Show when={root()}><V3In l="Start point (moves the whole run)" q="disp" v={[nn().x, nn().y, nn().z]} set={pp => act({ SetStart: { num: p.num, p: pp } })} /></Show>
    </Sec>
    <Sec t="Supports" h="supports">
      <For each={on(m().supports)} fallback={<p class="muted small">None. Add one from the Supports tab.</p>}>{s => <button class="ti" onClick={() => setSel({ kind: "support", id: s.id })}><span>{supName(s.kind)}</span><small>{s.tag}</small></button>}</For>
      <div class="iprow"><select onChange={ev => { C.support(ev.currentTarget.value); ev.currentTarget.value = ""; }}><option value="">Add a support…</option><For each={SUPPORTS}>{s => <option value={s[0]}>{s[1]}</option>}</For></select></div>
    </Sec>
    <Sec t="Nozzle and loads" h="nozzles">
      <For each={on(m().nozzles)}>{z => <button class="ti" onClick={() => setSel({ kind: "nozzle", id: z.id })}><span>{z.name}</span><small>nozzle</small></button>}</For>
      <For each={on(m().loads)}>{l => <button class="ti" onClick={() => setSel({ kind: "load", id: l.id })}><span>Force {fmt("force", Math.hypot(...l.f))}</span><small>load</small></button>}</For>
      <p class="ibtns"><Show when={!on(m().nozzles).length}><button class="btn sm" onClick={C.nozzle}>Nozzle point</button></Show><button class="btn sm" onClick={C.load}>Force or moment</button></p>
    </Sec>
  </>}</Show>;
};

const SupportPanel = (p: { s: Support }) => {
  const s = () => p.s;
  const put = (x: Partial<Support>) => act({ Set: { item: { Support: { ...s(), ...x } } } });
  const k = () => s().kind as any;
  const dirKinds = ["restraint", "spring", "rotspring"];
  return <>
    <p class="iwhat"><b>{supName(s().kind)}</b> at node {s().node}</p>
    <T l="Tag" v={s().tag} set={tag => put({ tag })} />
    <Show when={dirKinds.includes(k().kind)}><V3In l="Direction (X, Y, Z)" q="ratio" v={k().dir} set={dir => put({ kind: { ...k(), dir } })} /></Show>
    <Show when={k().kind === "spring"}><F l="Spring rate" q="tstiff" v={k().k} set={kk => put({ kind: { ...k(), k: kk } })} /></Show>
    <Show when={k().kind === "rotspring"}><F l="Rotational stiffness" q="rstiff" v={k().k} set={kk => put({ kind: { ...k(), k: kk } })} /></Show>
    <Show when={k().kind === "constant"}><F l="Load carried (up)" q="force" v={k().load} set={load => put({ kind: { ...k(), load } })} /></Show>
    <Show when={!["spring", "rotspring", "constant", "hanger"].includes(k().kind)}><F l="Stiffness (0 = rigid)" q="tstiff" v={s().k} set={kk => put({ k: kk })} /></Show>
    <Show when={k().kind === "hanger"}><p class="muted small">Sized by Analyze from the spring table (Supports › Spring Table): its hot load with the pipe held level here, its travel in operating case 1, and the first spring whose range takes both loads with the load variation within the limit. Results › Hangers shows the size, rate and cold load to set it to.</p></Show>
    <Show when={k().kind === "rest"}><label class="chk"><input type="checkbox" checked={s().one_way} onChange={e => put({ one_way: e.currentTarget.checked })} />One-way: the pipe can lift off</label>
      <Show when={!s().one_way}><p class="muted small">Two-way: the rest also holds the pipe down.</p></Show></Show>
    <Show when={["rest", "guide", "linestop", "restraint"].includes(k().kind)}><F l={k().kind === "rest" ? "Gap below the pipe" : "Gap on each side"} q="disp" v={s().gap ?? 0} set={gap => put({ gap })} />
      <F l="Friction coefficient μ (0 = none)" q="ratio" v={s().mu ?? 0} set={mu => put({ mu })} />
      <p class="muted small">Typical μ: 0.3 steel on steel, 0.1 on PTFE slide plates. It acts across the support's direction.</p></Show>
    <Show when={k().kind === "anchor" || k().kind === "restraint"}><Sec t="Imposed displacements" h="disp">
      <For each={[0, 1, 2]}>{c => <><V3In l={`Case ${c + 1}: movement`} q="disp" v={s().disp[c].slice(0, 3) as V3} set={v => put({ disp: s().disp.map((d, i) => i === c ? [...v, ...d.slice(3)] : d) as any })} />
        <Show when={k().kind === "anchor"}><V3In l={`Case ${c + 1}: rotation`} q="rot" v={s().disp[c].slice(3) as V3} set={v => put({ disp: s().disp.map((d, i) => i === c ? [...d.slice(0, 3), ...v] : d) as any })} /></Show></>}</For>
      <p class="muted small">{k().kind === "anchor" ? "Equipment growth or settlement in each operating case." : "The part along the restraint's direction is used."}</p></Sec></Show>
    <p class="ibtns"><button class="btn sm" onClick={() => setSel({ kind: "node", id: s().node })}>Node {s().node}</button><button class="btn sm" onClick={C.del}>Delete</button></p>
  </>;
};

/** The response spectrum: acceleration (g) against frequency, the factor in each direction and the damping. */
const Spectrum = () => {
  const rows = () => model()?.settings.spectrum ?? [];
  const set = (spectrum: [number, number][]) => C.settings({ spectrum });
  const put = (i: number, k: 0 | 1, x: number) => { const r = rows().map(a => [...a] as [number, number]); r[i][k] = x; if (x < 0 || (k === 0 && x <= 0)) return toast("Frequencies above 0, accelerations 0 or more."); r.sort((a, b) => a[0] - b[0]); if (r.some((a, j) => j > 0 && a[0] === r[j - 1][0])) return toast("Each frequency once."); set(r); };
  const nIn = (v: number, f: (x: number) => void) => <input class="mono" style={{ width: "70px" }} value={num(v, 6)} onChange={e => { const x = +e.currentTarget.value; if (Number.isFinite(x)) f(x); }} />;
  return <>
    <Show when={rows().length}>
      <table class="tbl"><thead><tr><th>Frequency Hz</th><th>Acceleration g</th><th /></tr></thead><tbody>
        <For each={rows()}>{(r, i) => <tr><td>{nIn(r[0], x => put(i(), 0, x))}</td><td>{nIn(r[1], x => put(i(), 1, x))}</td><td><button class="x" title="Remove this row" onClick={() => set(rows().filter((_, k) => k !== i()))}>×</button></td></tr>}</For>
      </tbody></table>
      <V3In l="Spectrum factor in X, Y, Z (0 = not in that direction)" q="ratio" v={model()!.settings.rs_dir ?? [0, 0, 0]} set={rs_dir => C.settings({ rs_dir })} />
    </Show>
    <p class="ibtns"><button class="btn sm" onClick={() => { const r = rows(); set(r.length ? [...r, [r[r.length - 1][0] * 2, r[r.length - 1][1]]] : [[1, 0.2], [10, 0.4], [33, 0.15]]); if (!r.length) C.settings({ spectrum: [[1, 0.2], [10, 0.4], [33, 0.15]], rs_dir: [1, 0, 1] }); }}>{rows().length ? "Add a frequency" : "Response spectrum"}</button></p>
  </>;
};

/** A harmonic load: a force and moment at one node at one frequency, solved by the modes. */
const Harmonic = () => {
  const h = () => model()?.settings.harmonic ?? null;
  const put = (x: Partial<HarmonicLoad>) => C.settings({ harmonic: { ...h()!, ...x } });
  return <>
    <Show when={h()}>{hh => <>
      <div class="iprow"><label>Harmonic load at node</label><input class="mono" id="harm-node" value={hh().node} onChange={e => { const x = +e.currentTarget.value; if (model()!.nodes.some(n => n.num === x)) put({ node: x }); else { toast("No node " + e.currentTarget.value + "."); e.currentTarget.value = String(hh().node); } }} /></div>
      <V3In l="Force amplitude" q="force" v={hh().f} set={f => put({ f })} />
      <V3In l="Moment amplitude" q="moment" v={hh().m} set={m => put({ m })} />
      <div class="iprow"><label>Frequency (Hz)</label><input class="mono" id="harm-freq" value={num(hh().freq, 6)} onChange={e => { const x = +e.currentTarget.value; if (x > 0 && Number.isFinite(x)) put({ freq: x }); else { toast("The frequency is above 0."); e.currentTarget.value = num(hh().freq, 6); } }} /></div>
      <p class="muted small">Gives case HARM: the amplitudes of movement, forces and support loads, with the damping ratio above, from the modes found.</p>
    </>}</Show>
    <p class="ibtns"><Show when={h()} fallback={<button class="btn sm" onClick={() => C.settings({ harmonic: { node: model()!.nodes[model()!.nodes.length - 1]?.num ?? 10, f: [0, 1000, 0], m: [0, 0, 0], freq: 10 } })}>Harmonic load</button>}><button class="btn sm" onClick={() => C.settings({ harmonic: null })}>Remove the harmonic load</button></Show></p>
  </>;
};

/** Wind pressure against elevation: rows of (elevation, pressure), from the wind standard for the site. */
const WindProfile = () => {
  const rows = () => model()?.settings.wind_profile ?? [];
  const set = (wind_profile: [number, number][]) => C.settings({ wind_profile });
  const cell = (q: Q, v: number, f: (x: number) => void) => <input class="mono" style={{ width: "86px" }} value={num(toU(q, v), 6)} onChange={numInQ(q, x => f(fromU(q, x)), () => num(toU(q, v), 6))} />;
  const put = (i: number, k: 0 | 1, x: number) => { const r = rows().map(a => [...a] as [number, number]); r[i][k] = x; if (k === 1 && x < 0) return toast("A pressure must be 0 or more."); r.sort((a, b) => a[0] - b[0]); if (r.some((a, j) => j > 0 && a[0] === r[j - 1][0])) return toast("Each elevation once."); set(r); };
  return <>
    <Show when={rows().length}>
      <table class="tbl"><thead><tr><th>Elevation Y ({unit("coord")})</th><th>Pressure ({unit("wpress")})</th><th /></tr></thead><tbody>
        <For each={rows()}>{(r, i) => <tr><td>{cell("coord", r[0], x => put(i(), 0, x))}</td><td>{cell("wpress", r[1], x => put(i(), 1, x))}</td><td><button class="x" title="Remove this row" onClick={() => set(rows().filter((_, k) => k !== i()))}>×</button></td></tr>}</For>
      </tbody></table>
      <p class="muted small">Wind pressure by elevation, interpolated between rows and held beyond them; each element takes its average.</p>
    </Show>
    <p class="ibtns"><button class="btn sm" onClick={() => { const r = rows(); const top = r.length ? r[r.length - 1] : null; set(top ? [...r, [top[0] + 10, top[1]]] : [[0, model()!.settings.wind_q || 800], [10, model()!.settings.wind_q || 800]]); }}>{rows().length ? "Add a height" : "Wind by height"}</button></p>
  </>;
};

const NozzlePanel = (p: { z: Nozzle }) => {
  const z = () => p.z;
  const put = (x: Partial<Nozzle>) => act({ Set: { item: { Nozzle: { ...z(), ...x } } } });
  const st = () => z().stiff ?? [0, 0, 0, 0, 0];
  const putS = (i: number, v: number) => { if (v < 0) return toast("A stiffness must be 0 (rigid) or more."); const s = [...st()] as [number, number, number, number, number]; s[i] = v; put({ stiff: s }); };
  return <>
    <p class="iwhat"><b>{z().name}</b> at node {z().node}</p>
    <T l="Name" v={z().name} set={name => put({ name })} />
    <V3In l="Nozzle axis (0, 0, 0 = along the pipe)" q="ratio" v={z().axis} set={axis => put({ axis })} />
    <Sec t="Allowable loads (0 = not checked)" h="nozzles">
      <div class="two"><F l="Axial force" q="force" v={z().allow[0]} set={v => put({ allow: [v, z().allow[1], z().allow[2], z().allow[3]] })} /><F l="Shear force" q="force" v={z().allow[1]} set={v => put({ allow: [z().allow[0], v, z().allow[2], z().allow[3]] })} /></div>
      <div class="two"><F l="Torsion" q="moment" v={z().allow[2]} set={v => put({ allow: [z().allow[0], z().allow[1], v, z().allow[3]] })} /><F l="Bending moment" q="moment" v={z().allow[3]} set={v => put({ allow: [z().allow[0], z().allow[1], z().allow[2], v] })} /></div>
    </Sec>
    <Sec t="Wall flexibility (0 = rigid)" h="nozzleflex">
      <div class="two"><F l="Axial (radial)" q="tstiff" v={st()[0]} set={v => putS(0, v)} /><F l="Shear" q="tstiff" v={st()[1]} set={v => putS(1, v)} /></div>
      <div class="two"><F l="Bending, long." q="rstiff" v={st()[3]} set={v => putS(3, v)} /><F l="Bending, circ." q="rstiff" v={st()[4]} set={v => putS(4, v)} /></div>
      <div class="two"><F l="Torsion" q="rstiff" v={st()[2]} set={v => putS(2, v)} /><div /></div>
      <V3In l="Vessel axis (0, 0, 0 = vertical)" q="ratio" v={z().vessel ?? [0, 0, 0]} set={vessel => put({ vessel })} />
      <p class="muted small">Bending in the longitudinal plane (of the nozzle and the vessel's axis) and the circumferential plane. Your values, from the vessel's or tank's maker or your own calculation. With any of them above 0, the anchor at this node holds along the nozzle's axes with these stiffnesses.</p>
    </Sec>
    <Show when={!(model()?.supports ?? []).some(s => s.node === z().node && s.kind.kind === "anchor")}><p class="warnline">Put an anchor at node {z().node} as well: the anchor carries the load, the nozzle reports it.</p></Show>
    <p class="ibtns"><button class="btn sm" onClick={() => setSel({ kind: "node", id: z().node })}>Node {z().node}</button><button class="btn sm" onClick={C.del}>Delete</button></p>
  </>;
};

const LoadPanel = (p: { l: Load }) => {
  const l = () => p.l;
  const put = (x: Partial<Load>) => act({ Set: { item: { Load: { ...l(), ...x } } } });
  return <>
    <p class="iwhat"><b>Force and moment</b> at node {l().node}: sustained, acting with the weight in every case.</p>
    <V3In l="Force FX, FY, FZ" q="force" v={l().f} set={f => put({ f })} />
    <V3In l="Moment MX, MY, MZ" q="moment" v={l().m} set={mm => put({ m: mm })} />
    <p class="ibtns"><button class="btn sm" onClick={() => setSel({ kind: "node", id: l().node })}>Node {l().node}</button><button class="btn sm" onClick={C.del}>Delete</button></p>
  </>;
};

const ModelPanel = () => {
  const m = () => model()!;
  const meta = (x: object) => act({ SetMeta: { name: m().name, notes: m().notes, project: m().project, ...x } });
  const proj = (k: keyof ReturnType<typeof m>["project"], v: string) => meta({ project: { ...m().project, [k]: v } });
  return <>
    <T l="Name" v={m().name} set={name => meta({ name })} />
    <div class="iprow"><label>Notes</label><textarea rows="2" value={m().notes} onChange={e => meta({ notes: e.currentTarget.value })} /></div>
    <Sec t="Title block" h="report">
      <div class="two"><T l="Project" v={m().project.project} set={v => proj("project", v)} /><T l="Line number" v={m().project.line} set={v => proj("line", v)} /></div>
      <div class="two"><T l="Calc number" v={m().project.calc} set={v => proj("calc", v)} /><T l="Revision" v={m().project.rev} set={v => proj("rev", v)} /></div>
      <div class="two"><T l="By" v={m().project.by} set={v => proj("by", v)} /><T l="Checked" v={m().project.checked} set={v => proj("checked", v)} /></div>
    </Sec>
    <Sec t="Analysis" h="settings">
      <F l="Installed temperature" q="temp" v={m().settings.ambient} set={ambient => C.settings({ ambient })} />
      <div class="iprow"><label>Displacement cycles over the plant's life</label><input class="mono" value={m().settings.cycles} onChange={numIn(v => C.settings({ cycles: v }), () => m().settings.cycles)} /><span class="hint">Up to 7000 cycles the allowable range factor f is 1.0.</span></div>
      <label class="chk"><input type="checkbox" checked={m().settings.liberal} onChange={e => C.settings({ liberal: e.currentTarget.checked })} />Liberal allowable stress range</label>
      <label class="chk"><input type="checkbox" checked={m().settings.bend_pressure} onChange={e => C.settings({ bend_pressure: e.currentTarget.checked })} />Pressure correction on bends</label>
      <label class="chk"><input type="checkbox" checked={m().settings.pressure_elongation} onChange={e => C.settings({ pressure_elongation: e.currentTarget.checked })} />Pressure elongation (straight pipe)</label>
      <label class="chk"><input type="checkbox" checked={m().settings.branch_ze} onChange={e => C.settings({ branch_ze: e.currentTarget.checked })} />Branch section modulus Ze at tees</label>
      <label class="chk"><input type="checkbox" checked={m().settings.hydro} onChange={e => C.settings({ hydro: e.currentTarget.checked })} />Hydrotest case</label>
      <div class="iprow"><label>Node number step</label><input class="mono" value={m().settings.step} onChange={numIn(v => v >= 1 && v % 1 === 0 && C.settings({ step: v }), () => m().settings.step)} /></div>
    </Sec>
    <Sec t="Occasional loads" h="occasional">
      <Show when={!(m().settings.wind_profile ?? []).length}><F l="Design wind pressure (0 = no wind)" q="wpress" v={m().settings.wind_q} set={wind_q => C.settings({ wind_q })} /></Show>
      <WindProfile />
      <F l="Shape factor Cd" q="ratio" v={m().settings.wind_cd} set={wind_cd => C.settings({ wind_cd })} />
      <V3In l="Wind blows towards (X, Y, Z; horizontal)" q="ratio" v={m().settings.wind_dir} set={wind_dir => C.settings({ wind_dir })} />
      <V3In l="Static seismic (g along X, Y, Z)" q="ratio" v={m().settings.seis} set={seis => C.settings({ seis })} />
      <label class="chk"><input type="checkbox" checked={!!m().settings.occ_both} onChange={e => C.settings({ occ_both: e.currentTarget.checked })} />Wind and seismic in both directions</label>
    </Sec>
    <Sec t="Dynamics" h="modes">
      <div class="iprow"><label>Natural modes to find (0 = none)</label><input class="mono" value={m().settings.modes ?? 0} onChange={numIn(v => v >= 0 && v <= 200 && v % 1 === 0 && C.settings({ modes: v }), () => m().settings.modes ?? 0)} /></div>
      <div class="iprow"><label>Damping ratio </label><input class="mono" id="damping" value={model()!.settings.damping ?? 0.05} onChange={e => { const x = +e.currentTarget.value; if (x > 0 && x < 1) C.settings({ damping: x }); else toast("The damping ratio is between 0 and 1."); }} /></div>
      <Spectrum />
      <Harmonic />
      <p class="muted small">Your inputs: take the wind pressure and seismic factors from the standard for the site. With either, OPE1 is solved with it added and an occasional case checks the sustained stress plus its own stress against 1.33 Sh.</p>
    </Sec>
  </>;
};

export const Inspector = () => {
  const m = () => model();
  const s = () => sel();
  const title = () => { const x = s(); return !x ? "Model" : x.kind === "element" ? "Element" : x.kind === "node" ? "Node" : x.kind === "support" ? "Support" : x.kind === "nozzle" ? "Nozzle" : "Load"; };
  return (
    <aside class="insp">
      <header>Inspector<Help k="inspector" /><span class="grow" /><b>{title()}</b></header>
      <div class="ibody">
        <Show when={m()}>
          <Switch fallback={<ModelPanel />}>
            <Match when={s()?.kind === "element" && m()!.elements.find(y => y.id === s()!.id)}>{e => <ElementPanel e={e()} />}</Match>
            <Match when={s()?.kind === "node" && s()!.id}>{n => <NodePanel num={n()} />}</Match>
            <Match when={s()?.kind === "support" && m()!.supports.find(y => y.id === s()!.id)}>{y => <SupportPanel s={y()} />}</Match>
            <Match when={s()?.kind === "nozzle" && m()!.nozzles.find(y => y.id === s()!.id)}>{y => <NozzlePanel z={y()} />}</Match>
            <Match when={s()?.kind === "load" && m()!.loads.find(y => y.id === s()!.id)}>{y => <LoadPanel l={y()} />}</Match>
          </Switch>
        </Show>
      </div>
    </aside>
  );
};
