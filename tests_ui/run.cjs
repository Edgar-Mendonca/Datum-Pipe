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
  await p.click('.insp label.chk:has-text("Pressure elongation") input'); await p.waitForTimeout(200);
  check("the pressure elongation setting sends one settings edit", (await lastOp())?.SetSettings?.s?.pressure_elongation === true);
  await p.click('.insp label.chk:has-text("Branch section modulus") input'); await p.waitForTimeout(200);
  check("the branch section modulus setting sends one settings edit", (await lastOp())?.SetSettings?.s?.branch_ze === false);
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

  // 9. The L-bend example: the 3D view draws it, the table lists it, Analyze shows results.
  await open("lbend");
  check("the table lists the L-bend's elements", (await p.$$(".egrid tbody tr:not(.newrow)")).length === fix.models.lbend.m.elements.length);
  check("the 3D view has a canvas", !!(await p.$(".v3d canvas")));
  check("node numbers are drawn", (await p.$$eval(".nlabel", els => els.map(e => e.textContent))).includes("20"));
  // Typing across a row keeps the keyboard in the table after each edit (0.1.2: every row was rebuilt).
  const dy = await p.$('.egrid tbody tr:first-child td:nth-child(5) input');
  await dy.click(); await dy.fill("100"); await p.keyboard.press("Tab"); await p.waitForTimeout(300);
  check("Tab after an edit moves to the next cell and keeps the focus", await p.evaluate(() => { const a = document.activeElement; return !!a && a.closest("td")?.cellIndex === 5 && a.closest("tr")?.rowIndex === 1; }));
  await p.click('.egrid tbody tr:first-child td.rn'); await p.waitForTimeout(150);
  check("clicking a row selects the element", (await p.textContent(".insp header")).includes("Element"));
  if (shots) await p.screenshot({ path: `${shots}/lbend.png` });
  await p.keyboard.press("F5"); await p.waitForTimeout(500);
  check("Analyze reports the load cases and the equilibrium", (await p.textContent("#toast")).includes("Reactions balance the loads in every case"));
  check("the legend shows the stress ratio", !!(await p.$(".legend3d")));
  if (shots) await p.screenshot({ path: `${shots}/lbend-results.png` });
  await p.click('.side button[title="Results"]'); await p.waitForTimeout(200);
  const sumTxt = await p.textContent(".rtable");
  check("the summary lists SUS and EXP1 with their checks", sumTxt.includes("SUS") && sumTxt.includes("EXP1") && sumTxt.includes("PASS"));
  await p.click('.rtabs button:text-is("Support loads")'); await p.waitForTimeout(150);
  check("support loads list both anchors", (await p.$$(".rtable tbody tr")).length === 2);
  check("support loads show the state of one-way and gapped supports", (await p.textContent(".rtable thead")).includes("State"));
  await p.click('.rtabs button:text-is("Equilibrium")'); await p.waitForTimeout(150);
  check("the equilibrium table ticks every case", (await p.$$eval(".rtable tbody tr td:last-child", els => els.every(e => e.textContent.includes("✓")))));
  await p.click('.rtabs button:text-is("Code stresses")'); await p.waitForTimeout(150);
  check("code stresses are shown in MPa", (await p.textContent(".rtable thead")).includes("MPa"));
  if (shots) await p.screenshot({ path: `${shots}/results.png` });
  await p.click('.side button[title="Model"]'); await p.waitForTimeout(200);
  await p.click('.tabs > button:text-is("Help")');
  await p.click('.ribbon button[title="Verification"]'); await p.waitForTimeout(400);
  check("Help › Verification: every check passes", (await p.textContent(".dlg .verif")).includes(`${fix.verification.length} of ${fix.verification.length} checks pass`));
  await p.keyboard.press("Escape");
  await p.click('.ribbon button[title="Where each equation is published"]'); await p.waitForTimeout(300);
  const src = await p.textContent(".dlg");
  check("Help › Sources lists where each equation is published", src.includes("not affiliated with or endorsed by ASME") && src.includes("Markl") && (await p.$$(".dlg .tbl tbody tr")).length >= 10);
  await p.keyboard.press("Escape");

  // The expansion loop wizard sends one Loop edit for the selected straight element.
  await p.click('.egrid tbody tr:nth-child(2) td.rn'); await p.waitForTimeout(150);
  await p.click('.tabs > button:text-is("Model")');
  await p.click('.ribbon button[title="Turn the selected straight element into an expansion loop"]'); await p.waitForTimeout(250);
  check("the loop wizard draws the loop", !!(await p.$(".dlg .looppic path")));
  await p.click('.dlg .btn.pri'); await p.waitForTimeout(250);
  const lp = await lastOp();
  check("the loop wizard sends a Loop edit", !!lp?.Loop && lp.Loop.width > 0 && lp.Loop.height > 0, JSON.stringify(lp));
  await p.click('.ribbon button[title="Copy Section"]'); await p.waitForTimeout(250);
  await p.click('.dlg .btn.pri'); await p.waitForTimeout(250);
  check("copy a section sends a Duplicate edit", !!(await lastOp())?.Duplicate);
  // A new rest is one-way; its Inspector offers the one-way switch and the gap.
  await p.click('.egrid tbody tr:nth-child(2) td.rn'); await p.waitForTimeout(150);
  await p.click('.tabs > button:text-is("Supports")');
  await p.click('.ribbon button[title="Rest"]'); await p.waitForTimeout(250);
  const rest = (await lastOp())?.Add?.item?.Support;
  check("a new rest is one-way with no gap", rest?.kind?.kind === "rest" && rest.one_way === true && rest.gap === 0, JSON.stringify(rest));
  const ins = await p.textContent(".insp");
  check("the rest's Inspector offers one-way and the gap", ins.includes("One-way: the pipe can lift off") && ins.includes("Gap below the pipe"), ins.slice(0, 200));
  await p.click('.tabs > button:text-is("Model")');

  // US units: lengths in feet, stresses in psi, the isometric in feet and inches.
  await p.click('.tabs > button:text-is("Home")');
  await p.click('.ribbon button[title="US: ft, in, lbf, psi"]'); await p.waitForTimeout(300);
  check("US units: the L-bend's 6000 mm leg shows as 19.685 ft", (await p.textContent(".egrid tbody tr:first-child")).includes("19.685") && (await p.textContent(".etable header")).includes("ft"));
  await p.click('.ribbon button[title="SI: mm, N, MPa, bar"]'); await p.waitForTimeout(200);

  // 10. The pump example: deformed shape, nozzle loads, the report's results pages.
  await open("pump");
  await p.keyboard.press("F5"); await p.waitForTimeout(500);
  await p.click('.ribbon button[title="Deformed Shape"]'); await p.waitForTimeout(300);
  check("the deformed shape shows its scale", (await p.textContent(".legend3d")).includes("largest move"));
  if (shots) await p.screenshot({ path: `${shots}/pump-deformed.png` });
  await p.click('.ribbon button[title="Nozzle Loads"]'); await p.waitForTimeout(200);
  check("nozzle loads name the pump nozzle with a result", (await p.textContent(".rtable")).includes("P-101 discharge") && (await p.$$(".rtable .pass")).length >= 1);
  await p.click('.side button[title="Report"]'); await p.waitForTimeout(300);
  const rep = await p.textContent(".rsheet");
  check("the report has the stress summary, support and nozzle loads and the equilibrium", rep.includes("3. Stress summary") && rep.includes("4. Support loads") && rep.includes("5. Nozzle loads") && rep.includes("7. Equilibrium checks"));
  await p.click('.side button[title="Model"]'); await p.waitForTimeout(200);

  // 11. The isometric: lines, dimensions, symbols, tags; it prints with the report.
  await p.click('.side button[title="Isometric"]'); await p.waitForTimeout(400);
  check("the isometric draws every element's dimension", (await p.$$(".isosvg .dim")).length === fix.models.pump.m.elements.length);
  check("the isometric shows the stress ratio tags", (await p.textContent(".isosvg")).includes("0.") && (await p.textContent(".isosvg")).includes("P-101"));
  if (shots) await p.screenshot({ path: `${shots}/iso.png` });
  await p.keyboard.press("Control+p"); await p.waitForTimeout(700);
  check("the printout includes the isometric sheet", (await p.$$(".pprev .isosvg")).length >= 1);
  if (shots) await p.screenshot({ path: `${shots}/print.png` });
  await p.keyboard.press("Escape"); await p.waitForTimeout(200);
  await p.click('.side button[title="Model"]'); await p.waitForTimeout(200);

  // 12. Every example opens, analyzes, draws its isometric and prints its report.
  for (const ex of fix.examples_list.filter(e => e.id !== "blank")) {
    await open(ex.id);
    await p.keyboard.press("F5"); await p.waitForTimeout(400);
    const ok = (await p.textContent("#toast")).includes("Reactions balance the loads in every case");
    await p.click('.side button[title="Isometric"]'); await p.waitForTimeout(300);
    const dims = (await p.$$(".isosvg .dim")).length;
    await p.keyboard.press("Control+p"); await p.waitForTimeout(700);
    const pages = (await p.$$(".pprev .ppage")).length;
    await p.keyboard.press("Escape"); await p.waitForTimeout(200);
    await p.click('.side button[title="Model"]'); await p.waitForTimeout(150);
    check(`${ex.name}: analyzes, draws its isometric, prints ${pages} pages`, ok && dims > 0 && pages >= 3, `${ok} ${dims} ${pages}`);
  }

  check("no errors in the page", errors.length === 0, errors.join(" | "));
  await b.close(); srv.close();
  console.log(failed ? `\n${failed} UI check(s) failed` : "\nAll UI checks passed");
  process.exit(failed ? 1 : 0);
})().catch(e => { console.error(e); process.exit(1); });
