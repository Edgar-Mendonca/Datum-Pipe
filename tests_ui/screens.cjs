// Screenshots for the README and the family site (docs/images, 1440×900): the built window (ui/dist) in Chromium with the engine's real models and results
// (cargo run -p datum-core --example dump > tests_ui/fixture.json) behind a mocked Tauri bridge
// that records every call. Run: npm run test:ui (or npm run build:ui && node tests_ui/run.cjs).
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
    const m = { examples_list: f.examples_list, options: opts, startup_file: null, recovery: null, check_update: null, recent: [], version: "test", unit_table: f.unit_table, library: f.library, verification: f.verification ?? [] };
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
  const ops = () => p.evaluate(() => window.__calls.filter(c => c[0] === "apply").map(c => c[1].op));
  const lastOp = async () => (await ops()).slice(-1)[0];
  const open = async id => {
    await p.evaluate(() => document.querySelector(".tabs .file").click()); await p.waitForTimeout(150);
    const n = fix.examples_list.find(e => e.id === id).name; await p.click(`.home .tile:has(b:text-is("${n}"))`); await p.waitForTimeout(250);
    if (await p.$('.dlg header:has-text("Save changes?")')) { await p.click('.dlg button:has-text("Don\'t save")'); await p.waitForTimeout(300); }
    await p.evaluate(i => { window.__curId = i; }, id);
    await p.waitForTimeout(200);
  };

  const clean = async () => { await p.evaluate(() => { const t = document.getElementById("toast"); if (t) t.remove(); }); await p.waitForTimeout(100); };
  const out = process.env.OUT || "docs/images";
  // 1. The steam line coloured by stress ratio, with the deformed shape.
  await open("steam");
  await p.keyboard.press("F5"); await p.waitForTimeout(600);
  await p.click('.egrid tbody tr:nth-child(5) td.rn'); await p.waitForTimeout(100);
  await p.evaluate(() => document.querySelector(".v3d")?.dispatchEvent(new Event("pointerdown")));
  await p.keyboard.press("Escape");
  await p.click('.ribbon button[title="Deformed Shape"]'); await p.waitForTimeout(400);
  await p.mouse.click(700, 300); await p.waitForTimeout(200);
  await clean(); await p.screenshot({ path: `${out}/model-stress.png` });
  // 2. The isometric of the pump discharge line with stress ratio tags.
  await open("pump");
  await p.keyboard.press("F5"); await p.waitForTimeout(600);
  await p.click('.side button[title="Isometric"]'); await p.waitForTimeout(500);
  await clean(); await p.screenshot({ path: `${out}/isometric.png` });
  // 3. The results tables: support loads of the steam line.
  await open("steam");
  await p.keyboard.press("F5"); await p.waitForTimeout(600);
  await p.click('.side button[title="Results"]'); await p.waitForTimeout(300);
  await p.click('.rtabs button:text-is("Code stresses")'); await p.waitForTimeout(200);
  await clean(); await p.screenshot({ path: `${out}/results.png` });
  // 4. The printed stress report.
  await p.keyboard.press("Control+p"); await p.waitForTimeout(900);
  await clean(); await p.screenshot({ path: `${out}/report.png` });
  if (process.env.DEBUG) console.log(await p.evaluate(() => { const pg = document.querySelector(".pprev .ppage"); const f = pg.querySelector(".pfoot").getBoundingClientRect(); return [...pg.querySelectorAll("*")].filter(e => e.getBoundingClientRect().top > f.bottom - 1 && e.getBoundingClientRect().height > 0).map(e => e.tagName + "." + e.className + " " + (e.textContent || "").slice(0, 40)).slice(0, 10); }));
  await b.close(); srv.close();
})().catch(e => { console.error(e); process.exit(1); });
