// The 3D view: arrives in 0.1.1 (three.js). Until then, the table is the model.
import { Show } from "solid-js";
import { model } from "./store";
import { fmt } from "./units";

export const View3D = () => (
  <div class="v3d">
    <div class="empty"><b>The 3D view arrives in 0.1.1</b>
      <span>Type the piping in the element table below: each row is one element from a node to the next, with its lengths along X, Y (up) and Z.</span>
      <Show when={model()?.nodes.length}><span class="muted">{model()!.nodes.length} nodes; the model spans {(() => { const n = model()!.nodes; const ext = [0, 1, 2].map(i => Math.max(...n.map(q => [q.x, q.y, q.z][i])) - Math.min(...n.map(q => [q.x, q.y, q.z][i]))); return ext.map(v => fmt("coord", v)).join(" × "); })()}.</span></Show>
    </div>
  </div>
);
