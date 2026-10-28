// Title bar, quick access toolbar, Tell Me and the ribbon (Home · Model · Loads · Supports · Analyze
// · Results · Isometric · View · Help). Copied from Datum Plan's ribbon by way of Datum Mesh;
// planned tools are shown dimmed with the release they arrive in.
import { For, Show, JSX, createSignal, createMemo, onMount } from "solid-js";
import { call, out, model, tab, setTab, setBackstage, openMenu, openDialog, closeDialog, theme, setTheme, sel, setSel, opts, setOpts, act, view, setView } from "./store";
import { C, ALL_CMDS, SUPPORTS } from "./cmds";
import { ResultsRibbon } from "./results";
import { IsoRibbon } from "./iso";
import * as D from "./dialogs";
import * as P from "./props";
import * as T from "./tools";
import { Icon } from "./ui";
import { Help, helpCard, HELP } from "./help";
import { invoke, Example } from "./api";
import { COMPS, compFrom, compKey } from "./table";

const GH = "https://github.com/Edgar-Mendonca/Datum-Pipe";

export const B = (p: { i: string; l: string; run?: (e: MouseEvent) => void; big?: boolean; on?: boolean; off?: boolean; dd?: (e: MouseEvent) => void; title?: string }) => p.big && p.run && p.dd ? (
  <div class="rbw big splitbig" classList={{ on: p.on }}>
    <button class="rb top" title={p.title ?? p.l} disabled={p.off} onClick={e => p.run!(e)}><Icon n={p.i} /></button>
    <button class="rb bot" title={`${p.l} options`} onClick={e => p.dd!(e)}><span>{p.l}</span><b class="car">▾</b></button>
  </div>
) : (
  <div class="rbw" classList={{ big: p.big }}>
    <button class="rb" classList={{ big: p.big, on: p.on }} disabled={p.off} title={p.title ?? p.l} onClick={e => (p.run ?? p.dd)?.(e)}><Icon n={p.i} />{p.big ? <span>{p.l}{p.dd && !p.run ? " ▾" : ""}</span> : <span>{p.l}</span>}{!p.big && p.dd && !p.run && <b class="car">▾</b>}</button>
    {p.run && p.dd && <button class="rb split" onClick={e => p.dd!(e)}>▾</button>}
  </div>
);
export const G = (p: { l: string; h?: string; children: JSX.Element }) => <div class="grp"><div class="items">{p.children}</div><div class="cap">{p.l}{p.h && <Help k={p.h} />}</div></div>;
export const S = (p: { children: JSX.Element }) => <div class="stack">{p.children}</div>;
/** A planned tool: shown dimmed with the release it arrives in. */
export const Planned = (p: { i: string; l: string; v: string; big?: boolean }) => <B i={p.i} l={p.l} big={p.big} off title={`${p.l}: planned for ${p.v}`} />;

/** Tell Me / command search (Ctrl+K). */
export const palette = () => openDialog(() => {
  const [q, setQ] = createSignal(""), [k, setK] = createSignal(0);
  const list = createMemo(() => { const s = q().toLowerCase(); return ALL_CMDS.filter(c => c[0].toLowerCase().includes(s)).slice(0, 14); });
  const go = (i: number) => { const c = list()[i]; if (c) { closeDialog(); setTimeout(() => c[1]()); } };
  let input!: HTMLInputElement;
  onMount(() => input.focus());
  return (
    <div class="ov top" onMouseDown={e => e.target === e.currentTarget && closeDialog()}>
      <div class="pal">
        <input ref={input} placeholder="Tell me what you want to do" value={q()} onInput={e => { setQ(e.currentTarget.value); setK(0); }}
          onKeyDown={e => { if (e.key === "ArrowDown") { e.preventDefault(); setK(Math.min(list().length - 1, k() + 1)); } else if (e.key === "ArrowUp") { e.preventDefault(); setK(Math.max(0, k() - 1)); } else if (e.key === "Enter") { e.preventDefault(); go(k()); } else if (e.key === "Escape") closeDialog(); }} />
        <div class="pl"><For each={list()}>{(c, i) => <button classList={{ on: i() === k() }} onMouseEnter={() => setK(i())} onClick={() => go(i())}><span>{c[0]}</span>{c[2] && <kbd>{c[2]}</kbd>}</button>}</For></div>
      </div>
    </div>
  );
});

export const TABS: [string, string][] = [["home", "Home"], ["model", "Model"], ["loads", "Loads"], ["supports", "Supports"], ["analyze", "Analyze"], ["results", "Results"], ["isometric", "Isometric"], ["view", "View"], ["help", "Help"]];

