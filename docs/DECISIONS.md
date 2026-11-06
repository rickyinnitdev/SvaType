# Decisions

## M8 — Offline packages and available-machine acceptance

`npm run build` creates both installers, a clean portable ZIP and SHA-256 hashes
under target/packages. The current Microsoft offline WebView2 installer is
214,927,568 bytes, larger than the plan's approximate 130 MB; the Full package
is 220,300,068 bytes. The slim installer remains 2,573,342 bytes. Do not apply the
slim 10 MB budget to the Full installer.

NSIS 3.13 has no Khmer/Cambodian language pack. Rather than invent a partial
language implementation, use its standard English wizard chrome plus bilingual
Khmer/English welcome text and custom messages, generated from the i18n table.
This is the chosen substitute for the requested two installer languages.

The default Tauri uninstaller deletes bundle-ID cache folders only. Our database
uses LocalAppData/SvaType, so a hook removes that directory only when the explicit
Delete my data checkbox is checked and this is not an update. Ordinary uninstall
and every update preserve it. Per-user uninstall never removes shared or portable
class data. A migration test verifies the old-schema backup precedes the upgrade.

Available-machine verification: on Windows 11, the slim installer installs into
an isolated workspace directory; a profile and completed History row survive a
Full reinstall, default uninstall, then slim reinstall. The extracted portable
ZIP starts and creates its database beside the EXE. Hash/resource checks pass;
no learner database is bundled. Native visual inspection confirms bundled Khmer
fonts and the Learning Home render in the installed app.

A licensed clean Windows 10 VM with networking disabled and WebView2 absent is
not available here. Creating one is not possible from the supplied environment.
The chosen development substitute is Windows 11 install/reinstall/uninstall plus
process-scoped external-host blocking, with the exact clean-VM test retained as
an outstanding release gate in TESTING_ON_SCHOOL_PCS.md. It is not recorded as a
passed Windows 10 test. Continue implementation under the plan's instruction to
record impossible environment requirements and proceed.

## M7 — Windows integration and performance measurement

The native Windows APIs and WebView2 bindings already present through Tauri are
now explicit dependencies; this avoids additional plugins for a session-local
named mutex, missing-runtime message and browser lockdown. Native startup errors
use a bilingual table in src/lib/i18n/native.json because they precede the UI.
Fonts are pinned, subset and bundled with OFL notices; no font download occurs
at runtime. Diagnostics collect only the last 200 key latencies and run no timers.

The plan's RAM wording is ambiguous: summing full process working sets double
counts shared WebView2 pages. Use the default Task Manager memory metric, private
working set, for the 150 MB budget and report both metrics. This release measures
103.3 MB private versus 409.7 MB full working-set sum. Do not claim the latter
meets 150 MB. Sampled settled CPU is 0.443%, with no application idle timers/rAF.
Detailed measurements and limits are in docs/PERFORMANCE.md. A cold HDD start
and physical Sandy Bridge machine are unavailable here; warm start and 6x CPU
throttling are the development substitutes, not hardware certification.

All static release budgets pass. Native tests verify <16 ms maximum measured
DOM-update latency under normal/6x throttle, no per-key IPC, no idle callbacks,
font loading and single-instance behavior. Workspace tests, frontend checks,
WASM/i18n tests and offline scans pass.

## M6 — Vocabulary

The eleven categories contain 25 unreviewed words each (275 total), with Khmer
category explanations, English glosses and Khmer examples. Original small SVG
category markers and the specified Windows emoji fallback avoid downloaded art.
Some abstract words use a category symbol; these are teaching aids, not image
recognition tests. No audio is bundled; buttons appear only for recordings that
exist in the local build. Category content loads lazily through the native cache.

All words and examples validate on both layouts. The native smoke test checks
11 categories, the cat card, hidden missing audio, typing, one learned word and
a completed vocabulary History row. Duplicate saves do not inflate learned counts.
435 engine tests and 25 storage/content tests pass.

## M5 — Lesson attempts and curriculum

74 generated, unreviewed lessons cover seven paths and all eight step types.
Migration 003 stores a bookmark plus aggregate summaries for completed steps.
A paused lesson over 20 seconds creates one unfinished History row; finishing
upgrades that same UID transactionally, so time and attempts are not duplicated.
Challenge retries preserve their errors. Stars use accuracy, and one star unlocks
the next lesson. Demo timers run only after Replay and stop on blur or hiding.

