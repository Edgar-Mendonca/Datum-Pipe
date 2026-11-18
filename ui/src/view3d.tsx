// The 3D view (three.js, MIT): the piping at its true diameter from the engine's analysis geometry
// (bends as arcs), insulation as a translucent sleeve, valves, flanges, reducers, tees, support and
// nozzle symbols, node numbers; orbit, pan, zoom, fit and the standard views; click to select (the
// table follows). After an analysis: colours by stress ratio with a legend, and the deformed shape
// with a scale, animated between installed and operating. The window draws what the engine gives:
// positions, arcs and results all come from it.
import { onMount, onCleanup, createEffect, createSignal, on, Show, For } from "solid-js";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment.js";
import { Geom, Model, Seg, V3, num } from "./api";
import { out, model, sel, setSel, opts, themeTick, openMenu, act, Item, note } from "./store";
import { cardOpen } from "./tree";
import { C, setFit, setStdView, SUPPORTS } from "./cmds";
import { COMPS, compFrom, compKey } from "./table";
import { results, stale, show, curCase, segRatios, setShowPart, modeNo, setModeNo, colourBy, setColourBy, drawMode, setDrawMode, setCaseId, ColourBy, speed, setSpeed } from "./rstate";
import { fmt } from "./units";

type Vec = [number, number, number];
const sub = (a: Vec, b: Vec): Vec => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const addv = (a: Vec, b: Vec): Vec => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
const mul = (a: Vec, s: number): Vec => [a[0] * s, a[1] * s, a[2] * s];
const cross = (a: Vec, b: Vec): Vec => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const len = (a: Vec) => Math.hypot(a[0], a[1], a[2]);
const unitv = (a: Vec): Vec => { const n = len(a); return n > 0 ? mul(a, 1 / n) : [0, 0, 0]; };
const V = (a: Vec) => new THREE.Vector3(a[0], a[1], a[2]);
const IDENTITY = new THREE.Matrix4();

/** A point along a segment at fraction s, and the direction there (the engine's arc rule). */
export const segAt = (g: Geom, s: Seg, f: number): [Vec, Vec] => {
  const pa = g.points[s.a].p as Vec, pb = g.points[s.b].p as Vec;
  if (s.shape.kind === "straight") return [addv(pa, mul(sub(pb, pa), f)), unitv(sub(pb, pa))];
  const { centre, normal, r, angle } = s.shape as { centre: Vec; normal: Vec; r: number; angle: number };
  const ra = unitv(sub(pa, centre)), t0 = cross(normal, ra), phi = angle * f;
  const rp = addv(mul(ra, Math.cos(phi)), mul(t0, Math.sin(phi)));
  return [addv(centre, mul(rp, r)), unitv(cross(normal, rp))];
};

/** Stress ratio colours: blue (low) to red (1.0), magenta above; or viridis. */
export const ratioColour = (r: number, scheme = opts().contours) => {
  if (r < 0) return "#9AA5B1";
  if (r > 1) return "#B0127A";
  const stops = scheme === "viridis" ? ["#440154", "#3B528B", "#21918C", "#5DC863", "#FDE725"] : ["#2B6FD6", "#2BB3D6", "#3BB273", "#E3B505", "#D64545"];
  const k = Math.min(stops.length - 1, Math.floor(r * stops.length));
  return stops[k];
};

const css = (v: string) => getComputedStyle(document.documentElement).getPropertyValue(v).trim() || "#fff";

class ArcCurve extends THREE.Curve<THREE.Vector3> {
  constructor(private f: (t: number) => Vec) { super(); }
  getPoint(t: number, target = new THREE.Vector3()) { const p = this.f(t); return target.set(p[0], p[1], p[2]); }
}

let fitNow: () => void = () => {};

/** The family's colour-blind-safe series colours (FAMILY §7, Okabe-Ito). */
const SERIES = ["#0072B2", "#E69F00", "#009E73", "#D55E00", "#CC79A7", "#56B4E9", "#F0E442", "#000000"];
/** Colour by a property (0.6.4): each segment's category and the legend's rows. Temperatures are
 *  ordered and coloured cold to hot; the rest take the series colours in the model's order. */
export const categories = (m: Model | undefined, g: Geom | null | undefined, cb: string): { colourOf: (si: number) => string; items: [string, string][] } | null => {
  if (!m || !g || !["size", "material", "cond", "temp", "soil"].includes(cb)) return null;
  const el = new Map(m.elements.map(e => [e.id, e]));
  const key = (si: number): string => {
    const sg = g.segs[si];
    if (cb === "size") return m.pipes.find(p => p.id === sg.pipe)?.name ?? "?";
    if (cb === "material") return m.materials.find(x => x.id === sg.material)?.name.split(",")[0] ?? "?";
    if (cb === "cond") return m.conds.find(x => x.id === sg.cond)?.name ?? "?";
    if (cb === "temp") { const c = m.conds.find(x => x.id === sg.cond); return c ? `${fmt("temp", c.t[0])}` : "?"; }
    const s = el.get(sg.element)?.soil; return s ? `Buried in ${m.soils?.find(x => x.id === s)?.name ?? "soil"}` : "Above ground";
  };
  const keys = g.segs.map((_, si) => key(si));
  let order = [...new Set(keys)];
  if (cb === "temp") { const tOf = (k: string) => m.conds.find(c => fmt("temp", c.t[0]) === k)?.t[0] ?? 0; order = order.sort((a, b) => tOf(a) - tOf(b)); }
  const colour = new Map(order.map((k, i) => [k, cb === "temp" ? ratioColour(order.length > 1 ? i / (order.length - 1) : 0.5) : cb === "soil" && k === "Above ground" ? "#9AA5B1" : SERIES[i % SERIES.length]]));
  return { colourOf: si => colour.get(keys[si])!, items: order.map(k => [k, colour.get(k)!]) };
};

/** Drawn again from nothing when the graphics card resets and WebGL does not come back by itself
 *  (a driver update, sleep; 0.7.1: the view stayed black until another view was opened). */
const [glGen, setGlGen] = createSignal(1);
export const View3D = () => <Show when={glGen()} keyed>{_ => <View3DInner />}</Show>;

