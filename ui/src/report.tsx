// The stress report: the title block, the input echo (pipes, materials, line conditions, elements,
// supports, nozzles), the load cases, and once analysed the results sections. Each block is a
// function so the print page can lay the report out on as many pages as it needs.
import { For, Show, JSX, onMount } from "solid-js";
import { Model, num } from "./api";
import { call, setBackstage, model } from "./store";
import { Help } from "./help";
import { TitleBlock, version, setVersion } from "./print";
import { fmt, unit, toU } from "./units";
import { compName } from "./table";
import { supName } from "./panels";
import { recipeText } from "./props";

type Block = () => JSX.Element;
let resultBlocks: (m: Model) => Block[] = () => [() => <p class="muted">Analyze the model (F5) to add the stress summary, support loads, nozzle loads and the equilibrium checks.</p>];
export const setResultBlocks = (f: (m: Model) => Block[]) => { resultBlocks = f; };
let isoPages: (m: Model, w: number, h: number) => JSX.Element[] = () => [];
export const setIsoPages = (f: (m: Model, w: number, h: number) => JSX.Element[]) => { isoPages = f; };
export const reportExtras = (m: Model, w: number, h: number) => isoPages(m, w, h);

const v = (q: Parameters<typeof fmt>[0], x: number) => num(toU(q, x), 4);

