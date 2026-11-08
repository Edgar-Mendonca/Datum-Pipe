// The bottom panel's other tabs (0.6.1): Messages (everything the window said, kept with its time)
// and the solver's log (one line per solve). The element table is its first tab (table.tsx).
import { For, Show } from "solid-js";
import { log, setLog, ptab, setPtab, unread, Note } from "./store";
import { solverLog, setSolverLog } from "./rstate";

const KIND: Record<Note["kind"], string> = { note: "Note", ok: "Done", warn: "Check", edit: "Edit", solve: "Solve" };
const hhmm = (d: Date) => d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });

/** The panel's tabs; `extra` sits on the right (the element table's facts). */
export const PanelTabs = () => (
  <nav class="ptabs">
    <For each={[["elements", "Elements"], ["messages", "Messages"], ["log", "Solver log"]]}>{([k, l]) =>
      <button classList={{ on: ptab() === k }} onClick={() => setPtab(k)}>{l}<Show when={k === "messages" && unread() > 0}><i class="badge">{unread()}</i></Show></button>}</For>
  </nav>
);

export const Messages = () => (
  <div class="pbody msgs">
    <Show when={log().length} fallback={<p class="muted pad">Nothing yet. Every edit, solve, warning and note the window gives is kept here with its time.</p>}>
      <div class="pbar"><span class="muted">{log().length} messages, newest last</span><span class="grow" /><button class="btn sm" onClick={() => setLog([])}>Clear</button></div>
      <For each={log()}>{m => <div class={`msg ${m.kind}`}><time>{hhmm(m.t)}</time><b>{KIND[m.kind]}</b><span>{m.text}</span></div>}</For>
    </Show>
  </div>
);

export const SolverLog = () => (
  <div class="pbody slog">
    <Show when={solverLog().length} fallback={<p class="muted pad">No solve yet. Analyze (F5): each solve adds a line with the cases, the equations and the time it took.</p>}>
      <div class="pbar"><span class="muted">One line per solve</span><span class="grow" /><button class="btn sm" onClick={() => setSolverLog([])}>Clear</button></div>
      <pre>{solverLog().join("\n")}</pre>
    </Show>
  </div>
);