const View3DInner = () => {
  let host!: HTMLDivElement, labels!: HTMLDivElement;
  let renderer: THREE.WebGLRenderer | null = null;
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(35, 1, 0.01, 1e5);
  let controls: OrbitControls | null = null;
  let group = new THREE.Group();
  let pickables: THREE.Object3D[] = [];
  let anim = 0, animT = 0, modeShown = false;
  /** Vertices to move for the deformed shape: [attribute, base positions, displacement per vertex]. */
  let deform: [THREE.BufferAttribute, Float32Array, Float32Array][] = [];
  let ampl = 1;
  let failed = false;
  let dark = false;
  // The look (0.7.3): a soft room's light reflected by the steel (set once the renderer exists), a
  // low sky light and a key light from over the viewer's shoulder, a ground that shades from light
  // to a little darker towards the bottom, and dark outlines round every solid.
  const sky = new THREE.HemisphereLight(0xffffff, 0x8899aa, 0.55);
  scene.add(sky);
  let roomLight: THREE.Texture | null = null;
  const sun = new THREE.DirectionalLight(0xffffff, 1.1);
  sun.position.set(-2, 4, 3);
  camera.add(sun);
  scene.add(camera);
  let bgKey = "";
  /** The ground: a vertical shade in the theme's colours, made again only when the theme changes. */
  const ground = () => {
    const [top, bot] = dark ? ["#1B2633", "#0C121A"] : ["#FFFFFF", "#DCE2E7"];
    if (bgKey === top + bot) return;
    bgKey = top + bot;
    const c = document.createElement("canvas"); c.width = 2; c.height = 256;
    const x = c.getContext("2d")!, gr = x.createLinearGradient(0, 0, 0, 256);
    gr.addColorStop(0, top); gr.addColorStop(0.55, top); gr.addColorStop(1, bot);
    x.fillStyle = gr; x.fillRect(0, 0, 2, 256);
    (scene.background as THREE.Texture | null)?.dispose?.();
    const t = new THREE.CanvasTexture(c); t.colorSpace = THREE.SRGBColorSpace; scene.background = t;
  };
  /** Outlines: each solid drawn again from behind, pushed out by a pixel and a half on screen, in
   *  the ink colour (the way drawings show a part's edges). `px` is that push in clip units. */
  const px = { value: new THREE.Vector2(0.002, 0.003) };
  const outlineMat = (color: string) => {
    const m = new THREE.MeshBasicMaterial({ color: new THREE.Color(color), side: THREE.BackSide });
    m.onBeforeCompile = sh => {
      sh.uniforms.uPx = px;
      sh.vertexShader = sh.vertexShader.replace("#include <common>", "#include <common>\nuniform vec2 uPx;").replace("#include <project_vertex>", `#include <project_vertex>
  vec2 dn = (projectionMatrix * modelViewMatrix * vec4(normal, 0.0)).xy;
  if (length(dn) > 1e-6) gl_Position.xy += normalize(dn) * uPx * gl_Position.w;`);
    };
    return m;
  };
  // The floor grid (after Datum Mesh): big squares with small ones inside, sized to the zoom,
  // centred under what the view looks at and fading out round it; made again only when its squares'
  // size or place changes. It lies just under the lowest pipe.
  const floorGrp = new THREE.Group();
  scene.add(floorGrp);
  let floorY = 0, floorL = 10, floorKey = "", floorOn = false;
  const nice = (v: number) => { const p = 10 ** Math.floor(Math.log10(v)), k = v / p; return (k <= 1 ? 1 : k <= 2 ? 2 : k <= 5 ? 5 : 10) * p; };
  const adaptFloor = () => {
    floorGrp.visible = floorOn;
    if (!floorOn) return;
    const t = controls?.target ?? new THREE.Vector3(), d = Math.max(camera.position.distanceTo(t), 1e-3);
    const minor = nice(d / 12), mant = Math.round(minor / 10 ** Math.floor(Math.log10(minor) + 1e-9)), major = minor * (mant === 5 ? 10 : 5);
    const R = Math.ceil(Math.max(floorL * 0.8, d * 1.6) / major) * major, Rm = Math.min(R, d * 1.1);
    const cx = Math.round(t.x / major) * major, cz = Math.round(t.z / major) * major;
    const bgc = dark ? "#121A24" : "#F4F6F8", c2 = dark ? "#3A4A5C" : css("--line3");
    const key = `${minor}|${cx}|${cz}|${R}|${floorY}|${bgc}|${c2}`;
    if (key === floorKey) return;
    floorKey = key;
    floorGrp.children.forEach(o => { const x = o as THREE.LineSegments; x.geometry.dispose(); (x.material as THREE.Material).dispose(); });
    floorGrp.clear();
    const bg = new THREE.Color(bgc), cMin = new THREE.Color(c2).lerp(bg, 0.68), cMaj = new THREE.Color(c2).lerp(bg, 0.3);
    const pos: number[] = [], col: number[] = [];
    // A line in short pieces, each faded by its distance from the centre (out to `reach`).
    const seg = (a: [number, number], b: [number, number], c: THREE.Color, reach: number) => {
      const k = 16;
      for (let i = 0; i < k; i++) for (const u of [i / k, (i + 1) / k]) {
        const x = a[0] + (b[0] - a[0]) * u, z = a[1] + (b[1] - a[1]) * u, f = Math.min(1, Math.hypot(x - cx, z - cz) / reach) ** 1.5, cc = c.clone().lerp(bg, f);
        pos.push(x, floorY, z); col.push(cc.r, cc.g, cc.b);
      }
    };
    const nm = Math.ceil(Rm / minor), nM = Math.round(R / major), per = Math.round(major / minor);
    for (let i = -nm; i <= nm; i++) {
      if (i % per === 0) continue;
      const v = i * minor;
      seg([cx + v, cz - Rm], [cx + v, cz + Rm], cMin, Rm); seg([cx - Rm, cz + v], [cx + Rm, cz + v], cMin, Rm);
    }
    for (let i = -nM; i <= nM; i++) {
      const v = i * major;
      seg([cx + v, cz - R], [cx + v, cz + R], cMaj, R); seg([cx - R, cz + v], [cx + R, cz + v], cMaj, R);
    }
    const g = new THREE.BufferGeometry();
    g.setAttribute("position", new THREE.BufferAttribute(new Float32Array(pos), 3));
    g.setAttribute("color", new THREE.BufferAttribute(new Float32Array(col), 3));
    floorGrp.add(new THREE.LineSegments(g, new THREE.LineBasicMaterial({ vertexColors: true })));
  };
  // The view cube (after Datum Mesh): turns with the view; a click on a face looks from that side.
  let cubeEl: SVGSVGElement | undefined;
  const FACES: [string, Vec][] = [["Front", [0, 0, 1]], ["Back", [0, 0, -1]], ["Right", [1, 0, 0]], ["Left", [-1, 0, 0]], ["Top", [0, 1, 0]], ["Bottom", [0, -1, 0]]];
  const turnCube = () => {
    if (!cubeEl) return;
    const e = camera.matrixWorldInverse.elements, R = (i: number, j: number) => e[j * 4 + i];
    const rot = (v: number[]) => [0, 1, 2].map(i => R(i, 0) * v[0] + R(i, 1) * v[1] + R(i, 2) * v[2]);
    const order: [number, SVGGElement][] = [];
    FACES.forEach(([name, n]) => {
      const g = cubeEl!.querySelector(`[data-face="${name}"]`) as SVGGElement; if (!g) return;
      const a = n.findIndex(c => c !== 0), [u, w] = [0, 1, 2].filter(i => i !== a);
      const pts = [[-1, -1], [1, -1], [1, 1], [-1, 1]].map(([p, q]) => { const v = [0, 0, 0]; v[a] = n[a]; v[u] = p; v[w] = q; const r = rot(v); return `${(32 + 17 * r[0]).toFixed(1)},${(32 - 17 * r[1]).toFixed(1)}`; });
      const depth = rot(n)[2];
      g.style.display = depth > 0.02 ? "" : "none";
      g.querySelector("polygon")!.setAttribute("points", pts.join(" "));
      const t = g.querySelector("text")!, c = rot(n); t.setAttribute("x", (32 + 17 * c[0]).toFixed(1)); t.setAttribute("y", (34.5 - 17 * c[1]).toFixed(1));
      t.style.opacity = String(Math.min(1, depth * 1.6));
      order.push([depth, g]);
    });
    order.sort((x, y) => x[0] - y[0]).forEach(([, g]) => cubeEl!.appendChild(g));
  };

  const draw = () => {
    if (!renderer) return;
    if (deform.length && (show().anim || modeShown)) {
      // A load case swings between installed and operating; a mode about the still position.
      const f = modeShown ? Math.sin(animT * 1.6) : 0.5 - 0.5 * Math.cos(animT);
      for (const [attr, base, dv] of deform) { const a = attr.array as Float32Array; for (let i = 0; i < a.length; i++) a[i] = base[i] + f * ampl * dv[i]; attr.needsUpdate = true; }
    }
    camera.updateMatrixWorld(); turnCube(); adaptFloor();
    renderer.render(scene, camera);
    placeLabels();
  };
  const placeLabels = () => {
    const w = renderer?.domElement.clientWidth || host.clientWidth, h = host.clientHeight;
    for (const el of [...labels.children] as HTMLElement[]) {
      const p = new THREE.Vector3(+el.dataset.x!, +el.dataset.y!, +el.dataset.z!).project(camera);
      const vis = p.z < 1 && p.z > -1;
      el.style.display = vis ? "" : "none";
      el.style.transform = `translate(${(p.x + 1) / 2 * w + 6}px, ${(1 - p.y) / 2 * h - 16}px)`;
    }
  };

  /** The pipe's tubes kept from one build to the next (0.7.2): a change of colour, case or
   *  selection redraws without making a single tube again; an edit makes only the tubes it moved.
   *  Tubes that move with the deformed shape are made fresh, as their vertices are changed. */
  let geoCache = new Map<string, THREE.BufferGeometry>();
  /** One build per frame, however many of its inputs change together (0.7.2: an edit built the
   *  scene twice, once for the new model and again as its results went out of date). */
  let pendingBuild = 0;
  const later = () => { if (!pendingBuild) pendingBuild = requestAnimationFrame(() => { pendingBuild = 0; build(); }); };
  /** Builds the scene from the engine's geometry, the selection and the results. */
  const build = () => {
    const o = out(), m = model(); if (!o || !m) return;
    scene.remove(group); group.traverse(x => { const mm = x as THREE.Mesh; if (!mm.geometry?.userData.kept) mm.geometry?.dispose?.(); }); group = new THREE.Group(); scene.add(group);
    const lastGeo = geoCache; geoCache = new Map();
    let made = 0;
    pickables = []; deform = []; labels.innerHTML = "";
    // The ground follows the theme (Light, Dark or Auto, which follows Windows), and nothing
    // else: 0.6.4 turned the view navy whenever results were coloured, which read as the window
    // going dark by itself once a model was analysed (0.7.1, the owner).
    dark = document.documentElement.dataset.theme === "dark"; host?.classList.toggle("dark", dark);
    if (renderer) renderer.setClearColor(new THREE.Color(css("--bg2")));
    ground(); floorOn = false;
    const g = o.geom; if (!g || !g.segs.length) { for (const gg of lastGeo.values()) gg.dispose(); draw(); return; }
    const s = sel();
    const selEl = s?.kind === "element" ? s.id : null, selNode = s?.kind === "node" ? s.id : null;
    const res = results() && !stale() ? results() : null;
    const c = res ? curCase() : null;
    // A natural mode, when one is chosen, replaces the case's shape (and is always drawn moving).
    const md = res && modeNo() !== null ? res.modes?.[modeNo()!] ?? null : null;
    modeShown = !!md;
    const dsrc = md ? md.shape : c?.disp ?? null;
    // Colour by (0.6.4): a result needs an analysis that is up to date; a property needs none.
    const cb = colourBy();
    const ratios = res && show().colour && !md && cb === "ratio" ? segRatios(c, g.segs.length) : null;
    const moves = res && c && !md && cb === "move" ? g.segs.map(sg => Math.max(Math.hypot(...(c.disp[sg.a] ?? [0, 0, 0]).slice(0, 3) as [number, number, number]), Math.hypot(...(c.disp[sg.b] ?? [0, 0, 0]).slice(0, 3) as [number, number, number]))) : null;
    const mmax = moves ? Math.max(1e-12, ...moves) : 1;
    const cat = categories(m, g, cb);
    // Deformed shape: the case's displacements (or the mode's), scaled so the largest shows as 4 %
    // of the model (8 % for a mode, whose size has no meaning).
    const ext = extent(g);
    const dmax = dsrc ? Math.max(1e-12, ...dsrc.map(d => Math.hypot(d[0], d[1], d[2]))) : 1;
    const auto = (md ? 0.08 : 0.04) * ext / dmax;
    ampl = !dsrc || (!show().deformed && !md) ? 0 : md ? auto : show().scale > 0 ? show().scale : opts().deform > 0 ? opts().deform : auto;
    const pipeOf = (id: number) => m.pipes.find(p => p.id === id);
    // Brushed steel; colours (results, properties) less metallic so they read true. Every solid
    // material is outlined (`userData.ol`), not the translucent insulation.
    const ol = <M extends THREE.Material>(m: M) => { m.userData.ol = true; return m; };
    const steel = ol(new THREE.MeshStandardMaterial({ color: 0xa3adb8, metalness: 0.6, roughness: 0.36 }));
    const selMat = ol(new THREE.MeshStandardMaterial({ color: new THREE.Color(css("--blue")), metalness: 0.2, roughness: 0.45, emissive: new THREE.Color(css("--blue")), emissiveIntensity: 0.25 }));
    const insMat = new THREE.MeshStandardMaterial({ color: 0xdfe5ea, metalness: 0.4, roughness: 0.35, transparent: true, opacity: 0.3, depthWrite: false });
    const darkMat = ol(new THREE.MeshStandardMaterial({ color: 0x56616d, metalness: 0.55, roughness: 0.38 }));
    const supMat = ol(new THREE.MeshStandardMaterial({ color: new THREE.Color(css("--green")), metalness: 0.15, roughness: 0.5 }));
    const nozMat = ol(new THREE.MeshStandardMaterial({ color: 0xb7791f, metalness: 0.3, roughness: 0.5 }));
    const colourMats = new Map<string, THREE.Material>();
    // Buried pipe is drawn earth-coloured (when the stresses are not shown).
    const soilMat = ol(new THREE.MeshStandardMaterial({ color: 0x9c7a4f, roughness: 0.8 }));
    const buried = new Set((model()?.elements ?? []).filter(e => e.soil).map(e => e.id));
    const matFor = (si: number, sg: Seg) => {
      if (sg.element === selEl) return selMat;
      const col = ratios ? ratioColour(ratios[si]) : moves ? ratioColour(moves[si] / mmax) : cat ? cat.colourOf(si) : null;
      if (col) { if (!colourMats.has(col)) colourMats.set(col, ol(new THREE.MeshStandardMaterial({ color: col, metalness: 0.1, roughness: 0.5 }))); return colourMats.get(col)!; }
      return sg.rigid ? darkMat : buried.has(sg.element) ? soilMat : steel;
    };
    const dispAt = (sg: Seg, f: number): Vec => {
      if (!dsrc) return [0, 0, 0];
      const da = dsrc[sg.a], db = dsrc[sg.b];
      return [0, 1, 2].map(i => da[i] + (db[i] - da[i]) * f) as Vec;
    };
    // Detail by size: a large model is seen from far away, so fewer facets cost nothing visible.
    const big = g.segs.length > 600, rad = big ? 10 : 24, arcSteps = big ? 6 : 16;
    const rings: number[] = [], nodePts = new Set(Object.values(g.node_point));
    // A very large model is lit by the sky and key lights alone: the room's reflections cost the
    // most to draw and show least from far off.
    scene.environment = big ? null : roomLight; sky.intensity = big ? 1.5 : 0.55;
    g.segs.forEach((sg, si) => {
      const p = pipeOf(sg.pipe), pb = pipeOf(sg.pipe_b) ?? p; if (!p) return;
      // Centre line: a thin rod in place of the pipe (0.6.4).
      const thin = drawMode() === "centre" ? Math.max(ext * 0.0015, 0.004) : 0;
      const r0 = thin || p.od / 2, r1 = thin || (pb?.od ?? p.od) / 2;
      const curve = new ArcCurve(f => segAt(g, sg, f)[0]);
      const steps = sg.shape.kind === "arc" ? arcSteps : 1;
      let geo: THREE.BufferGeometry;
      curve.arcLengthDivisions = steps * 4;
      // Straight pipe (and a reducer's cone) as a cylinder along the segment, which is far cheaper to
      // build than a tube along a curve; arcs as a tube.
      const straight = (ra: number, rb: number) => {
        const a = g.points[sg.a].p as Vec, b = g.points[sg.b].p as Vec, L = len(sub(b, a));
        const c = new THREE.CylinderGeometry(rb, ra, L, rad, 1, true);
        c.rotateX(Math.PI / 2); c.lookAt(V(sub(b, a))); c.translate(...(addv(a, mul(sub(b, a), 0.5)) as Vec));
        return c;
      };
      const pa = g.points[sg.a].p, pb2 = g.points[sg.b].p, shp = sg.shape.kind === "arc" ? JSON.stringify(sg.shape) : "";
      const keep = (k: string, make: () => THREE.BufferGeometry) => {
        if (ampl > 0) return make();
        const key = `${k}|${pa}|${pb2}|${shp}|${rad}|${steps}`;
        let gg = geoCache.get(key) ?? lastGeo.get(key);
        if (!gg) { gg = make(); gg.userData.kept = true; made++; }
        geoCache.set(key, gg); return gg;
      };
      geo = keep(`${r0},${r1}`, () => sg.shape.kind === "arc" ? new THREE.TubeGeometry(curve, steps, r0, rad, false) : straight(r0, r1));
      const mesh = new THREE.Mesh(geo, matFor(si, sg));
      mesh.userData = { element: sg.element };
      group.add(mesh); pickables.push(mesh);
      if (ampl > 0) addDeform(geo, sg, dispAt);
      if (p.ins_t > 0 && !thin) { const ro = r0 + p.ins_t + p.clad_t, ig = keep(`i${ro}`, () => sg.shape.kind === "arc" ? new THREE.TubeGeometry(curve, steps, ro, rad, false) : straight(ro, ro)); group.add(new THREE.Mesh(ig, insMat)); if (ampl > 0) addDeform(ig, sg, dispAt); }
      // The edge where one piece meets the next (a weld, a bend's tangent line), as drawings show it.
      if (!thin && !big && ampl === 0) for (const [pt, rr, dirF] of [[sg.a, r0, 0], [sg.b, r1, 1]] as [number, number, number][]) {
        // Only at the model's nodes and a bend's two ends, not at every point the pipe is split at.
        if (sg.shape.kind !== "arc" && !nodePts.has(pt)) continue;
        const [c0, t0] = segAt(g, sg, dirF), u = unitv(Math.abs(t0[1]) < 0.9 ? cross(t0, [0, 1, 0]) : cross(t0, [1, 0, 0])), w = cross(t0, u);
        for (let k = 0; k < rad; k++) for (const kk of [k, k + 1]) { const th = 2 * Math.PI * kk / rad; rings.push(...addv(c0, addv(mul(u, rr * Math.cos(th)), mul(w, rr * Math.sin(th))))); }
      }
      if (sg.rigid) rigidSymbol(sg, r0);
      if (sg.bellows) bellowsSymbol(sg, r0);
    });
    // Tees: a sphere where the branch joins, so the junction reads solid.
    for (const t of g.tees) { const p = pipeOf(t.pipe); if (!p) continue; const sp = new THREE.Mesh(new THREE.SphereGeometry(p.od / 2 * 1.08, 20, 14), steel); sp.position.copy(V(g.points[t.point].p as Vec)); group.add(sp); }
    // Nodes: small spheres to click, and their numbers.
    const od0 = m.pipes[0]?.od ?? 0.17;
    const nodeR = Math.max(od0 * 0.32, ext * 0.004);
    for (const [num, pi] of Object.entries(g.node_point)) {
      const n = +num, pt = g.points[pi].p as Vec;
      const br = n === selNode ? nodeR * 1.6 : nodeR, bk = `ball|${br}|${big}`;
      let bg = geoCache.get(bk) ?? lastGeo.get(bk);
      if (!bg) { bg = new THREE.SphereGeometry(br, big ? 8 : 14, big ? 6 : 10); bg.userData.kept = true; }
      geoCache.set(bk, bg);
      const ball = new THREE.Mesh(bg, n === selNode ? selMat : darkMat);
      ball.position.copy(V(pt)); ball.userData = { node: n }; ball.renderOrder = 2; group.add(ball); pickables.push(ball);
      if (opts().labels && (Object.keys(g.node_point).length < 400 || n === selNode)) {
        const el = document.createElement("div"); el.className = "nlabel" + (n === selNode ? " on" : ""); el.textContent = String(n);
        el.dataset.x = String(pt[0]); el.dataset.y = String(pt[1]); el.dataset.z = String(pt[2]); labels.append(el);
      }
    }
    // Supports, nozzles and loads.
    const sz = Math.max(od0, ext * 0.012);
    const dirAt = (pi: number): Vec => { const sg = g.segs.find(x => x.a === pi || x.b === pi); return sg ? segAt(g, sg, sg.a === pi ? 0 : 1)[1] : [1, 0, 0]; };
    for (const sp of m.supports) {
      const pi = g.node_point[String(sp.node)]; if (pi === undefined) continue;
      const pt = g.points[pi].p as Vec, t = dirAt(pi);
      const grp = supportSymbol(sp.kind, sz, t, sp.id === (s?.kind === "support" ? s.id : -1) ? selMat : supMat);
      grp.position.copy(V(pt)); grp.traverse(x => { x.userData = { support: sp.id }; }); group.add(grp); pickables.push(grp);
    }
    for (const z of m.nozzles) {
      const pi = g.node_point[String(z.node)]; if (pi === undefined) continue;
      const pt = g.points[pi].p as Vec;
      let ax = len(z.axis as Vec) > 0 ? unitv(z.axis as Vec) : null;
      if (!ax) { const sg = g.segs.find(x => x.a === pi || x.b === pi); ax = sg ? (sg.a === pi ? segAt(g, sg, 0)[1] : mul(segAt(g, sg, 1)[1], -1)) : [1, 0, 0]; }
      const stub = new THREE.Mesh(new THREE.CylinderGeometry(sz * 1.6, sz * 1.6, sz * 0.5, 24), nozMat);
      stub.position.copy(V(addv(pt, mul(ax, -sz * 0.25)))); stub.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), V(ax));
      stub.userData = { nozzle: z.id }; group.add(stub); pickables.push(stub);
      const eq = new THREE.Mesh(new THREE.BoxGeometry(sz * 5, sz * 5, sz * 5), new THREE.MeshStandardMaterial({ color: 0xb7791f, transparent: true, opacity: 0.18 }));
      eq.position.copy(V(addv(pt, mul(ax, -sz * 3)))); group.add(eq);
    }
    for (const l of m.loads) {
      const pi = g.node_point[String(l.node)]; if (pi === undefined) continue;
      const f = l.f as Vec; if (len(f) === 0) continue;
      const d = unitv(f), L = sz * 4;
      const arr = new THREE.ArrowHelper(V(d), V(sub(g.points[pi].p as Vec, mul(d, L + od0 / 2))), L, 0xc0392b, L * 0.3, L * 0.18);
      group.add(arr);
    }
    // Deformed shape: the undeformed centreline as a faint line.
    if (ampl > 0) {
      const pts: THREE.Vector3[] = [];
      g.segs.forEach(sg => { for (let k = 0; k < (sg.shape.kind === "arc" ? 8 : 1); k++) { const n = sg.shape.kind === "arc" ? 8 : 1; pts.push(V(segAt(g, sg, k / n)[0]), V(segAt(g, sg, (k + 1) / n)[0])); } });
      group.add(new THREE.LineSegments(new THREE.BufferGeometry().setFromPoints(pts), new THREE.LineDashedMaterial({ color: 0x8090a0, dashSize: ext * 0.01, gapSize: ext * 0.006 })).computeLineDistances());
      if (!show().anim && !modeShown) for (const [attr, base, dv] of deform) { const a = attr.array as Float32Array; for (let i = 0; i < a.length; i++) a[i] = base[i] + ampl * dv[i]; attr.needsUpdate = true; }
    }
    // A ground grid under the model.
    const lo = Math.min(...g.points.map(p => p.p[1])) - Math.max(od0 * 2, ext * 0.05);
    const size = Math.pow(10, Math.ceil(Math.log10(Math.max(ext * 1.6, 1))));
    floorY = lo; floorL = size; floorOn = true; floorKey = "";
    batch();
    // Outlines round every solid, and the edges where pieces meet (not on a very large model, seen
    // from far off, where they would double the drawing for little to see).
    const olMat = outlineMat(dark ? "#05080C" : "#3B4652");
    group.updateMatrixWorld(true);
    const solids: THREE.Mesh[] = [];
    group.traverse(x => { const mm = x as THREE.Mesh; if (mm.isMesh && !Array.isArray(mm.material) && mm.material.userData.ol) solids.push(mm); });
    if (!big) for (const mm of solids) { const o = new THREE.Mesh(mm.geometry, olMat); o.matrixAutoUpdate = false; o.matrix.copy(mm.matrixWorld); o.raycast = () => {}; group.add(o); }
    host.dataset.outlined = String(big ? 0 : solids.length);
    if (rings.length) {
      const rg = new THREE.BufferGeometry(); rg.setAttribute("position", new THREE.BufferAttribute(new Float32Array(rings), 3));
      group.add(new THREE.LineSegments(rg, new THREE.LineBasicMaterial({ color: new THREE.Color(dark ? "#05080C" : "#2C3540"), transparent: true, opacity: 0.55 })));
    }
    for (const [k, gg] of lastGeo) if (!geoCache.has(k)) gg.dispose();
    // For the window checks: how many builds, and how many tubes the last one had to make.
    host.dataset.builds = String(+(host.dataset.builds ?? 0) + 1); host.dataset.made = String(made);
    draw();
  };
  /** Thousands of small meshes are thousands of draw calls: merge every still mesh that shares a
   *  material into one, keeping which element, node, support or nozzle each triangle belongs to so
   *  clicks still find it. Meshes that move with the deformed shape stay as they are. */
  const batch = () => {
    group.updateMatrixWorld(true);
    const moving = new Set(deform.map(d => d[0]));
    const buckets = new Map<THREE.Material, THREE.Mesh[]>();
    group.traverse(x => {
      const mm = x as THREE.Mesh;
      if (!mm.isMesh || Array.isArray(mm.material) || (mm as any).isInstancedMesh) return;
      const geo = mm.geometry as THREE.BufferGeometry;
      if (moving.has(geo.getAttribute("position") as THREE.BufferAttribute) || !geo.index) return;
      const list = buckets.get(mm.material) ?? []; list.push(mm); buckets.set(mm.material, list);
    });
    for (const [mat, list] of buckets) {
      if (list.length < 2) continue;
      // One position, normal and index buffer for the bucket, each mesh moved into place as it is
      // copied (most pipe meshes are built in place already); `starts` maps triangles back.
      let nv = 0, ni = 0;
      for (const mm of list) { nv += mm.geometry.getAttribute("position").count; ni += mm.geometry.index!.count; }
      const pos = new Float32Array(nv * 3), nor = new Float32Array(nv * 3), idx = nv > 65535 ? new Uint32Array(ni) : new Uint16Array(ni);
      const starts: number[] = [], tags = list.map(mm => mm.userData);
      const v = new THREE.Vector3(), nm = new THREE.Matrix3();
      let ov = 0, oi = 0;
      for (const mm of list) {
        const gp = mm.geometry.getAttribute("position") as THREE.BufferAttribute, gn = mm.geometry.getAttribute("normal") as THREE.BufferAttribute, gi = mm.geometry.index!;
        const still = mm.matrixWorld.equals(IDENTITY);
        if (still) { pos.set(gp.array as Float32Array, ov * 3); if (gn) nor.set(gn.array as Float32Array, ov * 3); }
        else {
          nm.getNormalMatrix(mm.matrixWorld);
          for (let i = 0; i < gp.count; i++) {
            v.fromBufferAttribute(gp, i).applyMatrix4(mm.matrixWorld); pos[(ov + i) * 3] = v.x; pos[(ov + i) * 3 + 1] = v.y; pos[(ov + i) * 3 + 2] = v.z;
            if (gn) { v.fromBufferAttribute(gn, i).applyMatrix3(nm).normalize(); nor[(ov + i) * 3] = v.x; nor[(ov + i) * 3 + 1] = v.y; nor[(ov + i) * 3 + 2] = v.z; }
          }
        }
        starts.push(oi / 3);
        const ga = gi.array; for (let i = 0; i < gi.count; i++) idx[oi + i] = ga[i] + ov;
        ov += gp.count; oi += gi.count;
      }
      const merged = new THREE.BufferGeometry();
      merged.setAttribute("position", new THREE.BufferAttribute(pos, 3));
      merged.setAttribute("normal", new THREE.BufferAttribute(nor, 3));
      merged.setIndex(new THREE.BufferAttribute(idx, 1));
      merged.computeBoundingSphere();
      const big = new THREE.Mesh(merged, mat);
      big.renderOrder = Math.max(...list.map(mm => mm.renderOrder));
      big.userData = { pick: (face: number) => { let lo = 0, hi = starts.length - 1; while (lo < hi) { const mid = (lo + hi + 1) >> 1; if (starts[mid] <= face) lo = mid; else hi = mid - 1; } return tags[lo]; } };
      const picked = list.some(mm => pickables.includes(mm) || pickables.some(p => p !== mm && p.getObjectById(mm.id)));
      for (const mm of list) { mm.parent?.remove(mm); if (!mm.geometry.userData.kept) mm.geometry.dispose(); }
      group.add(big);
      pickables = pickables.filter(p => p.parent || p === group);
      if (picked) pickables.push(big);
    }
  };
  const addDeform = (geo: THREE.BufferGeometry, sg: Seg, dispAt: (s: Seg, f: number) => Vec) => {
    const attr = geo.getAttribute("position") as THREE.BufferAttribute;
    const a = attr.array as Float32Array, base = new Float32Array(a), dv = new Float32Array(a.length);
    const pa = V(out()!.geom!.points[sg.a].p as Vec), pb = V(out()!.geom!.points[sg.b].p as Vec);
    const ab = pb.clone().sub(pa), l2 = Math.max(ab.lengthSq(), 1e-18);
    for (let i = 0; i < a.length; i += 3) {
      // The vertex's place along the segment: projected on the chord (good for arcs up to 45°).
      const f = Math.min(1, Math.max(0, new THREE.Vector3(a[i], a[i + 1], a[i + 2]).sub(pa).dot(ab) / l2));
      const d = dispAt(sg, f); dv[i] = d[0]; dv[i + 1] = d[1]; dv[i + 2] = d[2];
    }
    deform.push([attr, base, dv]);
  };
  const rigidSymbol = (sg: Seg, r: number) => {
    const g = out()!.geom!, a = g.points[sg.a].p as Vec, b = g.points[sg.b].p as Vec, mid = addv(a, mul(sub(b, a), 0.5)), t = unitv(sub(b, a)), L = len(sub(b, a));
    const q = new THREE.Quaternion().setFromUnitVectors(new THREE.Vector3(0, 1, 0), V(t));
    const mat = new THREE.MeshStandardMaterial({ color: 0x6b7785, metalness: 0.3, roughness: 0.5 });
    const kind = sg.rigid![0];
    const put = (geo: THREE.BufferGeometry, at: Vec) => { const mm = new THREE.Mesh(geo, mat); mm.position.copy(V(at)); mm.quaternion.copy(q); mm.userData = { element: sg.element }; group.add(mm); pickables.push(mm); };
    if (kind === "valve") {
      put(new THREE.ConeGeometry(r * 1.7, L / 2, 20), addv(mid, mul(t, -L / 4)));
      const c2 = new THREE.ConeGeometry(r * 1.7, L / 2, 20); c2.rotateX(Math.PI); put(c2, addv(mid, mul(t, L / 4)));
      const up: Vec = Math.abs(t[1]) > 0.9 ? [1, 0, 0] : [0, 1, 0];
      const stem = new THREE.Mesh(new THREE.CylinderGeometry(r * 0.15, r * 0.15, r * 3, 8), mat); stem.position.copy(V(addv(mid, mul(up, r * 1.5)))); stem.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), V(up)); group.add(stem);
      const wheel = new THREE.Mesh(new THREE.TorusGeometry(r * 1.1, r * 0.12, 8, 24), mat); wheel.position.copy(V(addv(mid, mul(up, r * 3)))); wheel.quaternion.setFromUnitVectors(new THREE.Vector3(0, 0, 1), V(up)); group.add(wheel);
    } else if (kind === "flange" || kind === "slipon") {
      for (const f of [0.25, 0.75]) put(new THREE.CylinderGeometry(r * 1.75, r * 1.75, Math.min(L * 0.4, r * 0.5), 28), addv(a, mul(sub(b, a), f)));
    } else put(new THREE.BoxGeometry(r * 2.6, L * 0.9, r * 2.6), mid);
  };

  // An expansion joint: convolutions (rings) along it, with tie rods when it is tied.
  const bellowsSymbol = (sg: Seg, r: number) => {
    const g = out()!.geom!, a = g.points[sg.a].p as Vec, b = g.points[sg.b].p as Vec, t = unitv(sub(b, a));
    const q = new THREE.Quaternion().setFromUnitVectors(new THREE.Vector3(0, 0, 1), V(t));
    const mat = new THREE.MeshStandardMaterial({ color: 0x8a96a3, metalness: 0.4, roughness: 0.45 });
    const n = 7;
    for (let i = 0; i < n; i++) {
      const ring = new THREE.Mesh(new THREE.TorusGeometry(r * 1.25, r * 0.22, 8, 28), mat);
      ring.position.copy(V(addv(a, mul(sub(b, a), (i + 1) / (n + 1))))); ring.quaternion.copy(q); ring.userData = { element: sg.element }; group.add(ring); pickables.push(ring);
    }
    if (sg.bellows!.tied) {
      const side: Vec = Math.abs(t[1]) > 0.9 ? [1, 0, 0] : [0, 1, 0];
      for (const s of [1, -1]) { const rod = new THREE.Mesh(new THREE.CylinderGeometry(r * 0.08, r * 0.08, len(sub(b, a)), 6), mat); rod.position.copy(V(addv(addv(a, mul(sub(b, a), 0.5)), mul(side, s * r * 1.8)))); rod.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), V(t)); group.add(rod); }
    }
  };

  const fit = () => {
    const g = out()?.geom; if (!g || !g.points.length) { camera.position.set(8, 6, 8); camera.lookAt(0, 0, 0); controls?.target.set(0, 0, 0); controls?.update(); draw(); return; }
    const [lo, hi] = bounds(g);
    const c = new THREE.Vector3((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2);
    const r = Math.max(0.5, new THREE.Vector3(hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]).length() / 2);
    const dir = camera.position.clone().sub(controls?.target ?? new THREE.Vector3()).normalize();
    if (dir.lengthSq() < 0.5) dir.set(1, 0.8, 1).normalize();
    const fov = Math.min(camera.fov, 2 * THREE.MathUtils.radToDeg(Math.atan(Math.tan(THREE.MathUtils.degToRad(camera.fov / 2)) * freeAspect)));
    const d = r / Math.sin(THREE.MathUtils.degToRad(fov / 2)) * 0.9;
    camera.position.copy(c.clone().add(dir.multiplyScalar(d)));
    camera.near = d / 1000; camera.far = d * 100; camera.updateProjectionMatrix();
    controls?.target.copy(c); controls?.update(); draw();
  };
  const stdView = (v: string) => {
    const t = controls?.target ?? new THREE.Vector3();
    const d = Math.max(1, camera.position.distanceTo(t));
    const dirs: Record<string, [number, number, number]> = { iso: [1, 0.8, 1], plan: [0, 1, 1e-4], front: [0, 0, 1], side: [1, 0, 0] };
    const dv = new THREE.Vector3(...(dirs[v] ?? dirs.iso)).normalize().multiplyScalar(d);
    camera.up.set(0, 1, 0);
    if (v === "plan") camera.up.set(0, 0, -1);
    camera.position.copy(t.clone().add(dv)); camera.lookAt(t); controls?.update(); fit();
  };
  fitNow = fit;
  /** Look from a direction (the view cube), keeping the distance, then fit. */
  const lookFrom = (dv: Vec) => {
    const t = controls?.target ?? new THREE.Vector3(), d = Math.max(1, camera.position.distanceTo(t));
    const v = V(unitv(dv[1] !== 0 ? [dv[0], dv[1], dv[2] + 1e-4 * dv[1]] : dv)).multiplyScalar(d);
    camera.up.set(0, 1, 0); if (dv[1] !== 0) camera.up.set(0, 0, -dv[1]);
    camera.position.copy(t.clone().add(v)); camera.lookAt(t); controls?.update(); fit();
  };
  /** The Inspector card covers the view's right side (0.7.1: Fit put the model's far end under it):
   *  the canvas stops at the card, so the picture is centred, and Fit fills, the part left free. (A
   *  camera view offset did the same but WebKit drew the grid's lines as wide bands with it.) */
  let freeAspect = 1;
  const resize = () => {
    const w = host.clientWidth, h = host.clientHeight; if (!w || !h || !renderer) return;
    const card = (cardOpen() || sel()) ? host.parentElement?.querySelector(".ipop") : null;
    const fw = Math.round(w - (card ? Math.min(w * 0.5, card.getBoundingClientRect().width + 8) : 0));
    renderer.setSize(fw, h, false);
    renderer.domElement.style.width = `${fw}px`;
    freeAspect = camera.aspect = fw / h;
    px.value.set(2 * 1.0 / fw, 2 * 1.0 / h);
    if (cubeEl) cubeEl.parentElement!.style.left = `${fw - 100}px`;
    camera.updateProjectionMatrix(); draw();
  };

  onMount(() => {
    try { renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true }); }
    catch { failed = true; host.querySelector(".nogl")?.classList.add("on"); return; }
    renderer.setPixelRatio(Math.min(2, devicePixelRatio));
    renderer.toneMapping = THREE.NeutralToneMapping;
    { const pm = new THREE.PMREMGenerator(renderer); roomLight = pm.fromScene(new RoomEnvironment(), 0.04, 0.1, 100, { size: 64 }).texture; scene.environment = roomLight; scene.environmentIntensity = 0.9; pm.dispose(); }
    renderer.setClearColor(new THREE.Color(css("--bg2")));
    host.prepend(renderer.domElement);
    controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = false;
    controls.mouseButtons = { LEFT: THREE.MOUSE.ROTATE, MIDDLE: THREE.MOUSE.PAN, RIGHT: THREE.MOUSE.PAN };
    controls.addEventListener("change", draw);
    camera.position.set(8, 6, 8);
    const ro = new ResizeObserver(resize); ro.observe(host); resize();
    // The graphics card reset: WebGL comes back by itself (draw everything again) or, if it has not
    // within 3 s, the view is made again from nothing.
    let lostT = 0;
    renderer.domElement.addEventListener("webglcontextlost", e => { e.preventDefault(); clearTimeout(lostT); lostT = window.setTimeout(() => { if (renderer?.getContext().isContextLost()) { note("The graphics card restarted: the 3D view was made again.", "note"); setGlGen(g => g + 1); } }, 3000); });
    renderer.domElement.addEventListener("webglcontextrestored", () => { clearTimeout(lostT); build(); });
    // Click (not drag) to select.
    let down: [number, number] | null = null;
    renderer.domElement.addEventListener("pointerdown", e => { down = [e.clientX, e.clientY]; });
    // What is under the pointer: a node within 14 px wins (nodes are small targets), else what the
    // ray hits first.
    const pickAt = (e: MouseEvent): Record<string, number> => {
      const r = renderer!.domElement.getBoundingClientRect();
      const m = model();
      let best: [number, number] | null = null;
      for (const n of m?.nodes ?? []) {
        const v = V([n.x, n.y, n.z]).project(camera);
        if (v.z > 1) continue;
        const d = Math.hypot((v.x + 1) / 2 * r.width - (e.clientX - r.left), (1 - v.y) / 2 * r.height - (e.clientY - r.top));
        if (d < 14 && (!best || d < best[1])) best = [n.num, d];
      }
      if (best) return { node: best[0] };
      const ray = new THREE.Raycaster(); ray.setFromCamera(new THREE.Vector2((e.clientX - r.left) / r.width * 2 - 1, -(e.clientY - r.top) / r.height * 2 + 1), camera);
      const hit = ray.intersectObjects(pickables, true)[0];
      const u = hit?.object.userData ?? {};
      return u.pick && hit?.faceIndex !== undefined && hit.faceIndex !== null ? u.pick(hit.faceIndex) : u;
    };
    // Right-click (not a right-drag, which pans): select what is under the pointer and offer what
    // can be done to it.
    renderer.domElement.addEventListener("contextmenu", e => {
      e.preventDefault();
      if (down && Math.hypot(e.clientX - down[0], e.clientY - down[1]) > 4) return;
      const u = pickAt(e);
      const s = u.node !== undefined ? { kind: "node" as const, id: u.node } : u.support !== undefined ? { kind: "support" as const, id: u.support } : u.nozzle !== undefined ? { kind: "nozzle" as const, id: u.nozzle } : u.element !== undefined ? { kind: "element" as const, id: u.element } : null;
      setSel(s);
      openMenu(e, contextItems(s));
    });
    renderer.domElement.addEventListener("pointerup", e => {
      if (!down || Math.hypot(e.clientX - down[0], e.clientY - down[1]) > 4 || e.button !== 0) return;
      const u = pickAt(e);
      if (u.node !== undefined) setSel({ kind: "node", id: u.node });
      else if (u.support !== undefined) setSel({ kind: "support", id: u.support });
      else if (u.nozzle !== undefined) setSel({ kind: "nozzle", id: u.nozzle });
      else if (u.element !== undefined) setSel({ kind: "element", id: u.element });
      else setSel(null);
    });
    setFit(fit); setStdView(stdView);
    build(); setTimeout(fit, 30);
    const tick = (t: number) => { anim = requestAnimationFrame(tick); if (((show().anim && show().deformed) || modeShown) && deform.length) { animT = t * speed() / 600; draw(); } };
    anim = requestAnimationFrame(tick);
    // Leaving the view frees the GPU: the scene's geometry, and the WebGL context itself (dispose()
    // alone keeps the context; switching views many times ran the GPU out of contexts, found 0.3.7).
    onCleanup(() => {
      cancelAnimationFrame(anim); cancelAnimationFrame(pendingBuild); ro.disconnect(); controls?.dispose();
      scene.traverse(x => { const mm = x as THREE.Mesh; mm.geometry?.dispose?.(); }); for (const gg of geoCache.values()) gg.dispose(); geoCache.clear();
      renderer?.dispose(); renderer?.forceContextLoss(); renderer = null;
    });
  });
  createEffect(on([() => out(), sel, results, stale, show, colourBy, drawMode, curCase, () => opts().labels, () => opts().contours, () => opts().deform, themeTick], () => { if (!failed) later(); }, { defer: true }));
  createEffect(on([cardOpen, () => !!sel()], () => requestAnimationFrame(resize), { defer: true }));
  // The model's first elements: fit as it grows from nothing.
  createEffect(on(() => out()?.geom?.points.length ?? 0, (n, prev) => { if (renderer && (prev ?? 0) < 2 && n >= 2) setTimeout(fit, 20); }, { defer: true }));

  return (
    <div class="v3d" ref={host}>
      <div class="labels" ref={labels} />
      <div class="nogl"><b>The 3D view needs WebGL</b><span>This computer's graphics did not start it. The element table, analysis, results and isometric all work without it.</span></div>
      <Show when={!model()?.elements.length}><div class="empty"><b>Route the piping</b><span>Type the first element in the table below: its length in DX (east), DY (up) or DZ (south). It draws here at once, at its true diameter.</span></div></Show>
      <Show when={out()?.geom_error}><div class="geoerr">{out()!.geom_error}</div></Show>
      <Show when={out()?.geom?.warnings.length}><div class="geowarn">{out()!.geom!.warnings.join(" ")}</div></Show>
      <Show when={results() && !stale() && (show().colour && colourBy() === "ratio" || modeNo() !== null)}><Legend /></Show>
      <Show when={results() && !stale() && colourBy() === "move" && modeNo() === null}><MoveLegend /></Show>
      <Show when={results() && !stale()}><Player /></Show>
      <Show when={categories(model(), out()?.geom, colourBy())}>{c => <div class="legend3d"><b>{CB.find(x => x[0] === colourBy())?.[1]}</b><For each={c().items}>{([k, col]) => <span class="catrow"><i style={{ background: col }} />{k}</span>}</For></div>}</Show>
      <Show when={results() && stale()}><div class="stalebar3d">The model changed since the analysis: press F5 to analyze again.</div></Show>
      <div class="vcube"><svg class="cube" ref={cubeEl} viewBox="0 0 64 64" width="76" height="76">{FACES.map(([n, d]) => <g class={`cf cf-${n.toLowerCase()}`} data-face={n} onClick={() => lookFrom(d)}><title>{`Look from the ${n.toLowerCase()}`}</title><polygon /><text text-anchor="middle">{n}</text></g>)}</svg></div>
      <div class="axes3d"><span style={{ color: "#D64545" }}>X east</span><span style={{ color: "#3BB273" }}>Y up</span><span style={{ color: "#2B6FD6" }}>Z south</span></div>
    </div>
  );
};