export const ReportBody = (p: { m: Model }): Block[] => {
  const m = p.m;
  const name = (l: { id: number; name: string }[], id: number) => l.find(x => x.id === id)?.name ?? "?";
  return [
    () => <div><h2>{m.project.project || m.name}</h2><TitleBlock m={m} /><Show when={m.notes}><p>{m.notes}</p></Show>
      <p class="muted small">Calculated using the equations of the process piping code (ASME B31.3). Allowable stresses are the user's inputs. Results must be checked by a qualified engineer.</p></div>,
    () => <div><h3>1. Input</h3><h4>Pipes</h4>
      <table class="tbl"><thead><tr><th>Pipe</th><th class="r">OD ({unit("len")})</th><th class="r">Wall</th><th class="r">Corrosion</th><th class="r">Mill tol.</th><th class="r">Insulation</th><th class="r">Ins. density ({unit("density")})</th><th class="r">Contents ({unit("density")})</th></tr></thead><tbody>
        <For each={m.pipes}>{x => <tr><td>{x.name}</td><td class="r">{v("len", x.od)}</td><td class="r">{v("len", x.wall)}</td><td class="r">{v("len", x.corrosion)}</td><td class="r">{num(x.mill * 100, 3)} %</td><td class="r">{v("len", x.ins_t)}</td><td class="r">{v("density", x.ins_rho)}</td><td class="r">{v("density", x.fluid_rho)}</td></tr>}</For></tbody></table></div>,
    () => <div><h4>Materials</h4>
      <For each={m.materials}>{x => <div class="rmat"><b>{x.name}</b> · density {fmt("density", x.rho)} · ν {x.nu} · E {x.quality}, W {x.weld}, Y {x.y}
        <table class="tbl"><thead><tr><th>{unit("temp")}</th><For each={x.e}>{r => <th class="r">{v("temp", r[0])}</th>}</For></tr></thead><tbody>
          <tr><td>E ({unit("modulus")})</td><For each={x.e}>{r => <td class="r">{v("modulus", r[1])}</td>}</For></tr></tbody></table>
        <table class="tbl"><thead><tr><th>{unit("temp")}</th><For each={x.alpha}>{r => <th class="r">{v("temp", r[0])}</th>}</For></tr></thead><tbody>
          <tr><td>α ({unit("expansion")})</td><For each={x.alpha}>{r => <td class="r">{v("expansion", r[1])}</td>}</For></tr></tbody></table>
        <table class="tbl"><thead><tr><th>{unit("temp")}</th><For each={x.allow}>{r => <th class="r">{v("temp", r[0])}</th>}</For></tr></thead><tbody>
          <tr><td>S ({unit("stress")})</td><For each={x.allow}>{r => <td class="r">{v("stress", r[1])}</td>}</For></tr></tbody></table>
        <p class="muted small">{x.source}</p></div>}</For></div>,
    () => <div><h4>Line conditions</h4>
      <table class="tbl"><thead><tr><th>Conditions</th><th class="r">T1</th><th class="r">T2</th><th class="r">T3</th><th class="r">P1</th><th class="r">P2</th><th class="r">P3</th><th class="r">Hydrotest</th></tr></thead><tbody>
        <For each={m.conds}>{c => <tr><td>{c.name}</td><For each={c.t}>{t => <td class="r">{fmt("temp", t)}</td>}</For><For each={c.p}>{q => <td class="r">{fmt("press", q)}</td>}</For><td class="r">{fmt("press", c.hydro)}</td></tr>}</For></tbody></table>
      <p class="muted small">Installed temperature {fmt("temp", m.settings.ambient)}; {m.settings.cycles} displacement cycles; {m.settings.liberal ? "liberal" : "basic"} allowable stress range; bend pressure correction {m.settings.bend_pressure ? "on" : "off"}.</p></div>,
    () => <div><h4>Elements</h4>
      <table class="tbl small"><thead><tr><th>From</th><th>To</th><th class="r">DX ({unit("coord")})</th><th class="r">DY</th><th class="r">DZ</th><th>Pipe</th><th>Material</th><th>Conditions</th><th>Component</th></tr></thead><tbody>
        <For each={m.elements}>{e => <tr><td>{e.from}</td><td>{e.to}</td><For each={e.d}>{d => <td class="r">{d ? v("coord", d) : ""}</td>}</For><td>{name(m.pipes, e.pipe)}</td><td>{name(m.materials, e.material).split(",")[0]}</td><td>{name(m.conds, e.cond)}</td><td>{compName(e.comp)}</td></tr>}</For></tbody></table></div>,
    () => <div><h4>Supports, nozzles and loads</h4>
      <table class="tbl"><thead><tr><th>Node</th><th>Kind</th><th>Tag</th></tr></thead><tbody>
        <For each={m.supports} fallback={<tr><td colSpan={3}>No supports.</td></tr>}>{s => <tr><td>{s.node}</td><td>{supName(s.kind)}{s.k ? ` (${fmt("tstiff", s.k)})` : ""}</td><td>{s.tag}</td></tr>}</For>
        <For each={m.nozzles}>{z => <tr><td>{z.node}</td><td>Nozzle point</td><td>{z.name}</td></tr>}</For>
        <For each={m.loads}>{l => <tr><td>{l.node}</td><td>Force {l.f.map(x => v("force", x)).join(", ")} {unit("force")}; moment {l.m.map(x => v("moment", x)).join(", ")} {unit("moment")}</td><td /></tr>}</For></tbody></table></div>,
    () => <div><h3>2. Load cases</h3>
      <table class="tbl"><thead><tr><th>Case</th><th>Kind</th><th>Made of</th><th>Check</th></tr></thead><tbody>
        <For each={m.cases}>{c => <tr><td>{c.name}</td><td>{c.kind.toUpperCase()}</td><td>{recipeText(c, m.cases)}</td><td>{({ sus: "Sustained stress ≤ Sh", exp: "Stress range ≤ SA", ope: "Loads only", hyd: "Loads only", occ: "≤ 1.33 Sh" } as Record<string, string>)[c.kind]}</td></tr>}</For></tbody></table>
      <p class="muted small">W weight (pipe, contents, insulation, cladding, components, extra loads); T thermal; D imposed displacements; P pressure; HP hydrotest pressure.</p></div>,
    ...resultBlocks(m),
    () => <p class="muted small">Datum Pipe {version()} · free and open source (MIT) · https://github.com/Edgar-Mendonca/Datum-Pipe</p>,
  ];
};

/** The Report view: the stress report on screen, with a button to print it. */
export const ReportPage = () => {
  onMount(async () => { if (!version()) setVersion(await call<string>("version") ?? ""); });
  return <div class="page">
    <h1>Report <Help k="report" /></h1>
    <p class="muted">The stress report as it prints: the input, the load cases, and once analysed the stress summary, support and nozzle loads, the isometric and the equilibrium checks. <button class="btn pri sm" onClick={() => setBackstage("print")}>Print or save as PDF</button></p>
    <div class="rsheet"><Show when={model()}>{mm => <div class="preport">{ReportBody({ m: mm() }).map(b => b())}</div>}</Show></div>
  </div>;
};
