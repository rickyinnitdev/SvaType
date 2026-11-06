# Performance verification

Reference targets are in PLAN section 9. This development machine is Windows 11,
Intel Core i5-12450H (12 logical processors), 16 GB RAM. It is not the school PC.

Run `npm run perf` to build release and enforce the slim installer, installed
application, initial JavaScript and WASM size budgets. CSS and EXE sizes are
reported as well. Source content is included in installed-size accounting; the
frontend is already embedded in the EXE.

For runtime tests, copy the release EXE and bundled content into an isolated
folder under `target`, add `portable.txt`, and start that copy. Do not use a real
student database. Pass a process-scoped WebView2 debugger port 9223 and external
host blocking arguments as in the integration scripts. This does not disable the
machine's network adapter. Run `node scripts/smoke-performance.mjs`.

The probe forces Low-end mode and tests 200 keys at CPU throttle rates 1 and 6
using the Chromium DevTools protocol. It measures dispatch through DOM update,
not photon latency. It also checks zero per-key IPC, a bounded text viewport,
bundled font loading, and no active interval or animation-frame callbacks on
Home. Throttling is reset before the script closes.

Use `scripts/measure-runtime.ps1 -RootProcessId <pid>` for a 10-second read-only
sample of the complete native/WebView2 process tree. The RAM budget uses summed
private working sets, matching the default Windows Task Manager memory column.
Full working sets are reported too; they count shared pages once per process and
must not be mistaken for unique resident RAM. CPU is
normalized across logical processors. Measure separately without a debugging
port as well, since debugging can add overhead.

Ctrl+Shift+F9 toggles the release-safe overlay: p50/p95 over the last 200 keys,
and JS heap when the browser exposes it. It starts no idle timers. Heap is read
when opening the overlay; unavailable browser data shows a dash.

Fonts are bundled from the pinned upstream commit recorded in
`public/fonts/SOURCES.md`, with complete OFL notices. Both normal and semibold
subsets retain Khmer shaping tables. Native WebView2 configuration disables
browser accelerators, default context menus, status bar, script dialogs,
autofill and saved passwords. Developer tools are disabled in release.

School acceptance still requires measuring the Sandy Bridge PC, checking Khmer
shaping against Notepad, and measuring true cold starts on both SSD and HDD.
CPU throttling on this machine cannot certify those hardware-specific results.

## M7 measurements (2026-11-06)

- Slim installer: 2,588,061 bytes; EXE: 7,186,432 bytes.
- Installed EXE plus content: 7,453,889 bytes.
- Initial JS gzip: 30,525 bytes; WASM: 195,313 bytes; CSS total: 17,005 bytes.
- 200 keys, Low-end mode: normal p50 0.2 / p95 0.4 / max 0.7 ms;
  6x CPU throttle p50 1.6 / p95 3.5 / max 8.6 ms. Zero per-key IPC.
- Home: zero active intervals and animation-frame callbacks.
- Release without debugger, after settling: 103,305,216 bytes private working
  set across 7 processes; full working-set sum 409,690,112 bytes; sampled CPU
  0.443% of the 12-processor machine (rounds to 0% in Task Manager).
- Warm-cache process launch to visible first-painted window: 165 ms. This is
  not a cold-cache HDD measurement.
- A second launch exits within 3 seconds and leaves only one native process.

Raw local reports are under ignored `target/performance-*.json`. Repeat these
measurements after adding screens/content and on the actual school hardware.