/** What the pipe can be coloured by (0.6.4). */
export const CB: [ColourBy, string][] = [["ratio", "Stress ratio"], ["move", "Movement"], ["size", "Pipe size"], ["material", "Material"], ["cond", "Line conditions"], ["temp", "Temperature (case 1)"], ["soil", "Buried or not"], ["plain", "Plain"]];

/** The bar above the 3D view (0.6.4, after Datum Mesh): the case, colour by, solid or centre line,
 *  the standard views and fit. */
export const CtxBar = () => (
  <div class="ctxbar">
    <label>Case</label>
    <select class="casepick" disabled={!results()} onChange={e => setCaseId(+e.currentTarget.value)} title={results() ? "The load case the view shows" : "Analyze (F5) to choose a case"}>
      <Show when={results()} fallback={<option>—</option>}><For each={results()!.cases}>{c => <option value={c.id} selected={c.id === curCase()?.id}>{c.name}</option>}</For></Show>
    </select>
    <span class="sep" />
    <label>Colour by</label>
    <select class="cbpick" onChange={e => setColourBy(e.currentTarget.value as ColourBy)}><For each={CB}>{([k, l]) => <option value={k} selected={k === colourBy()}>{l}</option>}</For></select>
    <span class="sep" />
    <span class="seg"><button classList={{ on: drawMode() === "solid" }} onClick={() => setDrawMode("solid")}>Solid</button><button classList={{ on: drawMode() === "centre" }} onClick={() => setDrawMode("centre")}>Centre line</button></span>
    <span class="grow" />
    <span class="seg vbar"><For each={[["iso", "Iso"], ["plan", "Plan"], ["front", "Front"], ["side", "Side"]]}>{([k, l]) => <button title={`${l} view`} onClick={() => C.std(k)}>{l}</button>}</For><button title="Fit the model (F)" onClick={() => fitNow()}>Fit</button></span>
  </div>
);

