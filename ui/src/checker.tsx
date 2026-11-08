// The model checker (0.6.7): one list of what is wrong or worth a look before (and after) an
// analysis, each with the way to it. It reads what the engine already reports (the geometry's
// errors and warnings, the last analysis's warnings and failure) and adds a few plain checks of
// the model: nothing holding the piping, free ends, materials without allowable stresses.
import { For, Show } from "solid-js";
import { model, out, openDialog, setSel, setView } from "./store";
import { results, stale, solverLog } from "./rstate";
import { Dialog } from "./ui";
import { setCardOpen } from "./tree";

export type Finding = { level: "error" | "warn" | "info"; text: string; node?: number };

export const findings = (): Finding[] => {
  const m = model(), o = out(); if (!m || !o) return [];
  const f: Finding[] = [];
  if (!m.elements.length) return [{ level: "info", text: "The model has no elements yet: type the first one in the table." }];
  if (o.geom_error) f.push({ level: "error", text: o.geom_error });
  for (const w of o.geom?.warnings ?? []) f.push({ level: "warn", text: w });
  if (!m.supports.length && !m.nozzles.length) f.push({ level: "error", text: "Nothing holds the piping: add an anchor, a nozzle point or supports." });
  // Free ends: a node on one element with nothing at it (an open end is fine; say so).
  const count = new Map<number, number>();
  for (const e of m.elements) { count.set(e.from, (count.get(e.from) ?? 0) + 1); count.set(e.to, (count.get(e.to) ?? 0) + 1); }
  const held = new Set([...m.supports.map(s => s.node), ...m.nozzles.map(z => z.node), ...(m.reliefs ?? []).map(r => r.node)]);
  for (const [n, k] of count) if (k === 1 && !held.has(n)) f.push({ level: "info", text: `Node ${n} is a free end: nothing holds it there. Fine for an open end; otherwise add a support or a nozzle point.`, node: n });
  for (const mat of m.materials) if (!mat.allow.length && m.elements.some(e => e.material === mat.id)) f.push({ level: "warn", text: `${mat.name} has no allowable stresses: its pipe's code stresses cannot be checked.` });
  const last = solverLog()[solverLog().length - 1];
  if (last?.includes("not solved:")) f.push({ level: "error", text: `The last analysis failed: ${last.split("not solved:")[1].trim()}`, node: +(/at node (\d+)/.exec(last)?.[1] ?? NaN) || undefined });
  const r = results();
  if (r && stale()) f.push({ level: "info", text: "The model changed since the last analysis: press F5 to analyze again." });
  if (r && !stale()) for (const w of r.warnings) f.push({ level: "warn", text: w, node: +(/node (\d+)/.exec(w)?.[1] ?? NaN) || undefined });
  return f;
};

export const checkModel = () => openDialog(() => {
  const list = () => findings();
  const show = (n: number) => { setSel({ kind: "node", id: n }); setView("model"); setCardOpen(true); };
  return (
    <Dialog title="Check the model" wide help="analyze">
      <Show when={list().length} fallback={<p class="okline">Nothing to report: every check passes.</p>}>
        <p class="muted">{list().filter(x => x.level === "error").length} to fix, {list().filter(x => x.level === "warn").length} to check, {list().filter(x => x.level === "info").length} to note.</p>
        <div class="findings"><For each={list()}>{x => <div class={`finding ${x.level}`}><i>{x.level === "error" ? "Fix" : x.level === "warn" ? "Check" : "Note"}</i><span>{x.text}</span><Show when={x.node}><button class="btn sm" onClick={() => show(x.node!)}>Show</button></Show></div>}</For></div>
      </Show>
    </Dialog>
  );
});
