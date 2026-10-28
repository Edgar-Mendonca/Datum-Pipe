// Results: the ribbon's Results tab and the Results view (tables per load case). The solver
// arrives in 0.1.1; until then the tools are shown with the release they arrive in.
import { G, S, Planned } from "./ribbon";

export const ResultsRibbon = () => <>
  <G l="Load Case" h="results"><Planned big i="cases" l="Load Case" v="0.1.2" /></G>
  <G l="Show" h="results"><S><Planned i="contour" l="Stress Ratio Colours" v="0.1.2" /><Planned i="deform" l="Deformed Shape" v="0.1.2" /><Planned i="play" l="Animate" v="0.1.2" /></S></G>
  <G l="Tables" h="results"><S><Planned i="table" l="Displacements" v="0.1.2" /><Planned i="table" l="Support Loads" v="0.1.2" /><Planned i="table" l="Code Stresses" v="0.1.2" /></S></G>
</>;

export const ResultsPage = () => (
  <div class="page"><h1>Results</h1><div class="emptypage"><b>No results yet</b><span>Analysis arrives in Datum Pipe 0.1.1: displacements, forces, support loads and the equilibrium check; code stresses in 0.1.2.</span></div></div>
);
