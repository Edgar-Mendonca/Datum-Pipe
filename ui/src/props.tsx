// The property managers: pipes (sizes, allowances, insulation, contents), materials (tables
// against temperature and allowables, with sources), line conditions (T1–T3, P1–P3, hydrotest)
// and the load cases. Each edit goes to the engine as one operation.
import { For, Show, createSignal, createResource, JSX } from "solid-js";
import { Pipe, Material, Cond, LoadCase, Library, Recipe, Check, SpringSize, num, parseNum } from "./api";
import { openDialog, act, call, model, toast } from "./store";
import { C } from "./cmds";
import { Dialog, Icon } from "./ui";
import { Q, toU, fromU, unit } from "./units";

let lib: Promise<Library | undefined> | null = null;
export const library = () => (lib ??= call<Library>("library"));

export const numIn = (f: (v: number) => unknown, shown: () => string | number) => (e: Event & { currentTarget: HTMLInputElement }) => {
  const v = parseNum(e.currentTarget.value);
  if (v === null || f(v) === false) e.currentTarget.value = String(shown());
};
const show = (q: Q, v: number) => num(toU(q, v), 6).replace("−", "-");

/** A number field shown in display units and stored in SI. */
export const F = (p: { l: string; q: Q; v: number; set: (v: number) => unknown; hint?: string }) => (
  <div class="iprow"><label>{p.l}<span class="unit">{unit(p.q)}</span></label>
    <input class="mono" value={show(p.q, p.v)} onChange={numIn(v => p.set(fromU(p.q, v)), () => show(p.q, p.v))} />
    {p.hint && <span class="hint">{p.hint}</span>}</div>
);
export const T = (p: { l: string; v: string; set: (v: string) => unknown }) => (
  <div class="iprow"><label>{p.l}</label><input value={p.v} onChange={e => p.set(e.currentTarget.value)} /></div>
);

/** A list on the left and a form on the right, with add and delete. */
const Manager = <X extends { id: number; name: string }>(p: { list: () => X[]; cur: () => number; setCur: (id: number) => void; add: () => void; del: () => void; extra?: JSX.Element; children: (x: X) => JSX.Element; used: (id: number) => number }) => (
  <div class="mgr">
    <div class="mlist"><For each={p.list()}>{x => <button classList={{ on: x.id === p.cur() }} onClick={() => p.setCur(x.id)}>{x.name}<small class="muted"> · {p.used(x.id)}</small></button>}</For>
      <p class="ibtns"><button class="btn sm" onClick={p.add}>Add a copy</button><button class="btn sm" onClick={p.del} disabled={p.list().length < 2 || p.used(p.cur()) > 0} title="Only what no element uses can be deleted">Delete</button></p>
      {p.extra}</div>
    <Show when={p.list().find(x => x.id === p.cur())}>{x => <div class="form">{p.children(x())}</div>}</Show>
  </div>
);

const usedBy = (k: "pipe" | "material" | "cond") => (id: number) => (model()?.elements ?? []).filter(e => e[k] === id).length;

