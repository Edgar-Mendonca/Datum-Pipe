// The Inspector: the selected element, node, support, nozzle or load, in display units; with
// nothing selected, the model's name, title block and analysis settings.
import { For, Show, Switch, Match, JSX } from "solid-js";
import { Element, Support, Nozzle, Load, SupKind, V3, num } from "./api";
import { model, sel, setSel, act } from "./store";
import { C, SUPPORTS } from "./cmds";
import { Help } from "./help";
import { F, T, numIn } from "./props";
import * as P from "./props";
import * as Tools from "./tools";
import { toU, fromU, unit, fmt } from "./units";
import { compName, COMPS, compKey, compFrom } from "./table";

export const SUP_NAMES: Record<string, string> = { anchor: "Anchor", rest: "Rest (+Y)", guide: "Guide", linestop: "Line stop", restraint: "Restraint", spring: "Spring", rotspring: "Rotational spring", constant: "Constant-force support" };
export const supName = (k: SupKind) => k.kind === "restraint" ? `Restraint ${dirName(k.dir)}` : SUP_NAMES[k.kind];
export const dirName = (d: V3) => { const n = Math.hypot(...d); if (n === 0) return ""; const u = d.map(v => v / n); const ax = ["X", "Y", "Z"].findIndex((_, i) => Math.abs(Math.abs(u[i]) - 1) < 1e-9); return ax >= 0 ? `${u[ax] < 0 ? "−" : ""}${"XYZ"[ax]}` : `(${u.map(v => num(v, 3)).join(", ")})`; };

const Sec = (p: { t: string; h?: string; children: JSX.Element }) => <><h4>{p.t}{p.h && <Help k={p.h} />}</h4>{p.children}</>;
const V3In = (p: { l: string; q: "force" | "moment" | "disp" | "rot" | "ratio"; v: V3; set: (v: V3) => void }) => (
  <div class="iprow"><label>{p.l}<span class="unit">{p.q === "ratio" ? "" : unit(p.q)}</span></label>
    <div class="v3"><For each={[0, 1, 2]}>{i => <input class="mono" title={"XYZ"[i]} value={num(p.q === "ratio" ? p.v[i] : toU(p.q, p.v[i]), 6).replace("−", "-")} onChange={numIn(v => p.set(p.v.map((x, k) => k === i ? (p.q === "ratio" ? v : fromU(p.q, v)) : x) as V3), () => num(p.q === "ratio" ? p.v[i] : toU(p.q, p.v[i]), 6))} />}</For></div></div>
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
    </Sec>
    <Sec t="Properties" h="pipes">
      <div class="iprow"><label>Pipe</label><div class="inline"><select onChange={ev => put({ pipe: +ev.currentTarget.value })}><For each={m().pipes}>{x => <option value={x.id} selected={x.id === e().pipe}>{x.name}</option>}</For></select><button class="btn sm" onClick={() => P.pipes(e().pipe)}>Edit</button></div></div>
      <div class="iprow"><label>Material</label><div class="inline"><select onChange={ev => put({ material: +ev.currentTarget.value })}><For each={m().materials}>{x => <option value={x.id} selected={x.id === e().material}>{x.name}</option>}</For></select><button class="btn sm" onClick={() => P.materials(e().material)}>Edit</button></div></div>
      <div class="iprow"><label>Line conditions</label><div class="inline"><select onChange={ev => put({ cond: +ev.currentTarget.value })}><For each={m().conds}>{x => <option value={x.id} selected={x.id === e().cond}>{x.name}</option>}</For></select><button class="btn sm" onClick={() => P.conds(e().cond)}>Edit</button></div></div>
      <F l="Extra uniform load (down)" q="wlen" v={e().extra} set={extra => put({ extra })} hint="A walkway, ice or a cable tray carried by the pipe." />
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
    <Show when={!["spring", "rotspring", "constant"].includes(k().kind)}><F l="Stiffness (0 = rigid)" q="tstiff" v={s().k} set={kk => put({ k: kk })} /></Show>
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

const NozzlePanel = (p: { z: Nozzle }) => {
  const z = () => p.z;
  const put = (x: Partial<Nozzle>) => act({ Set: { item: { Nozzle: { ...z(), ...x } } } });
  return <>
    <p class="iwhat"><b>{z().name}</b> at node {z().node}</p>
    <T l="Name" v={z().name} set={name => put({ name })} />
    <V3In l="Nozzle axis (0, 0, 0 = along the pipe)" q="ratio" v={z().axis} set={axis => put({ axis })} />
    <Sec t="Allowable loads (0 = not checked)" h="nozzles">
      <div class="two"><F l="Axial force" q="force" v={z().allow[0]} set={v => put({ allow: [v, z().allow[1], z().allow[2], z().allow[3]] })} /><F l="Shear force" q="force" v={z().allow[1]} set={v => put({ allow: [z().allow[0], v, z().allow[2], z().allow[3]] })} /></div>
      <div class="two"><F l="Torsion" q="moment" v={z().allow[2]} set={v => put({ allow: [z().allow[0], z().allow[1], v, z().allow[3]] })} /><F l="Bending moment" q="moment" v={z().allow[3]} set={v => put({ allow: [z().allow[0], z().allow[1], z().allow[2], v] })} /></div>
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
