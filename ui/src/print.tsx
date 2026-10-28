// File › Print: settings on the left, a live page preview on the right (FAMILY §8). Pages are laid
// out at real size and printed with the WebView's print dialog, where "Save as PDF" makes a PDF.
// Built on Datum Plan's print page (paper sizes, header and footer slots and codes, @page), by way
// of Datum Mesh. The stress report flows over as many pages as it needs, with the header and
// footer on every page.
import { For, Show, JSX, createSignal, createMemo, createEffect, on, onMount } from "solid-js";
import { Portal } from "solid-js/web";
import { createStore } from "solid-js/store";
import { Model } from "./api";
import { out, opts, call, toast, model, setBackstage } from "./store";
import { Help } from "./help";
import { Pick, Check } from "./ui";
import { ReportBody, reportExtras } from "./report";

export const PAPERS: [string, number, number][] = [["A4", 210, 297], ["A3", 297, 420], ["A2", 420, 594], ["Letter", 215.9, 279.4], ["Legal", 215.9, 355.6], ["Tabloid", 279.4, 431.8]];
export const MM = 96 / 25.4;
type Slots = [string, string, string];
type Setup = { paper: string; landscape: boolean; margins: [number, number, number, number]; header: Slots; footer: Slots; report: boolean; iso: boolean };
export const CODES: [string, string][] = [["&[Page]", "Page number"], ["&[Pages]", "Total pages"], ["&[Date]", "Today's date"], ["&[Model]", "Model name"], ["&[Company]", "Company"], ["&[File]", "File name"], ["&[Line]", "Line number"]];
export const [ps, setPs] = createStore<Setup>({
  paper: "A4", landscape: false, margins: [12, 12, 12, 12], header: ["&[Company]", "&[Model]", "&[Line]"], footer: ["Datum Pipe", "Page &[Page] of &[Pages]", "&[Date]"],
  report: true, iso: true,
});
const codes = (t: string, page: number, pages: number) => t.replace(/&\[Page\]/g, String(page)).replace(/&\[Pages\]/g, String(pages)).replace(/&\[Date\]/g, new Date().toLocaleDateString())
  .replace(/&\[Model\]/g, out()?.m.name ?? "").replace(/&\[Company\]/g, opts().company).replace(/&\[File\]/g, out()?.path?.split(/[\\/]/).pop() ?? "not saved").replace(/&\[Line\]/g, out()?.m.project.line ?? "");

export const [version, setVersion] = createSignal("");

/** The title block of the report and the drawings. */
export const TitleBlock = (p: { m: Model; title?: string }) => (
  <table class="tblock"><tbody>
    <tr><td colSpan={2}><small>Project</small><b>{p.m.project.project || p.m.name}</b></td><td><small>Line</small>{p.m.project.line || "—"}</td><td><small>Calc number</small>{p.m.project.calc || "—"}</td><td><small>Revision</small>{p.m.project.rev || "—"}</td></tr>
    <tr><td><small>Document</small>{p.title ?? "Pipe stress report"}</td><td><small>By</small>{p.m.project.by || "—"}</td><td><small>Checked</small>{p.m.project.checked || "—"}</td><td><small>Date</small>{new Date().toLocaleDateString()}</td><td><small>Datum Pipe</small>{version()}</td></tr>
  </tbody></table>
);

/** One printed page: header, body, footer. */
const Sheet = (p: { body: JSX.Element; n: number; pages: number; w: number; h: number }) => (
  <div class="ppage" style={{ width: `${p.w}px`, height: `${p.h}px` }}>
    <div class="pin" style={{ left: `${ps.margins[3] * MM}px`, top: `${ps.margins[0] * MM}px`, width: `${p.w - (ps.margins[1] + ps.margins[3]) * MM}px`, height: `${p.h - (ps.margins[0] + ps.margins[2]) * MM}px` }}>
      <div class="phead"><For each={ps.header}>{(t, i) => <span class={`ps${i()}`}>{codes(t, p.n, p.pages)}</span>}</For></div>
      <div class="pbody">{p.body}</div>
      <div class="pfoot"><For each={ps.footer}>{(t, i) => <span class={`ps${i()}`}>{codes(t, p.n, p.pages)}</span>}</For></div>
    </div>
  </div>
);

/**
 * Splits the report into pages: the report is laid out once off screen in a column as wide as the
 * page, and its blocks (sections, tables, pictures) are packed onto pages by their heights.
 */
const paginate = (el: HTMLElement, h: number): number[][] => {
  const blocks = [...(el.firstElementChild?.children ?? [])] as HTMLElement[];
  const pages: number[][] = [[]];
  let used = 0;
  blocks.forEach((b, i) => {
    const bh = b.getBoundingClientRect().height + 6;
    if (used + bh > h && pages[pages.length - 1].length) { pages.push([]); used = 0; }
    pages[pages.length - 1].push(i);
    used += bh;
  });
  return pages;
};

const [printing, setPrinting] = createSignal(false);