export const pipes = (focus?: number) => openDialog(() => {
  const list = () => model()?.pipes ?? [];
  const [cur, setCur] = createSignal(focus ?? list()[0]?.id ?? 0);
  const [L] = createResource(library);
  const put = (x: Partial<Pipe>) => { const p = list().find(y => y.id === cur()); if (p) act({ Set: { item: { Pipe: { ...p, ...x } } } }); };
  const add = async () => { const p = list().find(y => y.id === cur()) ?? list()[0]; const o = await act({ Add: { item: { Pipe: { ...p, id: 0, name: `${p.name} (copy)` } }, after: null } }); if (o?.created.length) setCur(o.created[0]); };
  const pick = (nps: string, sched: string) => { const s = L()?.sizes.find(z => z.nps === nps); const w = s?.schedules.find(z => z[0] === sched) ?? s?.schedules.find(z => z[0] === "STD"); if (s && w) put({ nps, sched: w[0], od: s.od, wall: w[1], name: `${nps}" Sch ${w[0]}` }); };
  return (
    <Dialog title="Pipes" wide help="pipes">
      <Manager list={list} cur={cur} setCur={setCur} add={add} del={() => act({ Delete: { ids: [cur()] } }).then(o => o && setCur(list()[0]?.id ?? 0))} used={usedBy("pipe")}
        extra={<p class="muted small">{L()?.source}.</p>}>{p => <>
        <T l="Name" v={p.name} set={name => put({ name })} />
        <div class="iprow"><label>Nominal size and schedule</label><span class="inline">
          <select onChange={e => pick(e.currentTarget.value, p.sched || "STD")}><option value="" selected={!p.nps}>Own size</option><For each={L()?.sizes ?? []}>{s => <option value={s.nps} selected={s.nps === p.nps}>{s.nps}"</option>}</For></select>
          <select disabled={!p.nps} onChange={e => pick(p.nps, e.currentTarget.value)}><For each={L()?.sizes.find(s => s.nps === p.nps)?.schedules ?? []}>{s => <option value={s[0]} selected={s[0] === p.sched}>Sch {s[0]} · {num(toU("len", s[1]), 4)} {unit("len")}</option>}</For></select></span></div>
        <div class="two"><F l="Outside diameter" q="len" v={p.od} set={od => put({ od, nps: "", sched: "" })} /><F l="Nominal wall" q="len" v={p.wall} set={wall => put({ wall, nps: "", sched: "" })} /></div>
        <div class="two"><F l="Corrosion allowance" q="len" v={p.corrosion} set={corrosion => put({ corrosion })} />
          <div class="iprow"><label>Mill tolerance<span class="unit">%</span></label><input class="mono" value={num(p.mill * 100, 4)} onChange={numIn(v => put({ mill: v / 100 }), () => num(p.mill * 100, 4))} /></div></div>
        <div class="two"><F l="Insulation thickness" q="len" v={p.ins_t} set={ins_t => put({ ins_t })} /><F l="Insulation density" q="density" v={p.ins_rho} set={ins_rho => put({ ins_rho })} /></div>
        <div class="two"><F l="Cladding thickness" q="len" v={p.clad_t} set={clad_t => put({ clad_t })} /><F l="Cladding density" q="density" v={p.clad_rho} set={clad_rho => put({ clad_rho })} /></div>
        <F l="Contents density" q="density" v={p.fluid_rho} set={fluid_rho => put({ fluid_rho })} hint="0 for gas or an empty line; water is 1000 kg/m³ (62.4 lb/ft³). The hydrotest fills the line with water." />
        <table class="vmini"><tbody><tr><td>Inside diameter</td><td class="r">{num(toU("len", p.od - 2 * p.wall), 5)} {unit("len")}</td></tr><tr><td>Wall after corrosion and mill tolerance</td><td class="r">{num(toU("len", p.wall * (1 - p.mill) - p.corrosion), 4)} {unit("len")}</td></tr></tbody></table>
      </>}</Manager>
    </Dialog>
  );
});

/** A table of [temperature, value] rows. */
const TTable = (p: { l: string; q: Q; rows: [number, number][]; set: (r: [number, number][]) => void; scale?: number }) => {
  const s = p.scale ?? 1;
  return <div class="ttab"><h4>{p.l}</h4>
    <table class="ptbl"><thead><tr><th>{unit("temp")}</th><th>{p.q === "expansion" ? unit("expansion") : unit(p.q)}</th><th /></tr></thead><tbody>
      <For each={p.rows}>{(r, i) => <tr>
        <td><input class="mono" value={num(toU("temp", r[0]), 5)} onChange={numIn(v => p.set(p.rows.map((x, k) => k === i() ? [fromU("temp", v), x[1]] : x) as [number, number][]), () => num(toU("temp", r[0]), 5))} /></td>
        <td><input class="mono" value={num(toU(p.q, r[1]) * s, 5)} onChange={numIn(v => p.set(p.rows.map((x, k) => k === i() ? [x[0], fromU(p.q, v / s)] : x) as [number, number][]), () => num(toU(p.q, r[1]) * s, 5))} /></td>
        <td><button class="x" title="Remove this row" disabled={p.rows.length < 2} onClick={() => p.set(p.rows.filter((_, k) => k !== i()))}>×</button></td></tr>}</For></tbody></table>
    <button class="btn sm" onClick={() => { const l = p.rows[p.rows.length - 1]; p.set([...p.rows, [l[0] + 50, l[1]]]); }}>Add a row</button></div>;
};

export const materials = (focus?: number) => openDialog(() => {
  const list = () => model()?.materials ?? [];
  const [cur, setCur] = createSignal(focus ?? list()[0]?.id ?? 0);
  const [L] = createResource(library);
  const put = (x: Partial<Material>) => { const p = list().find(y => y.id === cur()); if (p) act({ Set: { item: { Material: { ...p, ...x } } } }); };
  const add = async () => { const p = list().find(y => y.id === cur()) ?? list()[0]; const o = await act({ Add: { item: { Material: { ...p, id: 0, name: `${p.name} (copy)`, source: "Your own values" } }, after: null } }); if (o?.created.length) setCur(o.created[0]); };
  const fromLib = async (id: string) => { const x = L()?.materials.find(y => y.id === id); if (!x) return; const o = await act({ Add: { item: { Material: { id: 0, name: x.name, rho: x.rho, nu: x.nu, e: x.e, alpha: x.alpha, allow: x.allow, quality: 1, weld: 1, y: 0.4, source: x.source } }, after: null } }); if (o?.created.length) setCur(o.created[0]); };
  return (
    <Dialog title="Materials" wide help="materials">
      <Manager list={list} cur={cur} setCur={setCur} add={add} del={() => act({ Delete: { ids: [cur()] } }).then(o => o && setCur(list()[0]?.id ?? 0))} used={usedBy("material")}
        extra={<div class="iprow"><label>Add from the library</label><select onChange={e => { fromLib(e.currentTarget.value); e.currentTarget.value = ""; }}><option value="">Choose…</option><For each={L()?.materials ?? []}>{x => <option value={x.id}>{x.name}</option>}</For></select></div>}>{x => <>
        <T l="Name" v={x.name} set={name => put({ name })} />
        <div class="two"><F l="Density" q="density" v={x.rho} set={rho => put({ rho })} /><div class="iprow"><label>Poisson's ratio</label><input class="mono" value={x.nu} onChange={numIn(v => put({ nu: v }), () => x.nu)} /></div></div>
        <div class="three"><TTable l="Modulus E" q="modulus" rows={x.e} set={e => put({ e })} /><TTable l="Mean expansion from 21 °C" q="expansion" rows={x.alpha} set={alpha => put({ alpha })} /><TTable l="Allowable stress S" q="stress" rows={x.allow} set={allow => put({ allow })} /></div>
        <p class="warnline"><Icon n="info" /> Allowable stresses are your inputs. The values filled in are typical (the lower of a third of the tensile strength and two thirds of the yield at temperature): check them against your code edition.</p>
        <div class="three"><div class="iprow"><label>Quality factor E</label><input class="mono" value={x.quality} onChange={numIn(v => put({ quality: v }), () => x.quality)} /></div>
          <div class="iprow"><label>Weld strength factor W</label><input class="mono" value={x.weld} onChange={numIn(v => put({ weld: v }), () => x.weld)} /></div>
          <div class="iprow"><label>Coefficient Y</label><input class="mono" value={x.y} onChange={numIn(v => put({ y: v }), () => x.y)} /></div></div>
        <div class="iprow"><label>Source</label><textarea rows="3" value={x.source} onChange={e => put({ source: e.currentTarget.value })} /></div>
      </>}</Manager>
    </Dialog>
  );
});

export const conds = (focus?: number) => openDialog(() => {
  const list = () => model()?.conds ?? [];
  const [cur, setCur] = createSignal(focus ?? list()[0]?.id ?? 0);
  const put = (x: Partial<Cond>) => { const p = list().find(y => y.id === cur()); if (p) act({ Set: { item: { Cond: { ...p, ...x } } } }); };
  const add = async () => { const p = list().find(y => y.id === cur()) ?? list()[0]; const o = await act({ Add: { item: { Cond: { ...p, id: 0, name: `${p.name} (copy)` } }, after: null } }); if (o?.created.length) setCur(o.created[0]); };
  return (
    <Dialog title="Line conditions" wide help="conds">
      <Manager list={list} cur={cur} setCur={setCur} add={add} del={() => act({ Delete: { ids: [cur()] } }).then(o => o && setCur(list()[0]?.id ?? 0))} used={usedBy("cond")}
        extra={<p class="muted small">Installed temperature: {num(toU("temp", model()?.settings.ambient ?? 294.15), 4)} {unit("temp")} (in the Inspector with nothing selected).</p>}>{c => <>
        <T l="Name" v={c.name} set={name => put({ name })} />
        <table class="ptbl conds"><thead><tr><th /><th>Temperature ({unit("temp")})</th><th>Pressure ({unit("press")})</th></tr></thead><tbody>
          <For each={[0, 1, 2]}>{i => <tr><td>Case {i + 1}</td>
            <td><input class="mono" value={num(toU("temp", c.t[i]), 5)} onChange={numIn(v => put({ t: c.t.map((x, k) => k === i ? fromU("temp", v) : x) as any }), () => num(toU("temp", c.t[i]), 5))} /></td>
            <td><input class="mono" value={num(toU("press", c.p[i]), 5)} onChange={numIn(v => put({ p: c.p.map((x, k) => k === i ? fromU("press", v) : x) as any }), () => num(toU("press", c.p[i]), 5))} /></td></tr>}</For>
          <tr><td>Hydrotest</td><td class="muted">ambient</td><td><input class="mono" value={num(toU("press", c.hydro), 5)} onChange={numIn(v => put({ hydro: fromU("press", v) }), () => num(toU("press", c.hydro), 5))} /></td></tr></tbody></table>
        <p class="muted small">Pressures are gauge. A case whose temperature is the installed temperature and whose pressure is 0 is not used by these elements.</p>
      </>}</Manager>
    </Dialog>
  );
});

const KINDS: [string, string][] = [["sus", "Sustained (SUS)"], ["ope", "Operating (OPE)"], ["exp", "Expansion range (EXP)"], ["hyd", "Hydrotest (HYD)"], ["occ", "Occasional (OCC)"]];
export const recipeText = (c: LoadCase, all: LoadCase[]) => {
  const r = c.recipe;
  if (r.kind === "range") return `${all.find(x => x.id === r.a)?.name ?? "?"} − ${all.find(x => x.id === r.b)?.name ?? "?"}`;
  return [r.w ? (r.water ? "W (full of water)" : "W") : "", r.t ? `T${r.t}` : "", r.d ? `D${r.d}` : "", r.p ? (r.p === 4 ? "HP" : `P${r.p}`) : "", r.wind ? "WIN" : "", r.seis ? "SEIS" : ""].filter(Boolean).join(" + ") || "nothing";
};

export const cases = () => openDialog(() => {
  const list = () => model()?.cases ?? [];
  const set = (cs: LoadCase[]) => act({ SetCases: { cases: cs } });
  const put = (id: number, x: Partial<LoadCase>) => set(list().map(c => c.id === id ? { ...c, ...x } : c));
  const putR = (c: LoadCase, x: Partial<Recipe>) => put(c.id, { recipe: { ...c.recipe, ...x } as Recipe });
  const sel = (v: number, set: (n: number) => void, opts: [number, string][]) => <select onChange={e => set(+e.currentTarget.value)}><For each={opts}>{o => <option value={o[0]} selected={o[0] === v}>{o[1]}</option>}</For></select>;
  const n03: [number, string][] = [[0, "—"], [1, "1"], [2, "2"], [3, "3"]];
  return (
    <Dialog title="Load cases" wide help="cases">
      <p class="muted">{model()?.settings.auto_cases ? "Built from the model, and kept up to date as you edit. Changing a case here turns that off." : "Your own load cases. Recommend builds them from the model again."}</p>
      <table class="tbl cases"><thead><tr><th>Name</th><th>Kind</th><th>Weight</th><th>Temperature</th><th>Displacements</th><th>Pressure</th><th>Occasional</th><th>Range</th><th>Made of</th><th /></tr></thead><tbody>
        <For each={list()}>{c => <tr>
          <td><input value={c.name} onChange={e => put(c.id, { name: e.currentTarget.value })} /></td>
          <td><select onChange={e => put(c.id, { kind: e.currentTarget.value as any })}><For each={KINDS}>{k => <option value={k[0]} selected={k[0] === c.kind}>{k[1]}</option>}</For></select></td>
          <Show when={c.recipe.kind === "sum"} fallback={<><td colSpan={5} class="muted">The range between two cases</td><td>{c.recipe.kind === "range" && <span class="inline">{sel(c.recipe.a, a => putR(c, { a } as any), list().filter(x => x.id !== c.id).map(x => [x.id, x.name]))} − {sel(c.recipe.b, b => putR(c, { b } as any), list().filter(x => x.id !== c.id).map(x => [x.id, x.name]))}</span>}</td></>}>{_ => { const r = c.recipe as Extract<Recipe, { kind: "sum" }>; return <>
            <td><select onChange={e => putR(c, { w: e.currentTarget.value !== "0", water: e.currentTarget.value === "2" } as any)}><option value="0" selected={!r.w}>—</option><option value="1" selected={r.w && !r.water}>W</option><option value="2" selected={r.w && r.water}>W full of water</option></select></td>
            <td>{sel(r.t, t => putR(c, { t } as any), n03)}</td><td>{sel(r.d, d => putR(c, { d } as any), n03)}</td><td>{sel(r.p, p => putR(c, { p } as any), [...n03, [4, "Hydrotest"]])}</td>
            <td>{sel((r.wind ? 1 : 0) + (r.seis ? 2 : 0), o => putR(c, { wind: !!(o & 1), seis: !!(o & 2) } as any), [[0, "—"], [1, "Wind"], [2, "Seismic"], [3, "Both"]])}</td><td /></>; }}</Show>
          <td class="muted">{recipeText(c, list())}</td>
          <td><button class="x" title="Remove this case" onClick={() => set(list().filter(x => x.id !== c.id))}>×</button></td></tr>}</For></tbody></table>
      <p class="ibtns"><button class="btn sm" onClick={() => set([...list(), { id: 0, name: `L${list().length + 1}`, kind: "ope", recipe: { kind: "sum", w: true, water: false, t: 1, d: 1, p: 1, wind: false, seis: false } }])}>Add a case</button>
        <button class="btn sm" onClick={() => { const a = list()[0], b = list()[1]; if (!a || !b) return toast("A range needs two cases."); set([...list(), { id: 0, name: `EXP${list().length + 1}`, kind: "exp", recipe: { kind: "range", a: a.id, b: b.id } }]); }}>Add a range</button>
        <button class="btn sm pri" onClick={C.recommendCases}>Recommend</button></p>
    </Dialog>
  );
});

/** The spring table hangers are sized from: your supplier's sizes, in order of preference. */
export const springTable = () => openDialog(() => {
  const s = () => model()!.settings;
  const set = (springs: SpringSize[]) => C.settings({ springs });
  const put = (i: number, x: Partial<SpringSize>) => set(s().springs.map((r, k) => k === i ? { ...r, ...x } : r));
  const [L] = createResource(library);
  const cell = (q: "tstiff" | "force", val: number, f: (v: number) => void) => <input class="mono" value={num(toU(q, val), 6)} onChange={e => { const v = parseNum(e.currentTarget.value); if (v !== null && v >= 0) f(fromU(q, v)); }} />;
  return (
    <Dialog title="Spring table" wide help="hangers">
      <p class="muted">Spring hangers are sized from these springs, in this order: the first whose working range takes both the hot and the cold load, with the load variation within the limit, is chosen. The table filled in is generic: replace it with your supplier's sizes and check the result against their catalogue.</p>
      <div class="iprow"><label>Load variation allowed (|k Δ| / hot load), %</label><input class="mono" value={num(100 * s().variability, 4)} onChange={e => { const v = parseNum(e.currentTarget.value); if (v !== null && v > 0 && v <= 100) C.settings({ variability: v / 100 }); }} /></div>
      <table class="tbl"><thead><tr><th>Size</th><th>Rate ({unit("tstiff")})</th><th>Working from ({unit("force")})</th><th>to ({unit("force")})</th><th /></tr></thead><tbody>
        <For each={s().springs}>{(r, i) => <tr><td><input value={r.name} onChange={e => put(i(), { name: e.currentTarget.value })} /></td><td>{cell("tstiff", r.k, k => k > 0 && put(i(), { k }))}</td><td>{cell("force", r.min, min => put(i(), { min }))}</td><td>{cell("force", r.max, max => put(i(), { max }))}</td>
          <td><button class="x" title="Remove this size" onClick={() => set(s().springs.filter((_, k) => k !== i()))}>×</button></td></tr>}</For></tbody></table>
      <p class="ibtns"><button class="btn sm" onClick={() => set([...s().springs, { name: `S-${s().springs.length + 1}`, k: 1e5, min: 1000, max: 2000 }])}>Add a size</button>
        <button class="btn sm" onClick={() => L() && set(L()!.springs)}>Back to the generic table</button></p>
    </Dialog>
  );
});

/** Help › Verification: every benchmark solved now by this version, against its reference. */
export const verification = () => openDialog(() => {
  const [rows] = createResource(() => call<Check[]>("verification"));
  const [ver] = createResource(() => call<string>("version"));
  const val = (q: string, v: number) => (q === "ratio" ? num(v, 7) : fmtQ(q as Q, v));
  return (
    <Dialog title="Verification" wide help="verification">
      <div class="verif">
        <p>Every check below was solved just now by Datum Pipe {ver()} and compared with a reference worked out independently. A check fails if its error is over the tolerance; the same checks run as tests before every release.</p>
        <Show when={rows()} fallback={<p class="muted">Solving the benchmarks…</p>}>{r => <>
          <p classList={{ okline: r().every(x => x.pass), badline: !r().every(x => x.pass) }}><Icon n={r().every(x => x.pass) ? "check" : "info"} /> {r().filter(x => x.pass).length} of {r().length} checks pass.</p>
          <table class="tbl"><thead><tr><th>#</th><th>Benchmark</th><th>Quantity</th><th class="r">Datum Pipe</th><th class="r">Reference</th><th class="r">Error</th><th>Source</th><th /></tr></thead><tbody>
            <For each={r()}>{x => <tr><td>{x.bench}</td><td>{x.name}</td><td>{x.quantity}</td><td class="r">{val(x.q, x.result)}</td><td class="r">{val(x.q, x.reference)}</td><td class="r">{x.error < 1e-12 ? "< 1e-12" : x.error.toExponential(1)}</td><td>{x.source}</td><td>{x.pass ? "✓ pass" : "✗ fail"}</td></tr>}</For></tbody></table>
          <p class="muted small">Coming with dynamics (0.4): the public piping benchmark problems published by nuclear regulators, which are dynamic problems (modes and response spectra).</p>
        </>}</Show>
      </div>
    </Dialog>
  );
});
const fmtQ = (q: Q, v: number) => `${num(toU(q, v), 7)} ${unit(q)}`;
