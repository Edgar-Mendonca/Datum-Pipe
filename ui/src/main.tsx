// Datum Pipe window: layout, sidebar, status bar, keyboard shortcuts, start-up and recovery.
import { render } from "solid-js/web";
import { For, Show, Switch, Match, onMount } from "solid-js";
import "@fontsource/ibm-plex-sans/latin-400.css";
import "@fontsource/ibm-plex-sans/latin-500.css";
import "@fontsource/ibm-plex-sans/latin-600.css";
import "@fontsource/ibm-plex-sans/latin-700.css";
import "@fontsource/ibm-plex-mono/latin-400.css";
import "@fontsource/ibm-plex-mono/latin-500.css";
import "@fontsource/ibm-plex-mono/latin-600.css";
import "./styles.css";
import { invoke, listen, Out, Options, Shown, Typed } from "./api";
import { out, setOut, view, setView, View, backstage, setBackstage, setOpts, opts, dialogs, toast, loadOut, setThemeRaw, applyTheme, sel, model, tab, setTab } from "./store";
import { C } from "./cmds";
import { TitleBar, Ribbon, palette } from "./ribbon";
import { Backstage, update, checkUpdate } from "./backstage";
import { ElementTable } from "./table";
import { Inspector } from "./panels";
import { View3D } from "./view3d";
import { ResultsPage } from "./results";
import { IsoPage } from "./iso";
import { ReportPage, setVesselBlocks } from "./report";
import { VesselPage, vesselReport } from "./vessel";
setVesselBlocks(vesselReport);
import { Menu, Toast, Icon } from "./ui";
import { setUnitTable } from "./units";
import * as D from "./dialogs";
import { solving, results, stale } from "./rstate";

const SIDE: ([View, string, string] | "-")[] = [["model", "model", "Model"], "-", ["results", "table", "Results"], ["iso", "iso", "Isometric"], ["vessel", "vessel", "Vessels"], ["report", "report", "Report"]];

/** The workflow bar (0.6.1, after Datum Mesh): the steps of a piping analysis down the left, each
 *  icon green once done, no numbers (the owner's choice). A click opens the step's ribbon tab and
 *  view; Isometric and Vessels follow a rule. */
type Step = { k: string; i: string; l: string; go: () => void; done: () => boolean; on: () => boolean };
const STEPS: (Step | "-")[] = [
  { k: "route", i: "pipe", l: "Route", go: () => { setView("model"); setTab("home"); }, done: () => !!model()?.elements.length, on: () => view() === "model" && tab() === "home" },
  { k: "props", i: "material", l: "Pipes", go: () => { setView("model"); setTab("model"); }, done: () => !!model()?.elements.length, on: () => view() === "model" && tab() === "model" },
  { k: "supports", i: "anchor", l: "Supports", go: () => { setView("model"); setTab("supports"); }, done: () => !!model()?.supports.length, on: () => view() === "model" && tab() === "supports" },
  { k: "loads", i: "force", l: "Loads", go: () => { setView("model"); setTab("loads"); }, done: () => !!model()?.cases.length && !!model()?.elements.length, on: () => view() === "model" && tab() === "loads" },
  { k: "analyze", i: "solve", l: "Analyze", go: () => { setView("model"); setTab("analyze"); C.analyze(); }, done: () => !!results() && !stale(), on: () => view() === "model" && tab() === "analyze" },
  { k: "results", i: "table", l: "Results", go: () => { setView("results"); setTab("results"); }, done: () => !!results() && !stale(), on: () => view() === "results" },
  { k: "report", i: "report", l: "Report", go: () => setView("report"), done: () => false, on: () => view() === "report" },
  "-",
  { k: "iso", i: "iso", l: "Isometric", go: () => { setView("iso"); setTab("isometric"); }, done: () => false, on: () => view() === "iso" },
  { k: "vessel", i: "vessel", l: "Vessels", go: () => setView("vessel"), done: () => !!model()?.vessels?.length, on: () => view() === "vessel" },
];
const Flow = () => <nav class="flow side"><For each={STEPS}>{s => s === "-" ? <hr /> : <button classList={{ on: s.on(), done: s.done() }} onClick={s.go} title={s.l}><Icon n={s.i} /><span>{s.l}</span></button>}</For></nav>;

