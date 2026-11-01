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
            if (a.op.SetReliefs) nx.m.reliefs = a.op.SetReliefs.list;
            if (a.op.SetVessels) nx.m.vessels = a.op.SetVessels.list;
            window.__cur = nx; return nx;
          }
          if (c === "history") return window.__cur;
          const curKey = () => Object.keys(f.models).find(k => f.models[k].m.name === window.__cur.m.name) ?? "";
          if (c === "analyze") { const r = f.results?.[curKey()]; if (r) return r; throw "The piping can move without resistance: add supports."; }
          if (c === "isometric") return f.iso?.[curKey()] ?? { sheets: [] };
          if (c === "vessel_check") return f.vessels?.[curKey()] ?? [];
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
  // File › Export › Import asks the shell to import a piping component file.
  await p.evaluate(() => document.querySelector(".tabs .file").click()); await p.waitForTimeout(150);
  await p.evaluate(() => [...document.querySelectorAll("button")].find(b => b.textContent.trim() === "Export")?.click());
  await p.waitForTimeout(150);
  await p.click('.tile:has(b:text-is("Import a piping component file"))'); await p.waitForTimeout(250);
  if (await p.$('.dlg header:has-text("Save changes?")')) { await p.click('.dlg button:has-text("Don\'t save")'); await p.waitForTimeout(250); }
  check("File › Export › Import calls the shell's import", await p.evaluate(() => window.__calls.some(c => c[0] === "import_pcf")));
  await p.keyboard.press("Escape"); await p.waitForTimeout(150);
  check("ribbon tabs", JSON.stringify(await p.$$eval(".tabs > button:not(.file)", els => els.map(e => e.textContent))) === JSON.stringify(["Home", "Model", "Loads", "Supports", "Analyze", "Results", "Isometric", "View", "Help"]));
  check("the table says where to start", (await p.textContent(".etable")).includes("Start here"));
  check("the Inspector shows the model's settings", (await p.textContent(".insp")).includes("Installed temperature"));
  await p.click('.insp label.chk:has-text("Pressure elongation") input'); await p.waitForTimeout(200);
  check("the pressure elongation setting sends one settings edit", (await lastOp())?.SetSettings?.s?.pressure_elongation === true);
  check("the Inspector has the occasional loads", (await p.textContent(".insp")).includes("Design wind pressure") && (await p.textContent(".insp")).includes("Static seismic"));
  await p.click('.insp label.chk:has-text("Branch section modulus") input'); await p.waitForTimeout(200);
  check("the branch section modulus setting sends one settings edit", (await lastOp())?.SetSettings?.s?.branch_ze === false);
  check("the installed temperature is shown in °C", (await p.textContent(".insp")).includes("°C"));

  // 3. Typing 3000 (mm) in DX adds an element of 3 m from node 10 to node 20.
  const dx = await p.$('.newrow input[placeholder="DX"]');
  await dx.fill("3000"); await dx.press("Tab"); await p.waitForTimeout(200);
  const add = await lastOp();
  check("typing DX in the last row adds an element", !!add?.Add?.item?.Element && add.Add.item.Element.from === 10 && add.Add.item.Element.to === 20 && Math.abs(add.Add.item.Element.d[0] - 3) < 1e-12, JSON.stringify(add));
  // Values can carry their units: 10 ft in a field shown in mm, and feet-and-inches.
  const dz = await p.$('.newrow input[placeholder="DZ"]');
  await dz.fill("10 ft"); await dz.press("Tab"); await p.waitForTimeout(200);
  check("a length typed in feet goes in as metres", Math.abs((await lastOp())?.Add?.item?.Element?.d?.[2] - 3.048) < 1e-12, JSON.stringify(await lastOp()));
  const dy0 = await p.$('.newrow input[placeholder="DY"]');
  await dy0.fill("3' 6\""); await dy0.press("Tab"); await p.waitForTimeout(200);
  check("feet and inches add up", Math.abs((await lastOp())?.Add?.item?.Element?.d?.[1] - 1.0668) < 1e-12, JSON.stringify(await lastOp()));
  const n0 = (await ops()).length;
  await dy0.fill("3 kg"); await dy0.press("Tab"); await p.waitForTimeout(200);
  check("a unit of the wrong kind is refused with a reason", (await ops()).length === n0 && (await p.textContent("#toast")).includes("not a unit of length"));

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
  // Right-click on empty space in the 3D view offers Fit and Analyze; Escape closes the menu.
  const cv = await (await p.$(".v3d canvas")).boundingBox();
  await p.mouse.click(cv.x + 12, cv.y + cv.height - 12, { button: "right" }); await p.waitForTimeout(200);
  check("right-clicking the 3D view opens a menu", ((await p.textContent(".menu").catch(() => "")) ?? "").includes("Fit the model"));
  await p.keyboard.press("Escape"); await p.mouse.click(cv.x + 12, cv.y + 12); await p.waitForTimeout(150);
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
  await p.click('.rtabs button:text-is("Hangers")'); await p.waitForTimeout(150);
  check("the Hangers table says how to add one when there is none", (await p.textContent("body")).includes("Supports › Spring Hanger"));
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
  check("the rest's Inspector offers one-way, the gap and friction", ins.includes("One-way: the pipe can lift off") && ins.includes("Gap below the pipe") && ins.includes("Friction coefficient"), ins.slice(0, 200));
  check("a new rest has no friction until it is given one", rest?.mu === 0);
  await p.click('.ribbon button[title="The springs hangers are chosen from, and the load variation allowed"]'); await p.waitForTimeout(300);
  check("the spring table lists the generic sizes and says to use the supplier's", (await p.$$(".dlg .tbl tbody tr")).length === 60 && (await p.textContent(".dlg")).includes("supplier"));
  await p.keyboard.press("Escape"); await p.waitForTimeout(100);
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
  for (const ex of fix.examples_list.filter(e => e.id !== "blank" && fix.models[e.id].m.elements.length)) {
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

  // 13. Rotating equipment: the steam line's turbine in Results › Equipment, and the dialog.
  await open("steam");
  await p.keyboard.press("F5"); await p.waitForTimeout(400);
  await p.click('.side button[title="Results"]'); await p.waitForTimeout(200);
  await p.click('.rtabs button:text-is("Equipment")'); await p.waitForTimeout(200);
  await p.click('.ctabs button:has(b:text-is("OPE1"))'); await p.waitForTimeout(200);
  const eqText = await p.textContent(".results, body");
  check("Results › Equipment shows the turbine's criteria", eqText.includes("T-301") && eqText.includes("3F + 1M ≤ 500·De"), eqText.slice(0, 200));
  await p.click('.side button[title="Model"]'); await p.waitForTimeout(150);
  await p.click('.tabs > button:text-is("Supports")');
  await p.click('.ribbon button[title^="Group nozzles into pumps"]'); await p.waitForTimeout(300);
  check("the equipment dialog lists the turbine and its criteria", ((await p.textContent(".dlg")) ?? "").includes("All nozzles resolved at") && (await p.$$(".dlg .eqcard")).length === 1);
  await p.keyboard.press("Escape"); await p.waitForTimeout(150);

  // 14. Flange leakage: the pump's valve flanges in Results › Flanges, and the dialog.
  await open("pump");
  await p.keyboard.press("F5"); await p.waitForTimeout(400);
  await p.click('.side button[title="Results"]'); await p.waitForTimeout(200);
  await p.click('.rtabs button:text-is("Flanges")'); await p.waitForTimeout(200);
  await p.click('.ctabs button:has(b:text-is("OPE1"))'); await p.waitForTimeout(200);
  check("Results › Flanges shows the valve's equivalent pressure", (await p.$$(".rtable tbody tr")).length === 1 && (await p.textContent(".rtable thead")).includes("P_eq"));
  await p.click('.side button[title="Model"]'); await p.waitForTimeout(150);
  await p.click('.tabs > button:text-is("Loads")');
  await p.click('.ribbon button[title^="Check flanged joints"]'); await p.waitForTimeout(300);
  check("the flange dialog lists the valve with its gasket and rating", (await p.$$(".dlg .tbl tbody tr")).length === 1 && ((await p.textContent(".dlg")) ?? "").includes("16M/(πG³)"));
  await p.keyboard.press("Escape"); await p.waitForTimeout(150);

  // 15. Relief valve thrust: the vent stack's thrust in Results › Relief, its cases, and the dialog.
  await open("relief");
  await p.keyboard.press("F5"); await p.waitForTimeout(400);
  await p.click('.side button[title="Results"]'); await p.waitForTimeout(200);
  await p.click('.rtabs button:text-is("Relief")'); await p.waitForTimeout(200);
  check("Results › Relief shows the valve's thrust", (await p.$$(".rtable tbody tr")).length === 1 && ((await p.textContent(".rtable tbody")) ?? "").includes("PSV-1"));
  await p.click('.rtabs button:text-is("Code stresses")'); await p.waitForTimeout(200);
  check("the relief cases OPE1+RV and OCC RV are there", (await p.$$('.ctabs button:has(b:text-is("OCC RV"))')).length === 1 && (await p.$$('.ctabs button:has(b:text-is("OPE1+RV"))')).length === 1);
  await p.click('.side button[title="Model"]'); await p.waitForTimeout(150);
  await p.click('.tabs > button:text-is("Loads")');
  await p.click('.ribbon button[title^="Relief valves discharging"]'); await p.waitForTimeout(300);
  check("the relief dialog lists the valve at its open end", (await p.$$(".dlg .tbl tbody tr")).length === 1 && ((await p.textContent(".dlg")) ?? "").includes("DLF"));
  await p.click('.dlg button:text-is("Add a relief valve")'); await p.waitForTimeout(300);
  check("adding a relief valve adds a row", (await p.$$(".dlg .tbl tbody tr")).length === 2);
  await p.click('.dlg .tbl tbody tr:nth-child(2) button.x'); await p.waitForTimeout(300);
  check("removing it takes the row away", (await p.$$(".dlg .tbl tbody tr")).length === 1);
  await p.click('.dlg button:text-is("Close")'); await p.waitForTimeout(150);
  check("the relief dialog closes", (await p.$$(".dlg")).length === 0);

  // 16. Nozzle flexibility: the tank outlet's wall stiffness in the nozzle's Inspector.
  await open("tank");
  await p.click('.egrid tbody tr:nth-child(1) td.rn'); await p.waitForTimeout(150);
  await p.click('.insp button:text-is("Node 10")'); await p.waitForTimeout(150);
  await p.click('.insp button.ti:has-text("TK-401 outlet")'); await p.waitForTimeout(150);
  const insp = (await p.textContent(".insp")) ?? "";
  const vals = await p.$$eval(".insp input", xs => xs.map(x => x.value));
  check("the nozzle Inspector has the wall flexibility with the example's stiffness", insp.includes("Wall flexibility") && insp.includes("Bending, circ.") && vals.includes("35000"), vals.join(","));

  // 17. Expansion joints: the example's joint in the Inspector, analysed, and on the isometric.
  await open("bellows");
  await p.click('.egrid tbody tr:nth-child(3) td.rn'); await p.waitForTimeout(150);
  const bi = (await p.textContent(".insp")) ?? "";
  check("the expansion joint's Inspector has its rates and the tied box", bi.includes("Axial rate") && bi.includes("Lateral rate") && bi.includes("Tied"));
  check("the table names the component", ((await p.textContent(".egrid")) ?? "").includes("Expansion joint"));
  await p.keyboard.press("F5"); await p.waitForTimeout(400);
  await p.click('.side button[title="Isometric"]'); await p.waitForTimeout(300);
  check("the isometric draws", (await p.$$(".isosvg path")).length > 5);
  await p.click('.side button[title="Model"]'); await p.waitForTimeout(150);

  // 18. Wind by height and both directions: the Inspector's profile editor and the checkbox.
  await open("blank");
  await p.click('.insp button:text-is("Wind by height")'); await p.waitForTimeout(200);
  const prof = await p.evaluate(() => window.__calls.filter(c => c[0] === "apply" && c[1].op.SetSettings).map(c => c[1].op.SetSettings.s.wind_profile).pop());
  check("Wind by height starts a profile of two rows", Array.isArray(prof) && prof.length === 2 && prof[1][0] > prof[0][0], JSON.stringify(prof));
  check("the profile table shows", (await p.$$('.insp .tbl tbody tr')).length === 2);
  await p.click('.insp label.chk:has-text("both directions") input'); await p.waitForTimeout(200);
  const both = await p.evaluate(() => window.__calls.filter(c => c[0] === "apply" && c[1].op.SetSettings).map(c => c[1].op.SetSettings.s.occ_both).pop());
  check("the both-directions box sets the option", both === true);

  // 19. Export a piping component file from File › Export.
  await open("lbend");
  await p.click('button.file'); await p.waitForTimeout(250);
  await p.click('.bs aside button:has-text("Export")'); await p.waitForTimeout(200);
  await p.click('button.tile:has-text("Export a piping component file")'); await p.waitForTimeout(300);
  check("Export a piping component file asks the engine for the file", await p.evaluate(() => window.__calls.some(c => c[0] === "export_pcf")));

  // 20. Vessels: the separator example's vessel, its drawing and results; a new vessel; the report.
  await open("separator");
  await p.click('.side button[title="Vessels"]'); await p.waitForTimeout(300);
  const vt = (await p.textContent(".vpage")) ?? "";
  check("the Vessels view shows V-101 with its MAWP and test pressure", vt.includes("V-101") && vt.includes("MAWP at the top") && vt.includes("Hydrotest pressure"));
  check("the vessel is drawn", (await p.$$(".vdraw path")).length >= 2);
  check("every part has a result row", (await p.$$(".vtables table:first-of-type tbody tr")).length === 3);
  check("every nozzle has a reinforcement row, all passing", (await p.$$(".vtables table:nth-of-type(2) tbody tr")).length === 4 && (await p.$$(".vtables table:nth-of-type(2) .fail")).length === 0);
  check("the nozzles are drawn", (await p.$$(".vdraw g path")).length >= 8);
  check("the nozzle editor lists them with their sizes", (await p.$$(".vleft table:nth-of-type(2) tbody tr")).length === 4);
  check("the shell governs", ((await p.textContent(".vsum")) ?? "").includes("set by Shell"));
  check("V-101 shows wind, seismic, the skirt, bolts and base ring", vt.includes("Wind, seismic and the skirt") && vt.includes("Anchor bolts") && vt.includes("Base ring thickness") && vt.includes("Skirt (base)"));
  check("the skirt editor shows for a vertical vessel", vt.includes("On a skirt") && vt.includes("Bolt circle"));
  const scheds = await p.$$eval(".vleft table:nth-of-type(2) tbody tr td:nth-child(5) select:nth-of-type(2)", xs => xs.map(x => x.value));
  check("each nozzle's schedule box shows its own schedule", scheds[0] === "80" && scheds[2] === "160", scheds.join(","));
  // Picking a size sends the library's dimensions (in metres) unchanged.
  await p.selectOption(".vleft table:nth-of-type(2) tbody tr:nth-child(1) td:nth-child(5) select >> nth=0", "6"); await p.waitForTimeout(250);
  const nn = await p.evaluate(() => window.__calls.filter(c => c[0] === "apply" && c[1].op.SetVessels).map(c => c[1].op.SetVessels.list[0].nozzles[0]).pop());
  check("picking a nozzle size sends its outside diameter in metres", Math.abs(nn.od - 0.168275) < 1e-6 && nn.t > 0.005 && nn.t < 0.02, JSON.stringify(nn));
  check("V-101's lifting lugs are checked", ((await p.textContent(".vpage")) ?? "").includes("Lug: tension across the hole") && ((await p.textContent(".vpage")) ?? "").includes("Lifting lugs and trunnions"));
  check("V-101's manway cover is checked by the bolted flange method", vt.includes("Bolted flanges and covers") && vt.includes("M1 cover (cover, thickness)"));
  await p.click('.vpage h1 button:has-text("Add a horizontal vessel")'); await p.waitForTimeout(300);
  const nv = await p.evaluate(() => window.__calls.filter(c => c[0] === "apply" && c[1].op.SetVessels).map(c => c[1].op.SetVessels.list).pop());
  check("Add a horizontal vessel adds one with two heads and a shell", Array.isArray(nv) && nv.length === 2 && nv[1].vertical === false && nv[1].parts.length === 3, JSON.stringify(nv?.[1]?.parts?.map(x => x.kind)));
  await p.click('.side button[title="Report"]'); await p.waitForTimeout(400);
  check("the report has the vessel", ((await p.textContent(".page")) ?? "").includes("Vessel V-101"));
  // Saddles: the drum example's saddle stresses and drawing.
  await open("drum");
  await p.click('.side button[title="Vessels"]'); await p.waitForTimeout(300);
  const dt = (await p.textContent(".vpage")) ?? "";
  check("the drum shows its saddle stresses by Zick's method", dt.includes("Saddles") && dt.includes("saddle horn") && dt.includes("Tangential shear"));
  check("the saddle editor shows for a horizontal vessel", dt.includes("On two saddles") && dt.includes("Contact angle"));
  check("the drum passes", ((await p.textContent(".vpage .ctabs")) ?? "").includes("PASS"));
  // Results go out of date after an edit even with Analyze automatically off (a hook was lost before 0.3.1).
  await open("lbend");
  await p.keyboard.press("F5"); await p.waitForTimeout(400);
  await p.click('.side button[title="Vessels"]'); await p.waitForTimeout(200);
  await p.click('.vpage h1 button:has-text("Add a vertical vessel")');
  await p.click('.side button[title="Results"]'); await p.waitForTimeout(60);
  check("an edit marks the results out of date (every edit hook runs)", ((await p.textContent(".page")) ?? "").includes("The model changed since this analysis"));

  check("no errors in the page", errors.length === 0, errors.join(" | "));
  await b.close(); srv.close();
  console.log(failed ? `\n${failed} UI check(s) failed` : "\nAll UI checks passed");
  process.exit(failed ? 1 : 0);
})().catch(e => { console.error(e); process.exit(1); });
