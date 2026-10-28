// Display units. The engine and the file use SI base units (m, kg, N, Pa, K); the engine also sends
// how every quantity is shown in SI and US units (crates/core/src/units.rs, from Datum Calc's unit
// table), so this file holds no conversion factors of its own: it only applies them.
import { createSignal } from "solid-js";
import { out } from "./store";
import { num, Shown } from "./api";

export type Q = "coord" | "len" | "disp" | "rot" | "force" | "moment" | "stress" | "press" | "modulus" | "temp" | "dtemp" | "density" | "expansion" | "mass" | "wlen" | "tstiff" | "rstiff" | "radius" | "ratio";

const [tables, setTables] = createSignal<[Shown[], Shown[]] | null>(null);
export const setUnitTable = (t: [Shown[], Shown[]]) => setTables(t);

const row = (q: Q, us?: boolean): Shown => {
  const t = tables(); const sys = us ?? out()?.m.units === "US";
  return t?.[sys ? 1 : 0].find(s => s.q === q) ?? { q, label: "", factor: 1, offset: 0 };
};

/** SI value → display value. */
export const toU = (q: Q, v: number) => { const r = row(q); return v / r.factor - r.offset; };
/** Display value → SI value. */
export const fromU = (q: Q, v: number) => { const r = row(q); return (v + r.offset) * r.factor; };
export const unit = (q: Q, us?: boolean) => row(q, us).label;
/** A value with its unit, for labels. */
export const fmt = (q: Q, v: number, sig = 4) => `${num(toU(q, v), sig)}${unit(q) ? " " + unit(q) : ""}`;
/** A temperature difference or rise has no offset. */
export const fmtD = (q: Q, v: number, sig = 4) => `${num(v / row(q).factor, sig)}${unit(q) ? " " + unit(q) : ""}`;
