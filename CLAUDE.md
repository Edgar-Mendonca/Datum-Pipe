# Datum Pipe: notes for Claude

Read `docs/MEMORY.md` first. It is the project's memory: architecture, decisions, roadmap, and a log of every session. The family design guide is `FAMILY.md` in `Edgar-Mendonca/Datum` (read-only from here; `docs/FAMILY.md` links to it). The brief this app started from is `briefs/DATUM-PIPE.md` in the same hub repo.
At the end of every session that changes anything, update `docs/MEMORY.md`:
- tick the roadmap
- add a session-log entry
- record any new decisions
- refresh the architecture section if files moved

User preferences:
- Rust and Tauri 2, small and efficient code, Windows first. The same look, ribbon, help, printing and updater as every Datum app (FAMILY §2–9); copy from Datum Plan (and Datum Mesh, the closest app: solver crate, verification, canvas window) rather than reinventing.
- Ship in small numbered releases (0.1.0, 0.1.1, …). Every release appears in the in-app updater.
- Engine logic belongs in `crates/core` (pure Rust, unit-tested), not in the UI. The window never calculates results.
- The UI is SolidJS + Vite in `ui/src/*.tsx` (built to `ui/dist`). State lives in `ui/src/store.ts`. The 3D view uses three.js (MIT).
- The engine and the `.dpipe` file use SI base units (m, kg, N, Pa, K). The engine sends the display units (SI and US) from its unit table (`crates/core/src/units.rs`, copied from Datum Calc); `ui/src/units.ts` only applies them.
- Public words (README, CHANGELOG, site.json, help, commits): never name, compare with, or say the app copies or replaces any commercial piping software (FAMILY §13). Standards may be named by number. Say "calculates using the equations of"; never "certified" or "compliant".
- Codes: implement the published equations only; never reproduce copyrighted tables or text. Allowable stresses are user inputs, pre-filled with typical values and marked "check against your code edition".
- Licences: MIT/BSD/Apache/MPL only. Record every dependency in `NOTICE`.

Quality bar (brief §4):
- Every element and solver path has a test against a known answer; each is a line of Help › Verification (`crates/core/src/bench.rs`, generated from the tests so it is always true for the version running).
- Never cut the benchmarks, the equilibrium check, the code checks or the report.
- Report only checks that actually ran; say what was not verified.

Checks before pushing:
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace` (includes the `hardening` tests and the benchmarks)
- `npm run check:ui`, then `npm run test:ui` (dumps the engine fixture, builds the window, runs the Playwright checks; set `NODE_PATH=$(npm root -g)` if Playwright is installed globally)
- `cargo deny check` (licences, advisories, sources; CI runs it)
- A change you can see: check it in the real app (`cargo build -p datum-pipe && tests_ui/realapp.sh steps.sh shots/`) and add a UI check for every bug

Releasing:
1. Bump the version in `src-tauri/Cargo.toml` and `crates/core/Cargo.toml`.
2. Add a CHANGELOG section (`## x.y.z — date`).
3. Keep `docs/site.json` true with each release (`about`, `steps`, `highlights`, `roadmap`).
4. Run `release.yml` from the Actions tab (workflow_dispatch; tag pushes are blocked from Claude sessions) or push the tag `vX.Y.Z`. The workflow creates the tag and publishes `DatumPipe-x.y.z-setup.exe` and `SHA256SUMS.txt`.