/** The player (0.6.6, after Datum Mesh): play and pause the deformed shape (or the mode shown),
 *  back to the still shape, and the speed. It loops. */
const Player = () => {
  const playing = () => show().anim && (show().deformed || modeNo() !== null);
  return <div class="player3d">
    <button title="Back to the still shape" onClick={() => { setShowPart({ anim: false }); setModeNo(null); }}>⏮</button>
    <button class="pp" title={playing() ? "Pause" : "Play the deformed shape"} onClick={() => playing() ? setShowPart({ anim: false }) : setShowPart({ anim: true, deformed: true })}>{playing() ? "❚❚" : "▶"}</button>
    <span class="what">{modeNo() !== null ? `Mode ${results()?.modes?.[modeNo()!]?.n ?? ""}` : `Deformed · ${curCase()?.name ?? ""}`} · loop</span>
    <span class="seg"><For each={[0.5, 1, 2]}>{v => <button classList={{ on: speed() === v }} onClick={() => setSpeed(v)}>{v}×</button>}</For></span>
  </div>;
};

/** The movement's colours: none to the largest of the case. */
const MoveLegend = () => {
  const c = () => curCase();
  const top = () => { const x = c(); return x ? Math.max(0, ...x.disp.map(d => Math.hypot(d[0], d[1], d[2]))) : 0; };
  return <div class="legend3d"><b>Movement · {c()?.name}</b>
    <div class="lbar"><For each={[0, 0.2, 0.4, 0.6, 0.8]}>{v => <i style={{ background: ratioColour(v + 0.1) }}><small>{fmt("disp", top() * v)}</small></i>}</For><i style={{ background: ratioColour(1) }}><small>{fmt("disp", top())}</small></i></div>
    <span>Largest {fmt("disp", top())}</span></div>;
};