Verification: the native isolated smoke test completed all 20 Grade 3 lessons
with three stars and exactly 20 History rows. All 74 lesson targets validate on
both layouts, path references resolve, and draft/partial upgrade tests pass.
435 engine tests, 24 storage/content tests, frontend checks, 12 WASM/i18n tests
and the offline scan pass. WASM remains 195,313 bytes.

## M4 — Typing metrics and text bounds

Per-key errors are attributed to the expected key/layer, while confusion pairs
retain the actual produced text. This makes the weak-key heatmap target the
missing skill rather than the accidental key press. Live metrics use constant-time
counters; per-key/confusion aggregation runs only for results and checkpoints.

Free typing treats every supported output as correct, records Backspace use and
speed, and has live metrics off by default. A page is capped at 20,000 characters
to keep persistence and rendering bounded on school PCs; finish to start another
page. Its viewport shows a bounded tail, while Copy and History retain the full
text. Timed tests repeat their text when needed and preserve aggregate metrics
between rounds. Countdown timers stop when paused or unfocused.

Custom text preparation is Rust-only: normalize line endings and ignore ZWSP,
then validate the selected layout. The source banks are generated data marked
`reviewed: false`. They require a Khmer teacher's review before classroom use.

M4 verification: 435 engine tests, 22 storage/content tests, 12 WASM/i18n tests,
the frontend checks and offline scan pass. Optimized WASM is 195,321 bytes.
`smoke-typing.mjs` verifies all five Typing Mode flows in the native WebView:
free typing/undo/copy (with clipboard mocked), custom text save/reuse, random
practice, analysis heatmap/drill, and a one-minute timed test expired with CDP's
virtual clock. Each completed activity appears in SQLite History. Terminal clock
observation includes trailing gaps up to the pause threshold and excludes longer
pauses, with a hand-computed engine test.

## M3 — Recovery without persisted raw keystrokes

Checkpoints keep the aggregate summary, typed text, and Rust parser position;
they do not store raw keystroke events in SQLite. Exact historical undo cannot
be reconstructed from aggregates (a multi-character key and separate keys may
produce the same text). After a crash, the recovered prefix is kept immutable;
Backspace works for strokes entered after recovery. The recovery screen explains
this boundary. Prior accuracy, errors and per-key aggregates survive, and the
remaining exercise continues under the same idempotent session UID.

The UI checkpoints every 30 seconds only while practice is active, and immediately
when pausing. Timers stop on blur, completion and unmount. Explicit navigation
saves abandoned activities over 20 seconds; shorter abandoned attempts are
discarded. Completed activities are saved regardless of duration.

## M3 — Database extensions and restore

Migration 001 implements the planned schema; additive migration 002 adds stable
session UIDs, full target text, cluster counts, and checkpoints. Backups use the
SQLite online backup API. Restore validates schema and integrity first, keeps a
`pre-restore-<timestamp>.db`, and uses SQLite's online restore on the existing
connection so active WAL files are handled consistently. It never copies over an
open database file. Existing export files are preserved; choose a new filename.

Profile settings use the specified `p:<id>:<name>` keys. Public settings commands
accept an allowlist, so they cannot read or write future teacher PIN secrets.
The test-only data directory override exists only in debug builds.

M3 verification on 2026-11-06: 19 storage tests pass, including an indexed
10,000-session query under 100 ms, transactional rollback, migration preservation,
WAL restart, profile isolation, backup/restore, CSV and checkpoint persistence.
There are now 428 engine tests and 10 WASM/i18n tests. Native UI tests in an
isolated `target/m3-smoke-data` directory created a profile through onboarding,
saved and viewed a completed practice, reloaded the profile picker, changed and
reloaded settings, restored a backup, and verified the CSV BOM. A real 30-second
checkpoint was followed by force-terminating the native process; after restart,
the UI recovered the first three correct strokes, finished with all 48 correct
strokes, saved one result, and removed its checkpoint.

## M2 — Local WASM loading and CSP

PLAN §6.1's `connect-src` omits `'self'`, which blocks its §9 requirement to
stream WASM from a bundled file. Add `'self'` to `connect-src` to permit the local
Tauri asset protocol. External origins remain forbidden; `default-src 'self'`
is unchanged. The generated wasm-bindgen URL loader is removed at build time;
the app supplies a Response from the literal `/engine/khmer_bg.wasm` path.
The build fails if a generated fetch fallback survives.