export const TitleBar = (p: { update: string | null }) => (
  <header class="top">
    <div class="bar">
      <div class="qat">
        <img src="logo.svg" alt="" class="logo" />
        <button title="Save (Ctrl+S)" onClick={() => C.save()}><Icon n="save" /></button>
        <button title="Undo (Ctrl+Z)" disabled={!out()?.undo} onClick={C.undo}><Icon n="undo" /></button>
        <button title="Redo (Ctrl+Y)" disabled={!out()?.redo} onClick={C.redo}><Icon n="redo" /></button>
        <button title="Print (Ctrl+P)" onClick={C.print}><Icon n="print" /></button>
      </div>
      <div class="title">{out() ? `${out()!.m.name}${out()!.dirty ? " •" : ""} — Datum Pipe` : "Datum Pipe"}</div>
      <button class="search" onClick={palette} title="Tell me what you want to do (Ctrl+K)"><Icon n="search" /><span>Tell me what you want to do</span><kbd>Ctrl+K</kbd></button>
      <Show when={p.update}><button class="upd" onClick={() => setBackstage("about")}>Update {p.update} available</button></Show>
      <button class="theme" title="Theme: Auto follows Windows" onClick={() => setTheme(theme() === "auto" ? "light" : theme() === "light" ? "dark" : "auto")}>Theme · {theme()}</button>
    </div>
    <nav class="tabs">
      <button class="file" onClick={() => setBackstage("home")}>File</button>
      <For each={TABS}>{([k, l]) => <button classList={{ on: tab() === k }} onClick={() => setTab(k)}>{l}</button>}</For>
      <div class="ctxgrp" style={{ visibility: selEl() ? "visible" : "hidden" }}><small>Element Tools</small><button classList={{ on: tab() === "element" }} onClick={() => setTab("element")}>Component</button></div>
    </nav>
  </header>
);

/** The selected element, for the contextual Component tab. */
const selEl = () => { const s = sel(), m = model(); return s?.kind === "element" && m ? m.elements.find(e => e.id === s.id) : undefined; };

/** Shrinks the ribbon in steps instead of scrolling sideways (FAMILY §6). */
const fitRibbon = (el: HTMLElement) => {
  const fit = () => {
    const gs = [...el.querySelectorAll<HTMLElement>(":scope > .grp")];
    el.dataset.fit = "0"; gs.forEach(g => delete g.dataset.fit);
    const over = () => el.scrollWidth > el.clientWidth + 1;
    if (!over()) return;
    el.dataset.fit = "1";
    for (const lvl of ["2", "3"]) for (let i = gs.length - 1; i >= 0 && over(); i--) gs[i].dataset.fit = lvl;
  };
  new ResizeObserver(fit).observe(el);
  new MutationObserver(() => requestAnimationFrame(fit)).observe(el, { childList: true, subtree: false });
};

const exampleMenu = async (e: MouseEvent) => {
  const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const list = await invoke<Example[]>("examples_list").catch(() => [] as Example[]);
  openMenu({ x: r.left, y: r.bottom + 2 }, list.map(x => ({ label: x.name, run: () => C.newModel(x.id) })));
};

/** Sets the component of the selected element. */
const setComp = (key: string) => { const e = selEl(), m = model(); if (!e || !m) return; act({ Set: { item: { Element: { ...e, comp: compFrom(key, e, m) } } } }); };
const Comp = (p: { k: string; i: string; l: string; big?: boolean }) => <B i={p.i} l={p.l} big={p.big} on={!!selEl() && compKey(selEl()!.comp) === p.k} off={!selEl()} run={() => setComp(p.k)} title={selEl() ? p.l : `${p.l}: select an element first`} />;
const Sup = (p: { k: string; i: string; big?: boolean; l?: string }) => <B i={p.i} l={p.l ?? SUPPORTS.find(s => s[0] === p.k)![1]} big={p.big} run={() => C.support(p.k)} />;