const Status = () => (
  <footer class="status">
    <span>{solving() ? "Analyzing…" : "Ready"}</span>
    <span class="muted">{model() ? `${model()!.elements.length} elements · ${model()!.nodes.length} nodes · ${model()!.supports.length} supports · ${model()!.cases.length} load cases` : ""}</span>
    <Show when={sel()}>{s => <span class="muted">{s().kind === "node" ? `Node ${s().id}` : s().kind === "element" ? (() => { const e = model()?.elements.find(x => x.id === s().id); return e ? `Element ${e.from}–${e.to}` : ""; })() : s().kind}</span>}</Show>
    <span class="grow" />
    <span class="muted">{model()?.units === "US" ? "US units" : "SI units"}</span>
    <span class="vsc"><For each={SIDE.filter(x => x !== "-") as [View, string, string][]}>{x => <button title={x[2]} onClick={() => setView(x[0])}><Icon n={x[1]} /></button>}</For></span>
  </footer>
);

const App = () => {
  onMount(async () => {
    try {
      const o = await invoke<Options>("options"); setOpts(o); setThemeRaw(o.theme || "auto"); applyTheme();
      setUnitTable(await invoke<[Shown[], Shown[], Typed[]]>("unit_table"));
      setOut(await invoke<Out>("state"));
      const start = await invoke<string | null>("startup_file");
      if (start) { await loadOut("open", { path: start }); setTimeout(C.fit, 80); }
      const rec = await invoke<{ name: string; saved: string } | null>("recovery");
      if (rec) D.recoverDialog(rec.name, rec.saved, () => { setBackstage(null); setTimeout(C.fit, 60); });
      checkUpdate(true);
      setInterval(() => { if (opts().autosave > 0) invoke("autosave").catch(() => {}); }, Math.max(1, opts().autosave) * 60000);
      listen("close-requested", () => D.confirmClose(() => invoke("quit")));
    } catch (e) { toast(String(e)); }
  });
  const keys = (e: KeyboardEvent) => {
    const ctrl = e.ctrlKey || e.metaKey, k = e.key.toLowerCase();
    if (dialogs().length) return;
    const typing = e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement || e.target instanceof HTMLSelectElement;
    if ((ctrl && k === "k") || (e.altKey && k === "q")) { e.preventDefault(); palette(); }
    else if (ctrl && k === "s") { e.preventDefault(); C.save(e.shiftKey); }
    else if (ctrl && k === "o") { e.preventDefault(); C.open(); }
    else if (ctrl && k === "n") { e.preventDefault(); setBackstage("home"); }
    else if (ctrl && k === "p") { e.preventDefault(); C.print(); }
    else if (ctrl && k === "z" && !typing) { e.preventDefault(); C.undo(); }
    else if (ctrl && k === "y" && !typing) { e.preventDefault(); C.redo(); }
    else if (ctrl && e.key === "Enter" && !backstage()) { e.preventDefault(); C.addAfter(); }
    else if (e.key === "F1") { e.preventDefault(); D.shortcuts(); }
    else if (e.key === "F5") { e.preventDefault(); C.analyze(); }
    else if (e.key === "Delete" && !typing && !backstage()) { e.preventDefault(); C.del(); }
    else if (!typing && !ctrl && !backstage() && view() === "model" && k === "f") C.fit();
    else if (!typing && !ctrl && !backstage() && view() === "model" && "1234".includes(k) && k.length === 1) C.std(["iso", "plan", "front", "side"][+k - 1]);
    else if (e.key === "Escape" && backstage() && out()) setBackstage(null);
  };
  return (
    <div class="app" ref={() => document.addEventListener("keydown", keys)}>
      <TitleBar update={update()?.available ? update()!.latest : null} />
      <Ribbon />
      <main class="work">
        <Flow />
        <div class="content">
          <Show when={out()}>
            <Switch fallback={<div class="modelview"><div class="center"><View3D /><ElementTable /></div><Inspector /></div>}>
              <Match when={view() === "results"}><ResultsPage /></Match>
              <Match when={view() === "iso"}><IsoPage /></Match>
              <Match when={view() === "report"}><ReportPage /></Match>
              <Match when={view() === "vessel"}><VesselPage /></Match>
            </Switch>
          </Show>
        </div>
      </main>
      <Status />
      <Show when={backstage()}><Backstage /></Show>
      <For each={dialogs()}>{d => d()}</For>
      <Menu /><Toast />
    </div>
  );
};

render(() => <App />, document.getElementById("app")!);
