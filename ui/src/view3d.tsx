// The 3D view (three.js, MIT): the piping at its true diameter from the engine's analysis geometry
// (bends as arcs), insulation as a translucent sleeve, valves, flanges, reducers, tees, support and
// nozzle symbols, node numbers; orbit, pan, zoom, fit and the standard views; click to select (the
// table follows). After an analysis: colours by stress ratio with a legend, and the deformed shape
// with a scale, animated between installed and operating. The window draws what the engine gives:
// positions, arcs and results all come from it.
import { onMount, onCleanup, createEffect, on, Show, For } from "solid-js";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { Geom, Model, Seg, V3, num } from "./api";
import { out, model, sel, setSel, opts, themeTick } from "./store";
import { C, setFit, setStdView } from "./cmds";
import { results, stale, show, curCase, segRatios, setShowPart } from "./rstate";
import { fmt } from "./units";

type Vec = [number, number, number];
const sub = (a: Vec, b: Vec): Vec => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const addv = (a: Vec, b: Vec): Vec => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
const mul = (a: Vec, s: number): Vec => [a[0] * s, a[1] * s, a[2] * s];
const cross = (a: Vec, b: Vec): Vec => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const len = (a: Vec) => Math.hypot(a[0], a[1], a[2]);
const unitv = (a: Vec): Vec => { const n = len(a); return n > 0 ? mul(a, 1 / n) : [0, 0, 0]; };
const V = (a: Vec) => new THREE.Vector3(a[0], a[1], a[2]);

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