export const Ribbon = () => {
  const m = () => model();
  return (
    <section class="ribbon" ref={fitRibbon}>
      <Show when={tab() === "home"}>
        <G l="Elements" h="table"><B big i="addel" l="Add Element" run={C.addAfter} title="Add an element after the selected one (Ctrl+Enter)" /><S><B i="split" l="Insert Node" run={C.split} /><B i="del" l="Delete" run={C.del} off={!sel()} /></S></G>
        <G l="Units" h="units"><S><B i="units" l="SI: mm, N, MPa, bar" on={m()?.units === "SI"} run={() => C.units("SI")} /><B i="units" l="US: ft, in, lbf, psi" on={m()?.units === "US"} run={() => C.units("US")} /></S></G>
        <G l="Examples" h="examples"><B big i="open" l="Examples" dd={exampleMenu} /></G>
        <G l="Analyze" h="analyze"><B big i="solve" l="Analyze" run={C.analyze} title="Solve every load case and check the code stresses (F5)" /></G>
      </Show>
      <Show when={tab() === "model"}>
        <G l="Elements" h="elements"><B big i="addel" l="Add Element" run={C.addAfter} /><S><B i="split" l="Insert Node" run={C.split} /><B i="renum" l="Renumber" run={C.renumber} /><B i="del" l="Delete" run={C.del} off={!sel()} /></S></G>
        <G l="Components" h="components"><Comp big k="bend:long" i="bend" l="Bend" /><S><Comp k="tee:welding" i="tee" l="Tee" /><Comp k="reducer" i="reducer" l="Reducer" /><Comp k="pipe" i="pipe" l="Straight" /></S><S><Comp k="rigid:valve" i="valve" l="Valve" /><Comp k="rigid:flange" i="flange" l="Flange Pair" /><Comp k="rigid:rigid" i="rigid" l="Rigid" /></S></G>
        <G l="Properties" h="pipes"><B big i="pipe" l="Pipes" run={() => P.pipes()} /><B big i="material" l="Materials" run={() => P.materials()} /><B big i="temp" l="Line Conditions" run={() => P.conds()} /></G>
        <G l="Edit" h="edit"><B big i="loop" l="Expansion Loop" run={() => T.loopWizard()} title="Turn the selected straight element into an expansion loop" /><S><B i="copy" l="Copy Section" run={() => T.copySection()} /><B i="mirror" l="Mirror Section" run={() => T.copySection(true)} /></S></G>
      </Show>
      <Show when={tab() === "loads"}>
        <G l="Conditions" h="conds"><B big i="temp" l="Line Conditions" run={() => P.conds()} /></G>
        <G l="Extra Loads" h="loads"><B big i="force" l="Force at Node" run={C.load} /><S><B i="dist" l="Uniform Load" run={() => selEl() ? setTab("element") : C.planned("Select an element, then set its extra load in the Inspector; uniform loads", "this release")} title="Select an element and set its extra load (per length) in the Inspector" /></S></G>
        <G l="Displacements" h="disp"><B big i="disp" l="Imposed Displacement" run={() => C.support("anchor")} title="Put an anchor on the selected node and set its movements per case in the Inspector" /></G>
        <G l="Load Cases" h="cases"><B big i="cases" l="Load Cases" run={() => P.cases()} /><S><B i="check" l="Recommend" run={C.recommendCases} on={!!m()?.settings.auto_cases} /><label class="chk"><input type="checkbox" checked={!!m()?.settings.hydro} onChange={e => C.settings({ hydro: e.currentTarget.checked })} />Hydrotest case</label></S></G>
        <G l="Occasional" h="cases"><S><Planned i="wind" l="Wind" v="0.2" /><Planned i="seismic" l="Static Seismic" v="0.2" /></S></G>
      </Show>
      <Show when={tab() === "supports"}>
        <G l="Supports" h="supports"><Sup big k="anchor" i="anchor" /><Sup big k="rest" i="rest" l="Rest" /><Sup big k="guide" i="guide" /><Sup big k="linestop" i="linestop" /></G>
        <G l="Restraints" h="supports"><S><Sup k="rx" i="restraint" /><Sup k="ry" i="restraint" /><Sup k="rz" i="restraint" /></S><S><Sup k="skew" i="restraint" l="Skewed" /><Sup k="spring" i="spring" l="Spring" /><Sup k="rotspring" i="spring" l="Rotational Spring" /></S></G>
        <G l="Hangers" h="supports"><Sup big k="constant" i="constant" l="Constant Force" /><S><Planned i="spring" l="Spring Hanger Sizing" v="0.2" /></S></G>
        <G l="Equipment" h="nozzles"><B big i="nozzle" l="Nozzle Point" run={C.nozzle} /><S><Planned i="nozzle" l="Pump and Compressor Checks" v="0.3" /></S></G>
        <G l="Nonlinear" h="supports"><S><Planned i="rest" l="One-way Rest" v="0.2" /><Planned i="guide" l="Gaps and Friction" v="0.2" /></S></G>
      </Show>
      <Show when={tab() === "analyze"}>
        <G l="Analyze" h="analyze"><B big i="solve" l="Analyze" run={C.analyze} title="Solve every load case (F5)" /><S><B i="cases" l="Load Cases" run={() => P.cases()} /><B i="verify" l="Verification" run={() => P.verification()} /></S></G>
        <G l="Code" h="code"><S>
          <label class="chk"><input type="checkbox" checked={!!m()?.settings.liberal} onChange={e => C.settings({ liberal: e.currentTarget.checked })} />Liberal allowable range</label>
          <label class="chk"><input type="checkbox" checked={!!m()?.settings.bend_pressure} onChange={e => C.settings({ bend_pressure: e.currentTarget.checked })} />Pressure correction on bends</label>
          <B i="opts" l="Analysis Settings" run={() => setSel(null)} title="Nothing selected: the model's settings show in the Inspector" />
        </S></G>
        <G l="Dynamics" h="analyze"><S><Planned i="wave" l="Natural Frequencies" v="0.4" /><Planned i="wave" l="Response Spectrum" v="0.4" /></S></G>
      </Show>
      <Show when={tab() === "results"}><ResultsRibbon /></Show>
      <Show when={tab() === "isometric"}><IsoRibbon /></Show>
      <Show when={tab() === "view"}>
        <G l="Zoom" h="view"><B big i="fit" l="Fit" run={C.fit} /></G>
        <G l="Standard Views" h="view"><B big i="cube" l="Isometric" run={() => C.std("iso")} /><S><B i="plan" l="Plan (from above)" run={() => C.std("plan")} /><B i="front" l="Front (from south)" run={() => C.std("front")} /><B i="side" l="Side (from east)" run={() => C.std("side")} /></S></G>
        <G l="Show" h="view"><S><label class="chk"><input type="checkbox" checked={opts().labels} onChange={e => setOpts({ ...opts(), labels: e.currentTarget.checked })} />Node numbers</label></S></G>
        <G l="Views" h="view"><S><B i="model" l="Model" on={view() === "model"} run={() => setView("model")} /><B i="table" l="Results" on={view() === "results"} run={() => setView("results")} /><B i="iso" l="Isometric" on={view() === "iso"} run={() => setView("iso")} /></S></G>
      </Show>
      <Show when={tab() === "element"}>
        <Show when={selEl()} fallback={<G l="Component"><div class="rnote">Select an element in the table or the 3D view.</div></G>}>
          <G l="Component" h="components"><For each={COMPS.slice(0, 4)}>{c => <Comp k={c[0]} i={c[0] === "pipe" ? "pipe" : "bend"} l={c[1]} />}</For></G>
          <G l="Tees" h="components"><S><For each={COMPS.slice(4, 7)}>{c => <Comp k={c[0]} i="tee" l={c[1]} />}</For></S></G>
          <G l="Fittings" h="components"><S><Comp k="reducer" i="reducer" l="Reducer" /><Comp k="rigid:valve" i="valve" l="Valve" /><Comp k="rigid:rigid" i="rigid" l="Rigid" /></S><S><Comp k="rigid:flange" i="flange" l="Flange pair, weld neck" /><Comp k="rigid:slipon" i="flange" l="Flange pair, slip-on" /></S></G>
        </Show>
      </Show>
      <Show when={tab() === "help"}>
        <G l="Learn"><B big i="report" l="Getting Started" run={() => D.gettingStarted()} /><B big i="info" l="Glossary" run={() => D.glossary()} /><B big i="help" l="What Is This Tool?" dd={e => openMenu(e, Object.entries(HELP).map(([k, h]) => ({ label: h.t, run: () => helpCard(k) })))} /><B big i="keys" l="Shortcuts" run={() => D.shortcuts()} /><B big i="search" l="Tell Me" run={palette} /></G>
        <G l="Trust"><B big i="verify" l="Verification" run={() => P.verification()} /></G>
        <G l="Datum Pipe on GitHub"><B big i="github" l="GitHub" run={() => call("open_url", { url: GH })} /><B big i="tag" l="Releases" run={() => call("open_url", { url: `${GH}/releases` })} /><B big i="bug" l="Report an Issue" run={() => call("open_url", { url: `${GH}/issues/new` })} /></G>
        <G l="Updates"><B big i="upd" l="Check for Updates" run={() => setBackstage("about")} /><B big i="list" l="Roadmap" run={() => setBackstage("roadmap")} /></G>
      </Show>
    </section>
  );
};