const Legend = () => {
  const c = () => curCase();
  const md = () => { const k = modeNo(); return k === null ? null : results()?.modes?.[k] ?? null; };
  const worst = () => { const x = c(); if (!x || x.worst === null) return null; return x.stresses[x.worst]; };
  return <div class="legend3d">
    <Show when={md()}>{m => <b>Mode {m().n} · {num(m().freq, 4)} Hz <button class="btn sm" onClick={() => setModeNo(null)}>Stop</button></b>}</Show>
    <Show when={!md()}>
    <b>{c()?.stresses.length ? `Stress ratio · ${c()!.name}` : "Highest stress ratio of any case"}</b>
    <div class="lbar"><For each={[0, 0.2, 0.4, 0.6, 0.8]}>{v => <i style={{ background: ratioColour(v + 0.1) }}><small>{num(v, 2)}</small></i>}</For><i style={{ background: ratioColour(1.5) }}><small>1.0</small></i><em>over 1</em></div>
    <Show when={worst()}>{w => <span class="mono">Worst {num(w().ratio, 3)} at {results()!.geom.points[w().point]?.label}: {fmt("stress", w().stress)} / {fmt("stress", w().allow)}</span>}</Show>
    </Show>
    <Show when={show().deformed && c() && !md()}><span class="muted">Deformed × <input type="range" min="0" max="3" step="0.05" value={show().scale ? Math.log10(show().scale) / 3 + 0.5 : 0.5} onInput={e => setShowPart({ scale: Math.pow(10, (+e.currentTarget.value - 0.5) * 3) })} /> largest move {fmt("disp", Math.max(...c()!.disp.map(d => Math.hypot(d[0], d[1], d[2]))))}</span></Show>
  </div>;
};

