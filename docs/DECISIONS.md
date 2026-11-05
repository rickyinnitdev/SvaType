# Decisions

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
