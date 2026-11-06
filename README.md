# SvaType

Offline Khmer typing app for Windows 10/11 x64. The implementation follows
[the plan](docs/PLAN.md) in milestone order.

## Development

Install Node.js, Rust (MSVC), Microsoft C++ Build Tools with the Windows SDK,
and WebView2. Dependency downloads are development-only; the app uses local
bundled files in both development and release.

Install the WASM build tools once:

```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
```

`npm install` supplies Binaryen for `wasm-opt -Oz`. The frontend build generates
the WASM/glue locally and checks the 200 KB budget. These generated files are
ignored by Git. `npm test` uses the real optimized WASM binary.

```powershell
npm install
npm run dev
npm run check
npm test
npm run test:offline
cargo test --workspace
```

If Cargo is not in PATH, use `$env:USERPROFILE/.cargo/bin/cargo.exe`.
The dev command builds local frontend assets and opens the app;
restart it after frontend edits. No development HTTP server is needed.

`npm run build` produces the slim installer, Full offline installer, portable ZIP
and SHA-256 checksums in `target/packages`. The Full installer embeds WebView2;
the other two packages require an existing runtime. Run `npm run perf` for release
size budgets, and `scripts/check-packages.ps1` to verify packaged resources/hashes.

Learning includes seven paths, 74 lessons, 275 words across eleven categories,
four games and local achievements. Typing includes free typing, timed tests,
custom texts, random drills and mistake analysis. Every finished activity is
saved to local SQLite History. Teacher tools use an optional local Argon2id PIN;
there are no accounts or runtime network services.

All generated learning content is marked unreviewed. Classroom deployment still
requires teacher review, the disconnected clean Windows 10 installation test,
and the physical Sandy Bridge checks listed in
[TESTING_ON_SCHOOL_PCS.md](docs/TESTING_ON_SCHOOL_PCS.md).
Development measurements and their limits are in
[PERFORMANCE.md](docs/PERFORMANCE.md); deviations are in
[DECISIONS.md](docs/DECISIONS.md).