export const View3D = () => {
  let host!: HTMLDivElement, labels!: HTMLDivElement;
  let renderer: THREE.WebGLRenderer | null = null;
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(35, 1, 0.01, 1e5);
  let controls: OrbitControls | null = null;
  let group = new THREE.Group();
  let pickables: THREE.Object3D[] = [];
  let anim = 0, animT = 0;
  /** Vertices to move for the deformed shape: [attribute, base positions, displacement per vertex]. */
  let deform: [THREE.BufferAttribute, Float32Array, Float32Array][] = [];
  let ampl = 1;
  let failed = false;
  scene.add(new THREE.HemisphereLight(0xffffff, 0x8899aa, 1.6));
  const sun = new THREE.DirectionalLight(0xffffff, 1.4);
  sun.position.set(3, 5, 4);
  scene.add(sun);
  scene.add(camera);

  const draw = () => {
    if (!renderer) return;
    if (deform.length && show().anim) {
      const f = 0.5 - 0.5 * Math.cos(animT);
      for (const [attr, base, dv] of deform) { const a = attr.array as Float32Array; for (let i = 0; i < a.length; i++) a[i] = base[i] + f * ampl * dv[i]; attr.needsUpdate = true; }
    }
    renderer.render(scene, camera);
    placeLabels();
  };
  const placeLabels = () => {
    const w = host.clientWidth, h = host.clientHeight;
    for (const el of [...labels.children] as HTMLElement[]) {
      const p = new THREE.Vector3(+el.dataset.x!, +el.dataset.y!, +el.dataset.z!).project(camera);
      const vis = p.z < 1 && p.z > -1;
      el.style.display = vis ? "" : "none";
      el.style.transform = `translate(${(p.x + 1) / 2 * w + 6}px, ${(1 - p.y) / 2 * h - 16}px)`;
    }
  };

  /** Builds the scene from the engine's geometry, the selection and the results. */
  const build = () => {
    const o = out(), m = model(); if (!o || !m) return;
    scene.remove(group); group.traverse(x => { const mm = x as THREE.Mesh; mm.geometry?.dispose?.(); }); group = new THREE.Group(); scene.add(group);
    pickables = []; deform = []; labels.innerHTML = "";
    const g = o.geom; if (!g || !g.segs.length) { draw(); return; }
    const s = sel();
    const selEl = s?.kind === "element" ? s.id : null, selNode = s?.kind === "node" ? s.id : null;
    const res = results() && !stale() ? results() : null;
    const c = res ? curCase() : null;
    const ratios = res && show().colour ? segRatios(c, g.segs.length) : null;
    // Deformed shape: the case's displacements, scaled so the largest shows as 4 % of the model.
    const ext = extent(g);
    const dmax = c ? Math.max(1e-12, ...c.disp.map(d => Math.hypot(d[0], d[1], d[2]))) : 1;
    const auto = 0.04 * ext / dmax;
    ampl = !c || !show().deformed ? 0 : show().scale > 0 ? show().scale : opts().deform > 0 ? opts().deform : auto;
    const pipeOf = (id: number) => m.pipes.find(p => p.id === id);
    const steel = new THREE.MeshStandardMaterial({ color: 0x9aa5b1, metalness: 0.25, roughness: 0.55 });
    const selMat = new THREE.MeshStandardMaterial({ color: new THREE.Color(css("--blue")), metalness: 0.2, roughness: 0.5, emissive: new THREE.Color(css("--blue")), emissiveIntensity: 0.25 });
    const insMat = new THREE.MeshStandardMaterial({ color: 0xe8e0c0, transparent: true, opacity: 0.28, depthWrite: false });
    const darkMat = new THREE.MeshStandardMaterial({ color: 0x55606e, metalness: 0.3, roughness: 0.5 });
    const supMat = new THREE.MeshStandardMaterial({ color: new THREE.Color(css("--green")), roughness: 0.6 });
    const nozMat = new THREE.MeshStandardMaterial({ color: 0xb7791f, roughness: 0.6 });
    const colourMats = new Map<string, THREE.Material>();
    const matFor = (si: number, sg: Seg) => {
      if (sg.element === selEl) return selMat;
      if (ratios) { const col = ratioColour(ratios[si]); if (!colourMats.has(col)) colourMats.set(col, new THREE.MeshStandardMaterial({ color: col, roughness: 0.6 })); return colourMats.get(col)!; }
      return sg.rigid ? darkMat : steel;
    };
    const dispAt = (sg: Seg, f: number): Vec => {
      if (!c) return [0, 0, 0];
      const da = c.disp[sg.a], db = c.disp[sg.b];
      return [0, 1, 2].map(i => da[i] + (db[i] - da[i]) * f) as Vec;
    };
    g.segs.forEach((sg, si) => {
      const p = pipeOf(sg.pipe), pb = pipeOf(sg.pipe_b) ?? p; if (!p) return;
      const r0 = p.od / 2, r1 = (pb?.od ?? p.od) / 2;
      const curve = new ArcCurve(f => segAt(g, sg, f)[0]);
      const steps = sg.shape.kind === "arc" ? 16 : 1;
      let geo: THREE.BufferGeometry;
      if (sg.shape.kind === "arc" || r0 === r1) geo = new THREE.TubeGeometry(curve, steps, r0, 20, false);
      else {
        // A reducer: a cone between the two diameters, along the segment.
        const a = g.points[sg.a].p as Vec, b = g.points[sg.b].p as Vec, L = len(sub(b, a));
        geo = new THREE.CylinderGeometry(r1, r0, L, 20, 1, true);
        geo.rotateX(Math.PI / 2); geo.lookAt(V(sub(b, a))); geo.translate(...(addv(a, mul(sub(b, a), 0.5)) as Vec));
      }
      const mesh = new THREE.Mesh(geo, matFor(si, sg));
      mesh.userData = { element: sg.element };
      group.add(mesh); pickables.push(mesh);
      if (ampl > 0) addDeform(geo, sg, dispAt);
      if (p.ins_t > 0) { const ig = new THREE.TubeGeometry(curve, steps, r0 + p.ins_t + p.clad_t, 20, false); group.add(new THREE.Mesh(ig, insMat)); if (ampl > 0) addDeform(ig, sg, dispAt); }
      if (sg.rigid) rigidSymbol(sg, r0);
    });
    // Tees: a sphere where the branch joins, so the junction reads solid.
    for (const t of g.tees) { const p = pipeOf(t.pipe); if (!p) continue; const sp = new THREE.Mesh(new THREE.SphereGeometry(p.od / 2 * 1.08, 20, 14), steel); sp.position.copy(V(g.points[t.point].p as Vec)); group.add(sp); }
    // Nodes: small spheres to click, and their numbers.
    const od0 = m.pipes[0]?.od ?? 0.17;
    const nodeR = Math.max(od0 * 0.32, ext * 0.004);
    for (const [num, pi] of Object.entries(g.node_point)) {
      const n = +num, pt = g.points[pi].p as Vec;
      const ball = new THREE.Mesh(new THREE.SphereGeometry(n === selNode ? nodeR * 1.6 : nodeR, 14, 10), n === selNode ? selMat : darkMat);
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
      if (!show().anim) for (const [attr, base, dv] of deform) { const a = attr.array as Float32Array; for (let i = 0; i < a.length; i++) a[i] = base[i] + ampl * dv[i]; attr.needsUpdate = true; }
    }
    // A ground grid under the model.
    const lo = Math.min(...g.points.map(p => p.p[1])) - Math.max(od0 * 2, ext * 0.05);
    const size = Math.pow(10, Math.ceil(Math.log10(Math.max(ext * 1.6, 1))));
    const grid = new THREE.GridHelper(size, Math.min(100, Math.round(size / Math.pow(10, Math.floor(Math.log10(size)) - 1))), new THREE.Color(css("--line3")), new THREE.Color(css("--line2")));
    const cx = (bounds(g)[0][0] + bounds(g)[1][0]) / 2, cz = (bounds(g)[0][2] + bounds(g)[1][2]) / 2;
    grid.position.set(Math.round(cx), lo, Math.round(cz)); group.add(grid);
    draw();
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

  const fit = () => {
    const g = out()?.geom; if (!g || !g.points.length) { camera.position.set(8, 6, 8); camera.lookAt(0, 0, 0); controls?.target.set(0, 0, 0); controls?.update(); draw(); return; }
    const [lo, hi] = bounds(g);
    const c = new THREE.Vector3((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2);
    const r = Math.max(0.5, new THREE.Vector3(hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]).length() / 2);
    const dir = camera.position.clone().sub(controls?.target ?? new THREE.Vector3()).normalize();
    if (dir.lengthSq() < 0.5) dir.set(1, 0.8, 1).normalize();
    const fov = Math.min(camera.fov, 2 * THREE.MathUtils.radToDeg(Math.atan(Math.tan(THREE.MathUtils.degToRad(camera.fov / 2)) * camera.aspect)));
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

  onMount(() => {
    try { renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true }); }
    catch { failed = true; host.querySelector(".nogl")?.classList.add("on"); return; }
    renderer.setPixelRatio(Math.min(2, devicePixelRatio));
    host.prepend(renderer.domElement);
    controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = false;
    controls.mouseButtons = { LEFT: THREE.MOUSE.ROTATE, MIDDLE: THREE.MOUSE.PAN, RIGHT: THREE.MOUSE.PAN };
    controls.addEventListener("change", draw);
    camera.position.set(8, 6, 8);
    const resize = () => { const w = host.clientWidth, h = host.clientHeight; if (!w || !h || !renderer) return; renderer.setSize(w, h, false); camera.aspect = w / h; camera.updateProjectionMatrix(); draw(); };
    const ro = new ResizeObserver(resize); ro.observe(host); resize();
    // Click (not drag) to select.
    let down: [number, number] | null = null;
    renderer.domElement.addEventListener("pointerdown", e => { down = [e.clientX, e.clientY]; });
    renderer.domElement.addEventListener("pointerup", e => {
      if (!down || Math.hypot(e.clientX - down[0], e.clientY - down[1]) > 4 || e.button !== 0) return;
      const r = renderer!.domElement.getBoundingClientRect();
      const ray = new THREE.Raycaster(); ray.setFromCamera(new THREE.Vector2((e.clientX - r.left) / r.width * 2 - 1, -(e.clientY - r.top) / r.height * 2 + 1), camera);
      const hit = ray.intersectObjects(pickables, true)[0];
      const u = hit?.object.userData ?? {};
      if (u.node !== undefined) setSel({ kind: "node", id: u.node });
      else if (u.support !== undefined) setSel({ kind: "support", id: u.support });
      else if (u.nozzle !== undefined) setSel({ kind: "nozzle", id: u.nozzle });
      else if (u.element !== undefined) setSel({ kind: "element", id: u.element });
      else setSel(null);
    });
    setFit(fit); setStdView(stdView);
    build(); setTimeout(fit, 30);
    const tick = (t: number) => { anim = requestAnimationFrame(tick); if (show().anim && show().deformed && deform.length) { animT = t / 600; draw(); } };
    anim = requestAnimationFrame(tick);
    onCleanup(() => { cancelAnimationFrame(anim); ro.disconnect(); controls?.dispose(); renderer?.dispose(); renderer = null; });
  });
  createEffect(on([() => out(), sel, results, stale, show, () => opts().labels, () => opts().contours, () => opts().deform, themeTick], () => { if (!failed) { if (renderer) renderer.setClearColor(new THREE.Color(css("--bg2"))); build(); } }, { defer: true }));
  // The model's first elements: fit as it grows from nothing.
  createEffect(on(() => out()?.geom?.points.length ?? 0, (n, prev) => { if (renderer && (prev ?? 0) < 2 && n >= 2) setTimeout(fit, 20); }, { defer: true }));

  return (
    <div class="v3d" ref={host}>
      <div class="labels" ref={labels} />
      <div class="nogl"><b>The 3D view needs WebGL</b><span>This computer's graphics did not start it. The element table, analysis, results and isometric all work without it.</span></div>
      <Show when={!model()?.elements.length}><div class="empty"><b>Route the piping</b><span>Type the first element in the table below: its length in DX (east), DY (up) or DZ (south). It draws here at once, at its true diameter.</span></div></Show>
      <Show when={out()?.geom_error}><div class="geoerr">{out()!.geom_error}</div></Show>
      <Show when={out()?.geom?.warnings.length}><div class="geowarn">{out()!.geom!.warnings.join(" ")}</div></Show>
      <ViewBar />
      <Show when={results() && !stale() && show().colour}><Legend /></Show>
      <Show when={results() && stale()}><div class="stalebar3d">The model changed since the analysis: press F5 to analyze again.</div></Show>
      <div class="axes3d"><span style={{ color: "#D64545" }}>X east</span><span style={{ color: "#3BB273" }}>Y up</span><span style={{ color: "#2B6FD6" }}>Z south</span></div>
    </div>
  );
};

