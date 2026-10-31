// Speed checks of the window on a large model (examples::big(160): about 1440 elements, 1600
// supports), with the engine's real model and results
// (cargo run -p datum-core --example dump > tests_ui/fixture.json) behind a mocked Tauri bridge
// that records every call. Run: npm run test:ui (it runs this after run.cjs).
const { chromium } = require("playwright");
const fs = require("fs"), http = require("http"), path = require("path");
const root = path.join(__dirname, "../ui/dist"), fix = JSON.parse(fs.readFileSync(path.join(__dirname, "fixture.json")));
const types = { ".js": "text/javascript", ".css": "text/css", ".svg": "image/svg+xml", ".woff2": "font/woff2", ".woff": "font/woff", ".html": "text/html" };
const srv = http.createServer((q, r) => {
  const f = path.join(root, q.url === "/" ? "index.html" : decodeURIComponent(q.url.split("?")[0]));
  fs.readFile(f, (e, b) => { if (e) { r.writeHead(404); return r.end(); } r.writeHead(200, { "content-type": types[path.extname(f)] ?? "application/octet-stream" }); r.end(b); });
});
let failed = 0;
const check = (name, ok, info = "") => { console.log(`${ok ? "ok  " : "FAIL"} ${name}${ok ? "" : "  " + info}`); if (!ok) failed++; };
const shots = process.env.SHOTS;

