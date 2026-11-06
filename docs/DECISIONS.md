# Decisions

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