export const bounds = (g: Geom): [V3, V3] => {
  const lo: V3 = [Infinity, Infinity, Infinity], hi: V3 = [-Infinity, -Infinity, -Infinity];
  for (const p of g.points) for (let i = 0; i < 3; i++) { lo[i] = Math.min(lo[i], p.p[i]); hi[i] = Math.max(hi[i], p.p[i]); }
  return [lo, hi];
};
const extent = (g: Geom) => { const [lo, hi] = bounds(g); return Math.max(0.5, Math.hypot(hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2])); };

/** A support's symbol, at the origin, sized s, for a pipe along t. */
const supportSymbol = (kind: Model["supports"][number]["kind"], s: number, t: Vec, mat: THREE.Material): THREE.Group => {
  const g = new THREE.Group();
  const add = (geo: THREE.BufferGeometry, at: Vec, dir?: Vec) => { const m = new THREE.Mesh(geo, mat); m.position.copy(V(at)); if (dir) m.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), V(unitv(dir))); g.add(m); return m; };
  const lat = unitv(len(cross([0, 1, 0], t)) > 0.1 ? cross([0, 1, 0], t) : [1, 0, 0]);
  switch (kind.kind) {
    case "anchor": add(new THREE.BoxGeometry(s * 2.2, s * 2.2, s * 2.2), [0, 0, 0]); break;
    case "rest": add(new THREE.ConeGeometry(s * 0.7, s * 1.4, 4), [0, -s * 1.3, 0]); add(new THREE.BoxGeometry(s * 2.2, s * 0.15, s * 2.2), [0, -s * 2.05, 0]); break;
    case "guide": for (const k of [-1, 1]) add(new THREE.BoxGeometry(s * 0.15, s * 1.6, s * 1.6), mul(lat, k * s * 0.85), [0, 1, 0]); add(new THREE.BoxGeometry(s * 2.2, s * 0.15, s * 2.2), [0, -s * 1.1, 0]); break;
    case "linestop": for (const k of [-1, 1]) add(new THREE.CylinderGeometry(s * 1.1, s * 1.1, s * 0.15, 20), mul(t, k * s * 0.6), t); break;
    case "restraint": { const d = unitv(kind.dir as Vec); add(new THREE.CylinderGeometry(s * 0.12, s * 0.12, s * 2.6, 8), [0, 0, 0], d); for (const k of [-1, 1]) add(new THREE.ConeGeometry(s * 0.35, s * 0.6, 12), mul(d, k * s * 1.5), mul(d, k)); break; }
    case "hanger":
    case "spring": {
      const d = unitv((kind.kind === "hanger" ? [0, 1, 0] : kind.dir) as Vec), pts: THREE.Vector3[] = [];
      for (let i = 0; i <= 24; i++) { const a = i / 24 * Math.PI * 8; const off = addv(mul(d, -s * (0.8 + 1.6 * i / 24)), addv(mul(lat, Math.cos(a) * s * 0.4), mul(unitv(cross(d, lat)), Math.sin(a) * s * 0.4))); pts.push(V(off)); }
      const ln = new THREE.Line(new THREE.BufferGeometry().setFromPoints(pts), new THREE.LineBasicMaterial({ color: (mat as THREE.MeshStandardMaterial).color })); g.add(ln);
      add(new THREE.BoxGeometry(s * 1.6, s * 0.15, s * 1.6), mul(d, -s * 2.5)); break;
    }
    case "rotspring": add(new THREE.TorusGeometry(s * 1.1, s * 0.1, 8, 28), [0, 0, 0], cross(unitv(kind.dir as Vec), [0, 0, 1]).some(Math.abs) ? kind.dir as Vec : [0, 1, 0]); break;
    case "constant": add(new THREE.CylinderGeometry(s * 0.08, s * 0.08, s * 2, 6), [0, s * 1.2, 0]); add(new THREE.BoxGeometry(s * 1.4, s * 1.0, s * 1.0), [0, s * 2.6, 0]); break;
  }
  return g;
};