(async () => {
  await new Promise(ok => srv.listen(0, ok));
  const url = `http://localhost:${srv.address().port}/`;
  const b = await chromium.launch(process.env.CHROMIUM ? { executablePath: process.env.CHROMIUM } : {});
  const p = await b.newPage({ viewport: { width: 1440, height: 900 } });
  const errors = [];
  p.on("pageerror", e => errors.push(e.message));
  p.on("console", m => { if (m.type() === "error") errors.push(m.text()); });
  await p.addInitScript(f => {
    const opts = { autosave: 0, beta: false, theme: "light", openLast: false, units: "SI", contours: "rainbow", labels: true, deform: 0, paper: "A4", landscape: false, company: "" };
    window.__cur = f.models.blank;
    const handlers = {};
    const m = { examples_list: [...f.examples_list, { id: "big", name: "Speed check", what: "A large rack line." }], options: opts, startup_file: null, recovery: null, check_update: null, recent: [], version: "test", unit_table: f.unit_table, library: f.library, verification: f.verification ?? [] };
    window.__calls = [];
    window.__emit = (e, x) => (handlers[e] ?? []).forEach(h => h({ payload: x }));
    window.__TAURI__ = {
      core: {
        invoke: async (c, a) => {
          window.__calls.push([c, a]);
          if (c === "state") return window.__cur;
          if (c === "new_model") { window.__cur = f.models[a.example] ?? f.models.blank; return window.__cur; }
          // Every edit gives back a new copy of the model, as the shell does.
          if (c === "apply") {
            const nx = { ...JSON.parse(JSON.stringify(window.__cur)), dirty: true, undo: true, created: [999] };
            if (a.op.SetSettings) { nx.m.units = a.op.SetSettings.units; nx.m.settings = a.op.SetSettings.s; }
            if (a.op.Add?.item?.Support) nx.m.supports.push({ ...a.op.Add.item.Support, id: 999 });
            window.__cur = nx; return nx;
          }
          if (c === "history") return window.__cur;
          const curKey = () => Object.keys(f.models).find(k => f.models[k].m.name === window.__cur.m.name) ?? "";
          if (c === "analyze") { const r = f.results?.[curKey()]; if (r) return r; throw "The piping can move without resistance: add supports."; }
          if (c === "isometric") return f.iso?.[curKey()] ?? { sheets: [] };
          if (c in m) return m[c];
          return null;
        },
      },
      event: { listen: async (e, h) => { (handlers[e] ??= []).push(h); return () => {}; } },
    };
  }, fix);
  await p.goto(url);
  await p.waitForTimeout(900);
  const ms = async (what, f, done, arg = null) => {
    const t0 = Date.now();
    await f();
    await p.waitForFunction(done, arg, { timeout: 60000, polling: 50 });
    await p.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))));
    const t = Date.now() - t0;
    console.log(`${what}: ${t} ms`);
    return t;
  };
  const n = fix.models.big.m.elements.length;
  // Open the large model: the table and the 3D view drawn.
  await p.evaluate(() => document.querySelector(".tabs .file").click()); await p.waitForTimeout(150);
  const cdp = process.env.PROFILE ? await p.context().newCDPSession(p) : null;
  if (cdp) { await cdp.send("Profiler.enable"); await cdp.send("Profiler.setSamplingInterval", { interval: 200 }); await cdp.send("Profiler.start"); }
  const open = await ms("open the large model", () => p.click('.home .tile:has(b:text-is("Speed check"))'), n => (document.querySelector(".etable header")?.textContent ?? "").includes(`${n} elements`) && !!document.querySelector(".v3d canvas"), n);
  if (cdp) {
    const { profile } = await cdp.send("Profiler.stop");
    const self = new Map(); const byId = new Map(profile.nodes.map(x => [x.id, x]));
    const dt = profile.timeDeltas; const counts = new Map();
    profile.samples.forEach((id, i) => counts.set(id, (counts.get(id) ?? 0) + (dt[i] ?? 0)));
    for (const [id, t] of counts) { const nd = byId.get(id); const k = `${nd.callFrame.functionName || "(anon)"} ${nd.callFrame.url.split("/").pop()}:${nd.callFrame.lineNumber}`; self.set(k, (self.get(k) ?? 0) + t); }
    console.log([...self].sort((a, b) => b[1] - a[1]).slice(0, 18).map(([k, t]) => `${(t / 1000).toFixed(0)} ms  ${k}`).join("\n"));
  }
  await p.waitForTimeout(1500);
  // One edit in the middle of the table: the engine's answer drawn again (and analysed automatically).
  await p.evaluate(n => { document.querySelector(".escroll").scrollTop = Math.floor(n / 2) * 25; }, n); await p.waitForTimeout(300);
  const cell = await p.$(`.egrid tbody tr:not(.spacer):nth-child(20) td:nth-child(4) input`);
  check("the table draws only the rows in view", (await p.$$(".egrid tbody tr")).length < 120);
  const before = await p.evaluate(() => window.__calls.length);
  if (cdp) { await cdp.send("Profiler.start"); }
  const edit = await ms("one edit, drawn again", async () => { await cell.fill("6500"); await cell.press("Tab"); }, b => window.__calls.filter(c => c[0] === "apply").length > 0 && window.__calls.length > b, before);
  if (cdp) {
    const { profile } = await cdp.send("Profiler.stop");
    const self = new Map(); const byId = new Map(profile.nodes.map(x => [x.id, x]));
    const dt = profile.timeDeltas; const counts = new Map();
    profile.samples.forEach((id, i) => counts.set(id, (counts.get(id) ?? 0) + (dt[i] ?? 0)));
    for (const [id, t] of counts) { const nd = byId.get(id); const k = `${nd.callFrame.functionName || "(anon)"} ${nd.callFrame.url.split("/").pop()}:${nd.callFrame.lineNumber}`; self.set(k, (self.get(k) ?? 0) + t); }
    console.log("EDIT\n" + [...self].sort((a, b) => b[1] - a[1]).slice(0, 14).map(([k, t]) => `${(t / 1000).toFixed(0)} ms  ${k}`).join("\n"));
  }
  await p.waitForTimeout(1500);
  // The results: code stresses of the expansion case.
  await p.click('.side button[title="Results"]');
  const res = await ms("results tables", async () => { await p.click('.rtabs button:text-is("Code stresses")'); }, () => document.querySelectorAll(".rtable tbody tr").length > 100);
  check(`opening ${n} elements takes under 3.5 s (${open} ms)`, open < 3500);
  check(`an edit is drawn again in under 1.2 s (${edit} ms)`, edit < 1200);
  check(`the code stresses table appears in under 1.5 s (${res} ms)`, res < 1500);
  if (process.env.SHOT_COMPRESSOR) {
    await p.click('.side button[title="Model"]');
    await p.evaluate(() => document.querySelector(".tabs .file").click()); await p.waitForTimeout(200);
    await p.click('.home .tile:has(b:text-is("Compressor suction to a header"))'); await p.waitForTimeout(400);
    if (await p.$('.dlg header:has-text("Save changes?")')) { await p.click('.dlg button:has-text("Don\'t save")'); await p.waitForTimeout(400); }
    await p.waitForTimeout(1500);
    await p.screenshot({ path: process.env.SHOT_COMPRESSOR });
  }
  check("no errors in the page", !errors.length, errors.join(" | "));
  await b.close(); srv.close();
  process.exit(failed ? 1 : 0);
})();