export const PrintPage = () => {
  setPs({ paper: opts().paper || "A4", landscape: opts().landscape === true });
  const [zoomP, setZoomP] = createSignal(0.55);
  const size = createMemo(() => { const p = PAPERS.find(x => x[0] === ps.paper) ?? PAPERS[0]; const [w, h] = ps.landscape ? [p[2], p[1]] : [p[1], p[2]]; return { pw: w, ph: h, w: w * MM, h: h * MM }; });
  const inner = () => ({ w: size().w - (ps.margins[1] + ps.margins[3]) * MM, h: size().h - (ps.margins[0] + ps.margins[2]) * MM - 84 });
  const [split, setSplit] = createSignal<number[][]>([]);
  let measure!: HTMLDivElement;
  const relayout = () => requestAnimationFrame(() => { if (measure) setSplit(ps.report ? paginate(measure, inner().h) : []); });
  onMount(async () => { setVersion(await call<string>("version") ?? ""); relayout(); });
  const blocks = createMemo(() => { const m = model(); return m && ps.report ? ReportBody({ m }) : []; });
  createEffect(on(blocks, relayout, { defer: true }));
  const extra = createMemo(() => { const m = model(); return m && ps.iso ? reportExtras(m, inner().w, inner().h) : []; });
  const pages = createMemo<JSX.Element[]>(() => {
    const b = blocks(); const out: JSX.Element[] = split().map(ix => <div class="preport">{ix.map(i => b[i]?.())}</div>);
    return [...out, ...extra()];
  });
  const doPrint = () => {
    setPrinting(true);
    const css = document.createElement("style");
    css.textContent = `@page { size: ${size().pw}mm ${size().ph}mm; margin: 0 }`;
    document.head.append(css);
    document.documentElement.classList.add("printing");
    setTimeout(() => { window.print(); document.documentElement.classList.remove("printing"); css.remove(); setPrinting(false); }, 250);
  };
  return (
    <div class="printpage">
      <div class="pset">
        <h1>Print <Help k="report" /></h1>
        <div class="pbtns"><button class="btn pri big" onClick={doPrint}><svg class="ic"><use href="#i-print" /></svg> Print / Save as PDF</button></div>
        <p class="muted">In the print window choose your printer, or <b>Save as PDF</b>. {pages().length} {pages().length === 1 ? "page" : "pages"} · {ps.paper} {ps.landscape ? "landscape" : "portrait"}.</p>
        <h4>What to print</h4>
        <Check label="The stress report" value={ps.report} set={v => { setPs("report", v); relayout(); }} />
        <Check label="The isometric drawing" value={ps.iso} set={v => setPs("iso", v)} />
        <div class="prow"><label>Paper<Pick value={ps.paper} set={v => { setPs("paper", v); relayout(); }} options={PAPERS.map(p => p[0])} /></label><label>Orientation<Pick value={ps.landscape ? "1" : "0"} set={v => { setPs("landscape", v === "1"); relayout(); }} options={[["0", "Portrait"], ["1", "Landscape"]]} /></label></div>
        <h4>Header and footer</h4>
        <For each={["header", "footer"] as const}>{k => <div class="pslots"><b>{k === "header" ? "Header" : "Footer"}</b>
          <For each={["Left", "Center", "Right"]}>{(l, i) => <label><span>{l}</span><textarea rows="1" value={ps[k][i()]} onInput={e => setPs(k, i(), e.currentTarget.value)} /></label>}</For></div>}</For>
        <p class="muted pcodes">Codes: <For each={CODES}>{([c, l]) => <code title={l} onClick={() => navigator.clipboard?.writeText(c).then(() => toast(`${c} copied.`, true))}>{c}</code>}</For></p>
        <div class="prow">{(["Top", "Right", "Bottom", "Left"] as const).map((l, i) => <label>{l} (mm)<input type="number" min="0" max="60" value={ps.margins[i]} onChange={e => { setPs("margins", i, +e.currentTarget.value); relayout(); }} /></label>)}</div>
        <p><button class="btn sm" onClick={() => setBackstage(null)}>Back to the model</button></p>
      </div>
      <div class="pprev">
        <div class="pzoom"><span class="muted">Preview</span><input type="range" min="0.25" max="1.2" step="0.05" value={zoomP()} onInput={e => setZoomP(+e.currentTarget.value)} /><span class="muted">{Math.round(zoomP() * 100)}%</span></div>
        <div class="pmeasure" ref={measure} style={{ width: `${inner().w}px` }}><div class="preport">{blocks().map(b => b())}</div></div>
        <For each={pages()} fallback={<p class="pempty">Nothing chosen to print.</p>}>{(pg, i) =>
          <div class="pwrap" style={{ width: `${size().w * zoomP()}px`, height: `${size().h * zoomP()}px` }}><div style={{ transform: `scale(${zoomP()})`, "transform-origin": "0 0" }}><Sheet body={pg} n={i() + 1} pages={pages().length} w={size().w} h={size().h} /></div><small>Page {i() + 1} of {pages().length}</small></div>}</For>
      </div>
      <Show when={printing()}><Portal mount={document.body}><div id="printroot"><For each={pages()}>{(pg, i) => <Sheet body={pg} n={i() + 1} pages={pages().length} w={size().w} h={size().h} />}</For></div></Portal></Show>
    </div>
  );
};
