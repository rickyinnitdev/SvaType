# SvaType

Offline Khmer typing app for Windows 10/11 x64. The implementation follows
[the plan](docs/PLAN.md) in milestone order.

## Development

Install Node.js, Rust (MSVC), Microsoft C++ Build Tools with the Windows SDK,
and WebView2. Dependency downloads are development-only; the app uses local
bundled files in both development and release.

```powershell
npm install
npm run dev
npm run check
npm test
npm run test:offline
cargo test --workspace
```

If Cargo is not in PATH, use `$env:USERPROFILE/.cargo/bin/cargo.exe`.
The dev command builds local frontend assets and opens the empty M0 window;
restart it after frontend edits. No development HTTP server is needed.

`npm run build` runs the Tauri release bundler. The full offline installer and
portable package are scheduled for M8.