/** What the 3D view's right-click menu offers for what is under the pointer. */
const contextItems = (s: { kind: "node" | "support" | "nozzle" | "element"; id: number } | null): Item[] => {
  const m = model();
  if (!s || !m) return [{ label: "Fit the model", run: () => C.fit(), keys: "F" }, { label: "Analyze", run: () => C.analyze(), keys: "F5" }];
  if (s.kind === "node") return [
    { label: "Add a support", sub: SUPPORTS.map(([k, l]) => ({ label: l, run: () => C.support(k) })) },
    { label: "Nozzle point", run: () => C.nozzle() },
    { label: "Force or moment", run: () => C.load() },
  ];
  if (s.kind === "element") {
    const e = m.elements.find(x => x.id === s.id);
    return [
      { label: "Component", sub: COMPS.map(([k, l]) => ({ label: l, on: !!e && compKey(e.comp) === k, run: () => { if (e) act({ Set: { item: { Element: { ...e, comp: compFrom(k, e, m) } } } }); } })) },
      { label: "Insert a node (split)", run: () => C.split() },
      { label: "Add an element after it", run: () => C.addAfter(), keys: "Ctrl+Enter" },
      { label: "Expansion loop…", run: () => import("./tools").then(t => t.loopWizard()) },
      "-",
      { label: "Delete", run: () => C.del(), keys: "Del" },
    ];
  }
  return [{ label: "Delete", run: () => C.del(), keys: "Del" }];
};
