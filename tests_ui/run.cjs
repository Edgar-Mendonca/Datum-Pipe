// UI tests: the built window (ui/dist) in Chromium with the engine's real models and results
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
          if (c === "apply") { window.__cur = { ...window.__cur, dirty: true, undo: true, created: [999] }; return window.__cur; }
          if (c === "history") return window.__cur;
          if (c === "solve") { const r = f.results?.[window.__curId ?? ""]; if (r) return r; throw "The piping can move without resistance: add supports."; }
          if (c === "iso") return f.iso?.[window.__curId ?? ""] ?? { sheets: [] };
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

  // 1. File page: every worked example, the moss sidebar, the update card.
  const tiles = await p.$$eval(".home .tile b", els => els.map(e => e.textContent));
  check("File page shows every worked example", fix.examples_list.every(e => tiles.includes(e.name)), JSON.stringify(tiles));
  check("File sidebar is the moss accent", (await p.$eval(".bs aside", el => getComputedStyle(el).backgroundColor)) === "rgb(77, 124, 15)");
  check("the update card names Datum Pipe", (await p.textContent(".card .brand")).includes("Datum Pipe"));
  if (shots) await p.screenshot({ path: `${shots}/file.png` });

  // 2. A blank model: ribbon tabs, the table's empty state, the Inspector shows the model.
  await open("blank");
  check("ribbon tabs", JSON.stringify(await p.$$eval(".tabs > button:not(.file)", els => els.map(e => e.textContent))) === JSON.stringify(["Home", "Model", "Loads", "Supports", "Analyze", "Results", "Isometric", "View", "Help"]));
  check("the table says where to start", (await p.textContent(".etable")).includes("Start here"));
  check("the Inspector shows the model's settings", (await p.textContent(".insp")).includes("Installed temperature"));
  check("the installed temperature is shown in °C", (await p.textContent(".insp")).includes("°C"));

  // 3. Typing 3000 (mm) in DX adds an element of 3 m from node 10 to node 20.
  const dx = await p.$('.newrow input[placeholder="DX"]');
  await dx.fill("3000"); await dx.press("Tab"); await p.waitForTimeout(200);
  const add = await lastOp();
  check("typing DX in the last row adds an element", !!add?.Add?.item?.Element && add.Add.item.Element.from === 10 && add.Add.item.Element.to === 20 && Math.abs(add.Add.item.Element.d[0] - 3) < 1e-12, JSON.stringify(add));

  // 4. Units: US sends SetSettings; Options shows the units in words.
  await p.click('.ribbon button[title="US: ft, in, lbf, psi"]'); await p.waitForTimeout(150);
  check("US units send one settings edit", (await lastOp())?.SetSettings?.units === "US");

  // 5. Supports need a node: with nothing selected a support explains what to do.
  await p.click('.tabs > button:text-is("Supports")');
  await p.click('.ribbon button[title="Anchor"]'); await p.waitForTimeout(150);
  check("a support with nothing selected says to pick a node", (await p.textContent("#toast")).includes("Select a node"));

  // 6. Property dialogs: pipes with library sizes, materials with typical allowables marked.
  await p.click('.tabs > button:text-is("Model")');
  await p.click('.ribbon button[title="Pipes"]'); await p.waitForTimeout(300);
  check("the pipes dialog offers the library sizes", (await p.$$eval(".dlg select option", els => els.map(e => e.textContent))).includes('24"'));
  await p.keyboard.press("Escape"); await p.waitForTimeout(100);
  await p.click('.ribbon button[title="Materials"]'); await p.waitForTimeout(300);
  check("materials say allowables are inputs to check", (await p.textContent(".dlg")).includes("check them against your code edition"));
  await p.keyboard.press("Escape"); await p.waitForTimeout(100);

  // 7. Load cases dialog lists the recommended cases.
  await p.click('.tabs > button:text-is("Loads")');
  await p.click('.ribbon button[title="Load Cases"]'); await p.waitForTimeout(300);
  const caseNames = await p.$$eval(".dlg .tbl.cases tbody tr td:first-child input", els => els.map(e => e.value));
  check("load cases: OPE1, SUS, EXP1", JSON.stringify(caseNames) === JSON.stringify(["OPE1", "SUS", "EXP1"]), JSON.stringify(caseNames));
  await p.keyboard.press("Escape"); await p.waitForTimeout(100);

  // 8. Print preview, report view, help cards, Tell Me, dark theme.
  await p.keyboard.press("Control+p"); await p.waitForTimeout(500);
  check("print preview shows a page", (await p.$$(".pprev .ppage")).length >= 1);
  check("the report says allowables are the user's inputs", (await p.textContent(".pprev")).includes("Allowable stresses are the user's inputs"));
  await p.keyboard.press("Escape"); await p.waitForTimeout(200);
  await p.click('.side button[title="Report"]'); await p.waitForTimeout(200);
  check("the Report view shows the input echo", (await p.textContent(".rsheet")).includes("Line conditions"));
  await p.click('.side button[title="Model"]'); await p.waitForTimeout(150);
  await p.click(".etable .help"); await p.waitForTimeout(150);
  check("the ? beside the table opens its card", (await p.textContent(".dlg")).includes("DX, DY or DZ"));
  await p.keyboard.press("Escape");
  await p.keyboard.press("Control+k"); await p.waitForTimeout(150);
  await p.keyboard.type("rest");
  check("Tell Me finds the rest support", (await p.textContent(".pal .pl")).includes("Rest"));
  await p.keyboard.press("Escape");
  await p.click(".theme"); await p.waitForTimeout(150);
  check("dark theme applies", await p.evaluate(() => document.documentElement.dataset.theme === "dark"));
  if (shots) await p.screenshot({ path: `${shots}/dark.png` });
  await p.click(".theme"); await p.click(".theme"); await p.waitForTimeout(100);

  check("no errors in the page", errors.length === 0, errors.join(" | "));
  await b.close(); srv.close();
  console.log(failed ? `\n${failed} UI check(s) failed` : "\nAll UI checks passed");
  process.exit(failed ? 1 : 0);
})().catch(e => { console.error(e); process.exit(1); });
