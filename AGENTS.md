# AGENTS.md — rules for Codex (and any coding agent) in this repo

Read `docs/PLAN.md` in full before writing any code. It is the spec. Build it
milestone by milestone, in order, and do not start a milestone until the
previous one meets its acceptance criteria.

## Non-negotiable product rules

1. **No internet, ever, at runtime.** No HTTP clients, no CDN links, no Google
   Fonts, no analytics, no crash reporting, no auto-updater, no remote images.
   The app must behave identically with the network cable unplugged.
   - Do not add `tauri-plugin-http`, `tauri-plugin-updater`, `tauri-plugin-shell`,
     or any crate/npm package whose job is network access.
   - CSP must be `default-src 'self'` (see PLAN §6). Any `http(s)://` URL in
     `src/`, `index.html` or bundled CSS is a bug, except in comments/docs.
2. **No sign-in.** No accounts, passwords, emails, or cloud identity. Users pick a
   local profile by tapping their name. The only secret is an optional local
   teacher PIN (hashed, never stored in plain text).
3. **History is kept.** Every finished practice/lesson/game/test session is saved
   locally in SQLite and is viewable on the History screen. Progress must
   survive restarts, crashes and app updates (use migrations, never drop tables).
4. **Windows 10 and 11, x64, low-end hardware.** Reference machine: Intel Core i3
   2nd gen (Sandy Bridge: has SSE4.2/AVX, **no AVX2**), 8 GB RAM, Intel HD
   2000/3000 graphics, possibly an HDD. Every feature must stay smooth there.
   - Never set `target-cpu=native` or enable AVX2.
   - No CSS `filter`, `backdrop-filter`, large `box-shadow`, blur, or
     full-screen gradients animating. Animate only `transform` and `opacity`.
   - No `requestAnimationFrame` loop or timer running while idle.

## Engineering rules

- Keep dependencies minimal. Before adding a crate or npm package, check whether
  ~50 lines of our own code would do. No UI component libraries, no chart
  libraries, no CSS frameworks, no state-management libraries.
- Khmer processing lives in `crates/khmer_core` (pure Rust, no I/O, no Tauri).
  The UI never re-implements Khmer logic.
- All learning content (lessons, vocabulary, layouts) is data in `content/`,
  never hard-coded in components.
- All user-facing text goes through the i18n table (`src/lib/i18n/`). Khmer is
  the default language, English is secondary.
- Write tests with each feature. `cargo test --workspace` and `npm run check`
  must pass before every commit.
- Commit after each milestone (and after meaningful sub-steps) with a clear
  message. Never commit `target/`, `node_modules/`, `dist/`, or `*.db`.
- If something in the plan is impossible or clearly wrong, write the problem
  and your chosen alternative in `docs/DECISIONS.md`, then continue. Do not
  stop to ask.
- Content you generate (vocabulary, sentences, definitions) must be marked
  `"reviewed": false` so a Khmer teacher can check it later.

## Commands

```bash
npm install
npm run dev          # tauri dev
npm run build        # release build + installers
npm run check        # svelte-check + tsc + eslint
cargo test --workspace
npm run test:offline # see PLAN §11
```
