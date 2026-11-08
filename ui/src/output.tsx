// Output (0.6.8): the cases and the reports chosen, written as one spreadsheet file (CSV, opened by
// Excel and every spreadsheet) in the units shown. Each report is a block with its own heading row.
import { For, createSignal } from "solid-js";
import { model, openDialog, call, toast, closeDialog } from "./store";
import { results, stale } from "./rstate";
import { Dialog } from "./ui";
import { toU, unit } from "./units";
import { num } from "./api";

const REPORTS: [string, string][] = [["stress", "Code stresses"], ["disp", "Movements"], ["support", "Support loads"], ["nozzle", "Nozzle loads"]];

/** One CSV cell: quoted when it needs to be, and never read as a formula (a leading = + - @ is
 *  kept as text, FAMILY's formula guard). */
const cell = (v: string | number): string => {
  let s = typeof v === "number" ? (Number.isFinite(v) ? num(v, 6) : "") : v;
  if (typeof v === "string" && /^[=+\-@]/.test(s)) s = "'" + s;
  return /[",\n;]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s;
};
const row = (xs: (string | number)[]) => xs.map(cell).join(",");

/** The CSV for the cases and reports chosen. */
export const toCsv = (cases: number[], reports: string[]): string => {
  const r = results(), m = model(); if (!r || !m) return "";
  const lab = (pt: number) => r.geom.points[pt]?.label ?? "";
  const tag = (id: number) => m.supports.find(s => s.id === id)?.tag ?? "";
  const zname = (id: number) => m.nozzles.find(z => z.id === id)?.name ?? "";
  const lines: string[] = [row([`Datum Pipe ${r.version}`, m.name, m.project?.line ?? "", new Date().toISOString().slice(0, 16).replace("T", " ")])];
  const chosen = r.cases.filter(c => cases.includes(c.id));
  if (reports.includes("stress")) {
    lines.push("", row(["Code stresses"]), row(["Case", "Point", `Stress (${unit("stress")})`, `Allowable (${unit("stress")})`, "Ratio", "Result", "SIF in-plane", "SIF out-of-plane"]));
    for (const c of chosen) for (const s of c.stresses) lines.push(row([c.name, lab(s.point), toU("stress", s.stress), toU("stress", s.allow), s.ratio, s.pass ? "PASS" : "FAIL", s.ii, s.io]));
  }
  if (reports.includes("disp")) {
    lines.push("", row(["Movements"]), row(["Case", "Node", `X (${unit("disp")})`, "Y", "Z", `RX (${unit("rot")})`, "RY", "RZ"]));
    for (const c of chosen) for (const [n, pt] of Object.entries(r.geom.node_point)) { const d = c.disp[pt]; if (d) lines.push(row([c.name, n, ...[0, 1, 2].map(k => toU("disp", d[k])), ...[3, 4, 5].map(k => toU("rot", d[k]))])); }
  }
  if (reports.includes("support")) {
    lines.push("", row(["Support loads (what each support puts on the pipe, global axes, Y up, as in the Support Loads table)"]), row(["Case", "Support", "Node", `FX (${unit("force")})`, "FY", "FZ", `MX (${unit("moment")})`, "MY", "MZ"]));
    for (const c of chosen) for (const x of c.reactions) lines.push(row([c.name, tag(x.support), x.node, ...[0, 1, 2].map(k => toU("force", x.f[k])), ...[3, 4, 5].map(k => toU("moment", x.f[k]))]));
  }
  if (reports.includes("nozzle")) {
    lines.push("", row(["Nozzle loads (in the nozzle's axes)"]), row(["Case", "Nozzle", "Node", `Axial (${unit("force")})`, "Shear", `Torsion (${unit("moment")})`, "Bending", "Result"]));
    for (const c of chosen) for (const z of c.nozzles) lines.push(row([c.name, zname(z.nozzle), z.node, toU("force", z.local[0]), toU("force", z.shear), toU("moment", z.local[3]), toU("moment", z.bending), z.pass === null ? "" : z.pass ? "PASS" : "FAIL"]));
  }
  return lines.join("\r\n") + "\r\n";
};

export const outputDialog = () => {
  const r = results();
  if (!r) return toast("Analyze (F5) first: the output is the results of the cases you choose.");
  openDialog(() => {
    const [cases, setCases] = createSignal<number[]>(r.cases.filter(c => c.kind !== "ope" || r.cases.length < 6).map(c => c.id));
    const [reps, setReps] = createSignal<string[]>(["stress", "support"]);
    const flip = <T,>(l: T[], x: T) => l.includes(x) ? l.filter(y => y !== x) : [...l, x];
    const save = async () => {
      if (!cases().length || !reps().length) { toast("Choose at least one case and one report."); return false; }
      const path = await call<string | null>("save_file", { name: `${model()?.name || "piping"} results.csv`, data: toCsv(cases(), reps()) });
      if (path) { toast("Saved the results as a spreadsheet file (CSV): it opens in Excel.", true); closeDialog(); }
      return false;
    };
    return (
      <Dialog title="Output: cases and reports" ok={save} okText="Save as a spreadsheet" help="results">
        <p class="muted">Tick the load cases and the reports; they are written as one spreadsheet file (CSV, which Excel opens) in the units shown.{stale() ? " The model changed since this analysis: press F5 first for results that match it." : ""}</p>
        <div class="outgrid">
          <div><b class="cap">Load cases</b><For each={r.cases}>{c => <label class="chk"><input type="checkbox" checked={cases().includes(c.id)} onChange={() => setCases(flip(cases(), c.id))} />{c.name}</label>}</For>
            <p class="ibtns"><button class="btn sm" onClick={() => setCases(r.cases.map(c => c.id))}>All</button><button class="btn sm" onClick={() => setCases([])}>None</button></p></div>
          <div><b class="cap">Reports</b><For each={REPORTS}>{([k, l]) => <label class="chk"><input type="checkbox" checked={reps().includes(k)} onChange={() => setReps(flip(reps(), k))} />{l}</label>}</For></div>
        </div>
      </Dialog>
    );
  });
};
