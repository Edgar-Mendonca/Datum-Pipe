// The isometric drawing: arrives in 0.1.3.
import { G, Planned } from "./ribbon";

export const IsoRibbon = () => <>
  <G l="Isometric" h="iso"><Planned big i="iso" l="Isometric" v="0.1.3" /><Planned big i="print" l="Print" v="0.1.3" /></G>
</>;

export const IsoPage = () => (
  <div class="page"><h1>Isometric</h1><div class="emptypage"><b>The isometric drawing arrives in 0.1.3</b><span>An automatic 30° isometric of the model with bends, tees, valves, flanges, supports, node numbers and dimensions, ready to print.</span></div></div>
);