/** View buttons over the 3D view. */
const ViewBar = () => (
  <div class="vbar">
    <For each={[["iso", "Iso"], ["plan", "Plan"], ["front", "Front"], ["side", "Side"]]}>{([k, l]) => <button title={`${l} view`} onClick={() => C.std(k)}>{l}</button>}</For>
    <button title="Fit the model (F)" onClick={() => fitNow()}>Fit</button>
  </div>
);

const Legend = () => {
  const c = () => curCase();
  const worst = () => { const x = c(); if (!x || x.worst === null) return null; return x.stresses[x.worst]; };
  return <div class="legend3d">
    <b>{c()?.stresses.length ? `Stress ratio · ${c()!.name}` : "Highest stress ratio of any case"}</b>
    <div class="lbar"><For each={[0, 0.2, 0.4, 0.6, 0.8]}>{v => <i style={{ background: ratioColour(v + 0.1) }}><small>{num(v, 2)}</small></i>}</For><i style={{ background: ratioColour(1.5) }}><small>1.0</small></i><em>over 1</em></div>
    <Show when={worst()}>{w => <span class="mono">Worst {num(w().ratio, 3)} at {results()!.geom.points[w().point]?.label}: {fmt("stress", w().stress)} / {fmt("stress", w().allow)}</span>}</Show>
    <Show when={show().deformed && c()}><span class="muted">Deformed × <input type="range" min="0" max="3" step="0.05" value={show().scale ? Math.log10(show().scale) / 3 + 0.5 : 0.5} onInput={e => setShowPart({ scale: Math.pow(10, (+e.currentTarget.value - 0.5) * 3) })} /> largest move {fmt("disp", Math.max(...c()!.disp.map(d => Math.hypot(d[0], d[1], d[2]))))}</span></Show>
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