The development frontend uses Vite's development _mode_ with static bundled
assets, and the Tauri launcher still disables its dev server. This allows the
M2 playground to be development-only without introducing an HTTP/HMR server.
WASM and glue are generated and ignored, with the exact wasm-bindgen CLI version
matching the locked crate. Binaryen is a development-only optimizer.

## M1 — Fingers and metric interpretation

The sample layout in PLAN §4.4 assigns KeyJ to R1 and KeyK to R2, conflicting
with the named finger convention (R1 is thumb). Use R2 index on KeyJ and R3
middle on KeyK, matching the home-row instructions; thumbs use Space. Number
row and ISO extra-key assignments are generated explicitly.

For stop-on-error, repeated mistakes at one position count once in error and
confusion aggregates, while every non-Backspace attempt remains in the accuracy
denominator. This satisfies both the error-deduplication and accuracy definitions.
Flow corrections also retain the original failed attempt in accuracy.

The Windows layout snapshots were extracted locally from KLIDs 00010453 and 00000453. Their `verified` flag means successful Windows extraction; classroom
hardware comparison remains a manual school-PC check.

## M0 — Static development protocol

Tauri's CLI automatically starts a static HTTP server when `devUrl` is absent.
The launcher explicitly passes `--no-dev-server` so the app uses its bundled
`tauri.localhost` protocol instead. The first blocked-network launch exposed this:
the test proxy blocked the generated loopback server, leaving the window hidden.
The build hook waits for Vite, and `build.rs` watches `dist` so frontend changes
invalidate the embedded asset cache. No dev server or HMR is required.

## M0 — Verification status (2026-11-06)

M0 acceptance verified after installing Visual Studio Build Tools 2022 (MSVC
14.44) and Windows SDK 10.0.26100 through the elevated Microsoft installer.
The earlier non-elevated attempt failed with access denied; elevation resolved
that prerequisite. `cargo test --workspace`, `npm run check`, `npm test`,
`npm run test:offline` and Rust formatting pass. Rust test suites are empty in
M0; engine behavior and its tests begin in M1. Four Node tests cover the offline
scanner and security configuration.

`npm run dev` opens the empty native window with `CARGO_NET_OFFLINE=true` and
WebView2 test-only arguments blocking DNS and routing all network requests to
an unavailable proxy (`--host-resolver-rules="MAP * ~NOTFOUND"
--proxy-server=127.0.0.1:9 --proxy-bypass-list="<-loopback>"`). Computer Use
verified the visible empty window and its `http://tauri.localhost/` document.
These test arguments are process-local, not machine settings. This is a
blocked-network launch test, not a claim that the physical adapter was disabled.
The native window is shown after two one-shot animation frames; no idle loop
runs. Debug diagnostics used during investigation are not in the launch command.

Tauri requires an ICO even for native development builds, so M0 includes an
original simple keyboard icon, its SVG source, and PNG. Final installer artwork
and packaging verification remain in M8.

## M0 — Framework namespace identifiers

The offline scanner permits the exact W3C XHTML, SVG and MathML namespace
identifiers used by DOM APIs. These are identifiers, never fetched resources;
removing them breaks framework rendering. All other external URLs still fail.
Svelte diagnostic links are converted to local error codes at build time and
Vite's fetch-based legacy module-preload polyfill is disabled (Chrome 105 already
supports module scripts). Tests cover the scanner's network rejection rules.

## M0 — Clean-room implementation

On 2026-11-06 the GitHub license API for BraedenKilburn/khmer-type returned 404. No license grant could be confirmed. Treat the original as reference only;
copy no code or learning content. All implementation here is clean-room.

## M0 — Development without a network server

`npm run dev` builds the frontend once and launches Tauri using bundled local
assets. There is no Vite HTTP/HMR server or WebSocket and no development CSP
exception. Restart the command after frontend changes. This deliberately trades
hot reload for identical offline asset loading in development and production.

## M0 — Toolchain and scope

The machine initially exposed only a GNU Rust toolchain, outside PATH. Use the
MSVC toolchain and static CRT required by the plan; the npm launcher also resolves
rustup from the user's installation. Never use native CPU or AVX2 flags.
The engine crate is an empty workspace member until M1. WASM begins in M2.
The dialog plugin is local-only and supplies exactly the planned file dialogs.
Full installers, icons, fonts and release WebView2 lockdown remain in their
specified later milestones.
