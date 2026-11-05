# SvaType — Build Plan (spec for Codex)

**SvaType** (working name; display name configurable in `content/app.json`) is an
offline Khmer typing and learning desktop app for Windows 10/11. It has **two
experiences on one shared engine**:

- **Learning Mode** — teaches the Khmer keyboard from zero: keys, fingers, hand
  position, characters, combined characters, words, sentences. Grade 3/4/5
  paths for primary students, and skill-based paths for everyone else.
- **Typing Mode** — for people who can already type: free typing, timed tests,
  custom text, random practice, mistake analysis, history.

Users choose a mode on first launch and can switch any time from the top bar.

Core principles: offline-first · no sign-in · history kept locally · accuracy
before speed · Khmer language first · runs smoothly on a Core i3 2nd gen.

This document is decisive on purpose. Where it says "do X", do X. If X turns
out to be impossible, record the alternative in `docs/DECISIONS.md` and continue.

---

## 1. Decisions already made

| Topic | Decision | Why |
|---|---|---|
| Shell | **Tauri 2** (Rust) | Uses the system WebView2 instead of shipping Chromium; small installer, low RAM. |
| UI | **Svelte 5 + TypeScript + Vite**, plain CSS | Compiles to small, fast code with no virtual DOM. Lower RAM and CPU than Vue/React for the same UI. |
| Khmer engine | **Rust crate `khmer_core`**, compiled **twice**: natively (used by the Tauri backend and tests) and to **WebAssembly** (used by the UI for each keystroke) | Each keystroke is checked inside the WebView with zero IPC delay, and the logic is still written once, in Rust. |
| Storage | **SQLite** via `rusqlite` (feature `bundled`), WAL mode, migrations | One local file, no server, crash-safe. |
| Key input | Read **`KeyboardEvent.code`** (physical key) and map it through our own layout tables | Works even if the Windows Khmer keyboard is **not installed** and whatever the language indicator shows. No OS IME confusion for children. |
| Default layout | **Khmer NiDA** (Windows "Khmer (NIDA)"). Also ship the legacy Windows "Khmer" layout. Data-driven, so more can be added. | NiDA is the national standard. |
| Fonts | Bundle **Noto Sans Khmer** and **Kantumruy Pro** (both SIL OFL), subset to Khmer + Latin + digits, `woff2` | No CDN, consistent rendering on Win10 machines that lack good Khmer fonts. |
| Charts | Hand-written SVG components | A chart library would cost more than the whole UI. |
| Router/state | Own tiny router (a `$state` screen enum) and Svelte runes stores | No extra libraries. |
| Installer | **NSIS, per-user install (no admin needed)**, two variants (see §10) | Schools often lack admin rights; works from USB. |
| Original repo | Treat [BraedenKilburn/khmer-type](https://github.com/BraedenKilburn/khmer-type) as **reference only**. Do not copy its code or content unless its LICENSE clearly allows it; record the outcome in `docs/DECISIONS.md`. A clean-room re-implementation is the default. | Licence not confirmed. |
| Audio | Optional, P2. No text-to-speech (Windows has no Khmer voice by default). Plan for recorded `.ogg` files that may or may not exist; the UI hides the audio button when a file is missing. | Avoids a hard dependency on recordings. |
| Finger detection | Not possible without a camera. We **teach** fingering visually; we never claim to detect it. | Honesty with users. |

---

## 2. Repository layout

```
SvaType/
├── AGENTS.md
├── docs/
│   ├── PLAN.md                # this file
│   ├── DECISIONS.md           # deviations + reasons (Codex writes this)
│   ├── KHMER_ENGINE.md        # how segmentation/matching works, with examples
│   └── TESTING_ON_SCHOOL_PCS.md
├── Cargo.toml                 # workspace: crates/*, src-tauri
├── crates/
│   ├── khmer_core/            # pure Rust, no I/O, no deps except serde (optional)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── chars.rs       # Khmer Unicode classes (consonant, coeng, vowel, sign…)
│   │   │   ├── cluster.rs     # Khmer cluster (syllable) segmentation
│   │   │   ├── normalize.rs   # canonical order + accepted equivalent orders
│   │   │   ├── layout.rs      # physical key code ↔ output, shift/AltGr layers, fingers
│   │   │   ├── plan.rs        # target text → expected keystroke sequence
│   │   │   ├── session.rs     # live typing state machine (per keystroke)
│   │   │   ├── metrics.rs     # KPM, WPM-equivalent, accuracy, hesitation
│   │   │   └── analysis.rs    # weak keys/clusters, targeted drill generation
│   │   └── tests/             # many table-driven tests (see §4.6)
│   └── khmer_wasm/            # thin wasm-bindgen wrapper around khmer_core
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   └── src/
│       ├── main.rs
│       ├── commands/          # one file per area: profiles, sessions, progress, content, teacher, backup
│       ├── storage/           # db.rs, migrations/*.sql, repo functions
│       ├── content.rs         # loads + validates content/ at startup (lazily per grade)
│       └── paths.rs           # data dir resolution, portable mode
├── src/                       # Svelte UI
│   ├── main.ts
│   ├── App.svelte
│   ├── lib/
│   │   ├── engine.ts          # loads khmer_wasm once, typed wrapper
│   │   ├── api.ts             # typed wrappers for Tauri commands
│   │   ├── router.svelte.ts
│   │   ├── stores/            # profile, settings, session
│   │   ├── i18n/              # km.ts, en.ts
│   │   └── components/        # Keyboard, Hands, TextTrack, StatTile, SvgChart, Button…
│   ├── screens/
│   │   ├── onboarding/        # first launch, mode pick, profile create
│   │   ├── learning/          # Home, PathMap, Lesson, KeyboardIntro, Vocabulary, Games/*
│   │   ├── typing/            # Free, Test, Custom, Random, Analysis
│   │   ├── shared/            # History, SessionResult, Settings, Profiles
│   │   └── teacher/           # PinGate, Students, Class, Assign, ContentEditor, Backup
│   └── styles/                # tokens.css, base.css (no framework)
├── content/
│   ├── app.json
│   ├── layouts/               # nida.json, khmer_legacy.json
│   ├── fingers.json           # finger ids, colours, home keys
│   ├── paths/                 # grade3.json, grade4.json, grade5.json, beginner.json …
│   ├── lessons/               # one JSON per lesson
│   ├── vocabulary/            # one JSON per category
│   ├── sentences/             # practice sentences & paragraphs by level
│   ├── images/vocab/          # SVG, small
│   └── audio/                 # optional .ogg, may be empty
├── public/fonts/              # subset woff2 + OFL.txt
├── scripts/
│   ├── subset-fonts.*         # one-time, dev only
│   ├── validate-content.*     # runs in CI/test: schema + layout coverage
│   └── check-offline.*        # greps the build for http(s):// and forbidden APIs
└── package.json
```

---

## 3. Screens and navigation

Top bar on every screen: **app name · current profile (tap to switch) · mode
switch [Learning | Typing] · History · Settings**. Teacher area is reached from
Settings → "Teacher" (PIN gate).

### 3.1 First launch
1. Language: ខ្មែរ (default) / English.
2. "Who is using this computer?" → **Create profile**: name (Khmer or Latin),
   pick an avatar (12 built-in SVG animals), optional grade (3/4/5/None).
3. "What do you want to do?" → big cards: **Learn to type** (Learning Mode) /
   **Practice typing** (Typing Mode). A short line says you can switch later.
4. Keyboard check: "Press the key with ក on it" → if the physical layout looks
   different (user presses a key that maps elsewhere twice), offer the layout
   picker. Always skippable.

### 3.2 Profile picker (every later launch)
Grid of avatars + names, large tap targets. "Add person" button. Last-used
profile is pre-highlighted; Enter continues. No passwords. Max profiles: no hard
limit; grid scrolls; search box appears above 24 profiles.

### 3.3 Learning Mode
- **Learning Home**: "Continue" (the next lesson, one click), the path map, stars
  earned, today's practice time, a "weak keys" quick drill if any exist.
- **Path map**: chosen path (Grade 3/4/5, or Beginner → Elementary → Intermediate
  → Advanced) as a simple vertical list of units → lessons with 0–3 stars. Locked
  lessons show a lock but **teacher can unlock all** (setting) and the user can
  always replay finished lessons.
- **Keyboard Intro** (Module A + C): home row, left/right hand areas, finger
  colours, row-by-row introduction, "press the highlighted key" drills.
- **Lesson player** (Module B): steps of type `explain`, `demo`, `find_key`,
  `type_chars`, `type_clusters`, `type_words`, `type_sentence`, `challenge`. See §5.
- **Vocabulary** (Module D): categories → word cards (word, picture, optional
  audio, simple Khmer explanation, example sentence) → "Type this word".
- **Games** (Module E, P2): Falling Characters, Match the Picture, Complete the
  Word, Typing Journey. See §7.
- **My Progress**: stars, lessons done, words learned, accuracy trend (SVG line),
  weakest 5 keys, personal bests.

### 3.4 Typing Mode
- **Free Typing**: blank page, on-screen keyboard optional, live KPM/accuracy
  off by default (toggle). Text can be copied out. Nothing is "wrong" here; we
  only record speed and backspace use.
- **Speed Test**: choose duration 1/3/5/10 min and level (easy words / common
  words / sentences / paragraphs). Shows results screen at the end.
- **Custom Practice**: paste or type any Khmer text (up to 20,000 chars), it is
  saved in the profile's "My texts" list for reuse.
- **Random Practice**: generated drills from the word/sentence banks, filters by
  difficulty and by "include my weak keys".
- **Mistake Analysis**: per-key heatmap on the keyboard (colour = error rate),
  most-confused pairs ("you typed ទ when you meant ធ"), slowest keys
  (hesitation), one-click "Drill these".

### 3.5 Shared
- **History** (required): table of all sessions for the current profile —
  date/time, mode, activity name, duration, KPM, accuracy, stars. Filters:
  mode, activity type, date range. Click → **Session Result** detail (mistakes,
  per-key stats, the text). SVG trend chart above the table (last 30 sessions /
  by day). Delete a single session (with confirm). Paginated, 50 rows per page.
- **Session Result**: shown after each session; KPM, WPM-equivalent, accuracy,
  time, mistakes, stars (Learning), "Try again" / "Next".
- **Settings**: language, keyboard layout, show/hide on-screen keyboard, show
  hands, sound effects on/off, font size (S/M/L/XL), **Low-end mode** (on by
  default when hardware detection says ≤4 logical cores — see §9), reduce
  motion, theme (light / dark / high-contrast).

### 3.6 Teacher area (P2, PIN-protected)
- Set/change/remove 4–6 digit PIN (hash with Argon2id, store in `settings`).
  Forgotten PIN: reset by deleting a `reset-pin.txt` file placed next to the
  database — documented in the help screen. No sign-in.
- Students: create/rename/delete profiles, set grade/path, reset progress.
- Class view: table of all profiles — lessons done, average accuracy, time
  practised this week, "needs help" flag (accuracy < 80% over last 5 sessions or
  no activity in 7 days).
- Difficult characters across the class (aggregated key stats).
- Assign: pin a lesson or path for a profile or for all; unlock all lessons.
- Content editor: add/edit vocabulary words and practice sentences (stored in the
  DB as overrides, not by editing bundled files; exported as JSON).
- Backup/Restore/Export: see §8.4.

---

## 4. Khmer engine (`crates/khmer_core`) — the most important part

Write `docs/KHMER_ENGINE.md` alongside the code with worked examples.

### 4.1 Character classes (`chars.rs`)
Classify every code point in U+1780–U+17FF and U+19E0–U+19FF: consonant
(U+1780–U+17A2), independent vowel (U+17A3–U+17B3), dependent vowel
(U+17B6–U+17C5), sign (U+17C6–U+17D1, U+17D3, U+17DD), coeng (U+17D2),
register shifters (U+17C9 MUUSIKATOAN, U+17CA TRIISAP), punctuation (U+17D4–U+17DA,
U+17DC), digits (U+17E0–U+17E9), lek attak (U+17F0–U+17F9), plus ZWSP (U+200B),
ZWNJ, ZWJ, space, and Latin/other. Also mark deprecated/avoid code points
(U+17A3, U+17A4, U+17B4, U+17B5, U+17D3, U+17D8) so content validation can warn.

### 4.2 Cluster segmentation (`cluster.rs`)
Do **not** rely on generic grapheme segmentation; it splits coeng sequences.
Implement a Khmer orthographic cluster parser:

```
cluster := base (COENG consonant){0,2} register? vowel* sign*
base    := consonant | independent_vowel
```
plus single-code-point clusters for digits, punctuation, spaces, ZWSP and anything
non-Khmer. Malformed input never panics: unknown sequences become their own
clusters. Return `Vec<Cluster { start, end }>` as byte/char ranges of the source.

Used for: displaying the text track one cluster per box, highlighting the
current cluster, counting "clusters typed", and per-cluster error stats.

### 4.3 Normalisation and equivalent orders (`normalize.rs`)
Same visual result can come from different typing orders. Rules:
- Compare **per cluster** after canonicalising both target and typed input.
- Canonical order inside a cluster: base → coengs (with **COENG RO (្រ) moved
  last** among coengs, the Unicode-recommended order) → register shifter →
  vowel(s) → signs.
- Accept typing a coeng-ro before another coeng (e.g. ស្រ្ត typed as ស ្រ ្ត or
  ស ្ត ្រ) as correct.
- Multi-part vowels: accept `េ + ា` for `ោ`? **No** — those are different code
  points and render differently in some fonts; treat `ោ` (U+17C4) as one key
  output. Content validation must reject targets using `េា`.
- Keep an explicit table `EQUIVALENTS` of accepted alternatives with a test for
  every entry. Do not guess beyond the table.
- `strict` flag (used only in Learning Mode "find_key" steps) disables
  equivalence.

### 4.4 Layout model (`layout.rs` + `content/layouts/*.json`)
```jsonc
{
  "id": "nida",
  "name": { "km": "នីដា (NiDA)", "en": "Khmer NiDA" },
  "keys": {
    "KeyK": { "base": "ក", "shift": "គ", "altgr": null, "finger": "R2", "row": 2 },
    "KeyJ": { "base": "្", "shift": "ញ", "altgr": null, "finger": "R1", "row": 2 }
    // … every physical key in rows 0–4 (Backquote … Slash) + Space
  }
}
```
- Keys are `KeyboardEvent.code` names. Layers: `base`, `shift`, `altgr`,
  `shift_altgr`. A layer value may be **more than one code point** (some NiDA
  keys output e.g. `ុំ` or `ោះ`); the engine must handle multi-code-point keys.
- **Data source**: generate the layout tables with a dev-only Rust tool
  `scripts/dump-layout` that loads the installed Windows layout (`LoadKeyboardLayoutW`
  with KLID `00000453` for legacy Khmer and the NiDA KLID from the registry
  `HKLM\SYSTEM\CurrentControlSet\Control\Keyboard Layouts`) and calls
  `ToUnicodeEx` for each scan code × modifier. Commit the generated JSON. This
  makes the tables exactly match Windows. If the tool cannot run, hand-write the
  JSON from Microsoft's published layout and mark `"verified": false`; the
  teacher verifies on school PCs (§11).
- `fingers.json`: finger ids `L5 L4 L3 L2 L1 R1 R2 R3 R4 R5` (5 = little,
  1 = thumb… use `L2` = left index), colour per finger (colour-blind-safe set,
  also distinguished by a small pattern/letter so colour is not the only cue),
  home keys `KeyA KeyS KeyD KeyF` / `KeyJ KeyK KeyL Semicolon`, thumbs on Space.
- Reverse map: output string → list of `(code, layer)`; used by the planner.

### 4.5 Keystroke planning and live session (`plan.rs`, `session.rs`)
- `plan(target: &str, layout) -> Vec<PlannedStroke>`: walks the canonical target,
  greedily matching the **longest** layer output at each position, producing
  `{ code, layer, produces, cluster_index, finger }`. Content validation fails if
  any target character cannot be planned on the selected layout.
- `Session` state machine, one call per key event:
  `on_key(code, shift, altgr, timestamp_ms) -> KeyResult { correct, produced,
  expected_next: PlannedStroke, cluster_done, finished }`.
  - Two input policies (per activity): **`stop_on_error`** (default in Learning:
    wrong key = shake + red flash, cursor does not advance, error counted once per
    position) and **`flow`** (default in Typing tests: wrong output is inserted,
    shown red, Backspace removes; uncorrected errors count against accuracy).
  - Backspace, Space, Enter handled explicitly. Ignore modifier-only events,
    auto-repeat (`event.repeat`) and keys with Ctrl/Meta (except AltGr, which on
    Windows arrives as Ctrl+Alt — detect via `getModifierState("AltGraph")`).
  - Hesitation = time from previous correct stroke to this stroke; recorded per
    key.
- Session keeps all per-keystroke data in memory (compact struct, ~16 bytes each)
  and produces a `SessionSummary` at the end (see §4.6).

### 4.6 Metrics (`metrics.rs`) — defined, not borrowed from English
Show these definitions in the help screen and the result screen tooltip.
- **KPM (keystrokes per minute)** = correct keystrokes ÷ active minutes. Primary
  speed metric. Active time excludes pauses > 10 s (configurable).
- **WPM-equivalent** = KPM ÷ 5. Labelled "WPM*" with footnote "5 keystrokes =
  1 word", because Khmer does not separate words with spaces.
- **Accuracy** = correct keystrokes ÷ all non-Backspace keystrokes × 100.
- **Clusters per minute** — shown in detail view.
- **Stars** (Learning): 3★ accuracy ≥ 95%, 2★ ≥ 85%, 1★ ≥ 70%, else retry
  suggested. Speed never affects stars in Grades 3–4; Grade 5 / Advanced lessons
  may have an optional KPM target for a bonus badge.
- `SessionSummary { duration_ms, active_ms, keystrokes, correct, errors,
  backspaces, kpm, accuracy, per_key: Vec<{code, layer, hits, errors, avg_hes_ms}>,
  confusions: Vec<{expected, typed, count}>, clusters_done }`.

### 4.7 Analysis (`analysis.rs`)
- Weak keys: error rate × log(hits) ranking over the last N sessions (data
  fetched from DB, computed in Rust natively).
- Drill generator: given weak keys + word bank + seed, return a practice text
  that over-represents those keys, deterministic for a seed (testable).

### 4.8 Tests (must exist and pass)
- Table-driven tests for segmentation: at least 60 real words including
  ស្ត្រី, ព្រះ, ខ្មែរ, ស្រ្តី, ក្សត្រ, ញ៉ាំ, ប៊ិច, ឪពុក, ម្តាយ/ម្ដាយ (both
  spellings exist — note it), សៀវភៅ, ឆ្កែ, digits ១២៣, punctuation ។ ៕.
- Equivalence tests for every `EQUIVALENTS` entry.
- Planner: every character in all bundled content plans on every bundled layout.
- Session: correct path, wrong key, backspace, multi-code-point key output,
  AltGr, auto-repeat ignored, finish detection.
- Metrics: hand-computed examples.
- Property tests (`proptest`, dev-dependency only): segmentation never panics and
  clusters concatenate back to the input.
- Benchmark (`criterion`, dev-only): `on_key` < 5 µs native; whole-lesson plan for
  2,000 chars < 1 ms.

### 4.9 WASM wrapper (`crates/khmer_wasm`)
- `wasm-bindgen`, `opt-level = "z"`, `lto = true`, run `wasm-opt -Oz`. Target size
  < 200 KB. Expose: `segment`, `Layout::from_json`, `plan`, `Session::new/on_key/
  summary`, `validate_target`.
- Load once at startup (`engine.ts`), keep one instance; never re-instantiate per
  screen.

---

## 5. Content model

All JSON validated by `scripts/validate-content` (run in `cargo test` via a
build-time test in `src-tauri`) against schemas in `content/schema/`.

### 5.1 Paths
```jsonc
// content/paths/grade3.json
{ "id": "grade3", "title": {"km": "ថ្នាក់ទី៣", "en": "Grade 3"},
  "audience": "grade", "units": [
    { "id": "g3u1", "title": {...}, "lessons": ["kb_home_row", "kb_home_left", ...] }
  ] }
```
Paths: `grade3`, `grade4`, `grade5`, `beginner`, `elementary`, `intermediate`,
`advanced`. Lessons can be shared between paths.

Suggested progression (Codex writes the lessons; all `"reviewed": false`):
- **Grade 3 / Beginner**: sitting & hand position, home row both hands, each
  finger's keys row by row, common consonants, dependent vowels, Space, short
  2–3 cluster words, ~20 lessons.
- **Grade 4 / Elementary**: shift layer consonants, subscripts (coeng: ្ + consonant),
  coeng-ro, independent vowels, signs (់ ំ ះ ៈ ៉ ៊), digits ០–៩, common words
  from vocabulary categories, ~22 lessons.
- **Grade 5 / Intermediate**: longer words, ZWSP awareness (explain that Khmer uses
  invisible word breaks; we do not require typing ZWSP — strip U+200B from targets
  by default), punctuation ។ ៕ ? , short sentences, paragraphs, accuracy
  challenges, ~20 lessons.
- **Advanced**: speed-building, mixed paragraphs, numbers & dates, AltGr symbols
  (៛ etc.).

### 5.2 Lesson
```jsonc
{ "id": "kb_home_left", "version": 1, "reviewed": false,
  "title": {"km": "...", "en": "Left hand home row"},
  "focus_keys": ["KeyA","KeyS","KeyD","KeyF"],
  "steps": [
    { "type": "explain", "text": {"km": "...", "en": "..."}, "image": "hands_home.svg" },
    { "type": "demo", "keys": ["KeyF","KeyD","KeyS","KeyA"] },
    { "type": "find_key", "items": ["ថ","ដ","ស","ា"], "strict": true },
    { "type": "type_chars", "text": "ស ដ ថ ា ស ា ដ ា", "policy": "stop_on_error" },
    { "type": "type_words", "items": ["ដា","សា"] },
    { "type": "challenge", "text": "...", "min_accuracy": 85 }
  ] }
```
Lesson text uses **layout-neutral characters**, not key codes, except `demo` /
`focus_keys`. The planner converts characters to keys for the selected layout,
so the same lesson works on NiDA and legacy layouts.

### 5.3 Vocabulary
```jsonc
// content/vocabulary/animals.json
{ "id": "animals", "title": {"km": "សត្វ", "en": "Animals"}, "icon": "cat.svg",
  "words": [
    { "id": "cat", "word": "ឆ្មា", "level": 1, "image": "cat.svg",
      "audio": "cat.ogg", "meaning_km": "...", "example_km": "...",
      "grade": [3,4,5], "reviewed": false }
  ] }
```
Categories for v1: animals, fruits, family, school, nature, body, colours,
numbers, food, home, everyday actions. ~25–40 words each. Only common,
age-appropriate words. Images: simple flat SVGs drawn by us (≤ 4 KB each) or
emoji fallback (Segoe UI Emoji is on Win10/11). No downloaded/copyrighted art.

### 5.4 Sentences
`content/sentences/{level}.json`: arrays of sentences and paragraphs, levels 1–5,
`reviewed: false`, no ZWSP required.

---

## 6. Tauri backend (`src-tauri`)

### 6.1 Config (security + offline)
- `tauri.conf.json`:
  - `app.security.csp`: `"default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; media-src 'self'; connect-src ipc: http://ipc.localhost"`
  - `app.withGlobalTauri: false`; window `visible: false` initially, shown from the
    UI after first paint (avoids white flash); `backgroundColor` set to the theme
    background; min size 1024×640; starts maximised on screens ≤ 1366 px wide.
  - No `updater`, `http`, `shell`, `opener` (except to open the data folder in
    Explorer, implement via our own command using `std::process::Command("explorer")`).
  - `bundle.windows.webviewInstallMode` — see §10.
- `capabilities/default.json`: only `core:default`, `core:window:allow-show`,
  `core:window:allow-set-fullscreen`, `dialog:allow-save`, `dialog:allow-open`
  (for backup/export), and our own commands.
- Release: disable devtools; disable the default context menu and browser
  accelerators. Use `with_webview` to get `ICoreWebView2Settings` and set
  `AreBrowserAcceleratorKeysEnabled = false`, `AreDefaultContextMenusEnabled =
  false`, `IsStatusBarEnabled = false`, `IsZoomControlEnabled = false`. In JS also
  `preventDefault` on F5, Ctrl+R, Ctrl+P, Ctrl+F, Ctrl+U, Ctrl+Shift+I, F12.
- WebView2 additional args (via `additional_browser_args` on the window builder):
  `--disable-features=msSmartScreenProtection,msWebOOUI,msPdfOOUI,ElasticOverscroll --disable-background-networking --disable-component-update --no-first-run`. Verify the app still renders; drop any flag that breaks it and note it in DECISIONS.md.
- Single instance: use `tauri-plugin-single-instance` (local-only); second launch
  focuses the existing window.

### 6.2 Data location and portable mode (`paths.rs`)
- Default: `%LOCALAPPDATA%\SvaType\svatype.db` (+ `backups\`).
- **Portable mode**: if a file `portable.txt` exists next to the exe, use
  `<exe dir>\data\`. Lets a school run the app from a USB stick or a shared folder.
- Optional **shared machine mode**: if `%PROGRAMDATA%\SvaType\shared.txt` exists,
  store the DB in `%PROGRAMDATA%\SvaType\` so all Windows accounts on a lab PC
  see the same student profiles. (Installer for this case documented, not default.)

### 6.3 Database
`rusqlite` with `bundled`. On open: `PRAGMA journal_mode=WAL; synchronous=NORMAL;
foreign_keys=ON; temp_store=MEMORY; cache_size=-4000;` (4 MB). One connection
behind a `Mutex`, commands run on Tauri's async runtime via `spawn_blocking`.

Migrations: numbered SQL files embedded with `include_str!`, tracked in
`schema_version`. Never destructive. Back up the DB file automatically before
running a new migration.

Schema v1:
```sql
CREATE TABLE profiles (
  id INTEGER PRIMARY KEY, name TEXT NOT NULL, avatar TEXT NOT NULL,
  grade INTEGER, path_id TEXT, layout_id TEXT NOT NULL DEFAULT 'nida',
  preferred_mode TEXT NOT NULL DEFAULT 'learning',
  created_at INTEGER NOT NULL, last_used_at INTEGER, archived INTEGER NOT NULL DEFAULT 0);

CREATE TABLE sessions (
  id INTEGER PRIMARY KEY, profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  mode TEXT NOT NULL,            -- learning | typing
  kind TEXT NOT NULL,            -- lesson | vocab | game | free | test | custom | random | drill
  ref_id TEXT,                   -- lesson id, game id, custom text id…
  layout_id TEXT NOT NULL,
  started_at INTEGER NOT NULL, duration_ms INTEGER NOT NULL, active_ms INTEGER NOT NULL,
  keystrokes INTEGER NOT NULL, correct INTEGER NOT NULL, errors INTEGER NOT NULL,
  backspaces INTEGER NOT NULL, kpm REAL NOT NULL, accuracy REAL NOT NULL,
  stars INTEGER, completed INTEGER NOT NULL,
  text_sample TEXT);             -- first 500 chars of target, for history detail
CREATE INDEX idx_sessions_profile_time ON sessions(profile_id, started_at DESC);

CREATE TABLE session_keys (      -- per-key aggregate per session (not per keystroke)
  session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  key_code TEXT NOT NULL, layer TEXT NOT NULL,
  hits INTEGER NOT NULL, errors INTEGER NOT NULL, avg_hes_ms INTEGER NOT NULL,
  PRIMARY KEY (session_id, key_code, layer)) WITHOUT ROWID;

CREATE TABLE session_confusions (
  session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  expected TEXT NOT NULL, typed TEXT NOT NULL, count INTEGER NOT NULL,
  PRIMARY KEY (session_id, expected, typed)) WITHOUT ROWID;

CREATE TABLE lesson_progress (
  profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  lesson_id TEXT NOT NULL, best_stars INTEGER NOT NULL, best_accuracy REAL NOT NULL,
  attempts INTEGER NOT NULL, completed_at INTEGER, bookmark_step INTEGER,
  PRIMARY KEY (profile_id, lesson_id)) WITHOUT ROWID;

CREATE TABLE words_learned (
  profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  word_id TEXT NOT NULL, times_typed INTEGER NOT NULL, best_accuracy REAL NOT NULL,
  first_at INTEGER NOT NULL, PRIMARY KEY (profile_id, word_id)) WITHOUT ROWID;

CREATE TABLE custom_texts (
  id INTEGER PRIMARY KEY, profile_id INTEGER REFERENCES profiles(id) ON DELETE CASCADE,
  title TEXT NOT NULL, body TEXT NOT NULL, created_at INTEGER NOT NULL);

CREATE TABLE achievements (
  profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  achievement_id TEXT NOT NULL, earned_at INTEGER NOT NULL,
  PRIMARY KEY (profile_id, achievement_id)) WITHOUT ROWID;

CREATE TABLE assignments (
  id INTEGER PRIMARY KEY, profile_id INTEGER REFERENCES profiles(id) ON DELETE CASCADE, -- NULL = everyone
  lesson_id TEXT, path_id TEXT, created_at INTEGER NOT NULL);

CREATE TABLE content_overrides (   -- teacher-added words/sentences
  id INTEGER PRIMARY KEY, kind TEXT NOT NULL, category TEXT, json TEXT NOT NULL,
  created_at INTEGER NOT NULL, deleted INTEGER NOT NULL DEFAULT 0);

CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL) WITHOUT ROWID;
                                   -- global; per-profile settings use key 'p:<id>:<name>'
CREATE TABLE schema_version (version INTEGER NOT NULL);
```
Do **not** store every raw keystroke in the DB (aggregates are enough and keep
the DB small: ~1 KB per session). History for 40 students × 3 sessions/day ×
1 year ≈ 40 MB worst case — fine.

### 6.4 Commands (all `async`, return `Result<T, AppError>` serialised as `{code, message}`)
`profiles_list/create/update/archive/delete`, `profile_touch`,
`session_save(summary)`, `session_checkpoint(partial)` (every 30 s during long
tests, so a crash loses at most 30 s), `history_list(profile, filter, page)`,
`history_detail(id)`, `history_delete(id)`, `history_trend(profile, range)`,
`progress_get(profile)`, `lesson_progress_set`, `weak_keys(profile, n)`,
`drill_generate(profile, seed)`, `content_index()`, `content_lesson(id)`,
`content_vocab(category)`, `settings_get/set`, `teacher_pin_set/verify/clear`,
`class_overview()`, `assignments_*`, `backup_create(path)`, `backup_restore(path)`,
`export_csv(path, scope)`, `open_data_folder()`.

Content is read from the bundled resources dir at startup into memory **lazily**:
index first (titles only), lessons/vocab on demand, cached in a `HashMap`.

---

## 7. UI implementation details

### 7.1 Keystroke path (latency budget < 16 ms, target < 4 ms)
`window` `keydown` listener (one, registered by the active typing screen) →
`engine.session.onKey(...)` in WASM → update a few `$state` fields → Svelte
patches only: current cluster box, previous cluster box, highlighted key(s),
finger highlight. No IPC, no DB, no re-render of the full text.
- Text track: render only a window of ~300 clusters around the cursor
  (virtualised); long paragraphs never create thousands of DOM nodes.
- Use `<input>`-less capture; set `inputmode="none"` focus trap on a hidden
  element so the OS IME candidate window never appears; also handle
  `compositionstart` by cancelling.
- Sound effects: preload 3 tiny `.ogg` files into `AudioBuffer`s once; play with
  Web Audio; off by default in Typing Mode.

### 7.2 On-screen keyboard component
- One SVG (or CSS grid of ~60 divs) built once from layout JSON. Each key shows
  the base char large and shift char small; colours per finger (light tint).
- Props: `highlight: {code, layer}`, `heat?: Map<code, number>`, `showFingers`.
  Changing highlight toggles **two** class names, nothing else.
- Shift/AltGr needed → also highlight the correct Shift key (opposite hand).
- Physical keyboard alignment: draw the standard ANSI/ISO row stagger.

### 7.3 Hands component (Module C)
- Two inline SVG hands, each finger a separate `<path>` with the finger colour.
- Active finger: raised opacity + small `translateY` (transform only). "Replay
  demo" animates the finger path from home key → target key → home with CSS
  transitions (~300 ms per step). Disabled in reduced-motion/low-end mode
  (shows arrows instead).

### 7.4 Visual design for children
- Big type: Khmer target text ≥ 40 px for Grade 3 path, 32 px default; UI text
  18 px+. Line height ≥ 1.8 for Khmer (subscripts need room).
- Large buttons (min 48 px tall), clear icons + Khmer labels, max 2–3 choices
  per screen, friendly colour palette, no clutter, no timers on Learning screens
  unless it's a game.
- Encouraging feedback in Khmer ("ពូកែណាស់!" etc., from i18n), never shaming.
- Theme tokens in `styles/tokens.css` using CSS custom properties; light, dark,
  high-contrast.

### 7.5 Games (P2) — all share one `GameLoop` helper
- `GameLoop` uses `requestAnimationFrame` **only while the game is running and
  the window is visible** (`document.visibilityState`, window blur → auto-pause).
  Max 60 fps; in low-end mode cap at 30 fps by skipping frames.
- Sprites are DOM elements moved with `transform: translate3d`; max 12 on screen.
- **Falling Characters**: chars/clusters fall; type to pop. Speed adapts to
  accuracy, not just speed. 3 lives, no harsh fail state.
- **Match the Picture**: show image (+audio), type the word. Hint button reveals
  next key on the keyboard.
- **Complete the Word**: word with 1–2 clusters missing; type the missing part.
- **Typing Journey**: a path of 10 stops; each stop is a short challenge;
  progress is saved per stop.
- Games are saved as sessions with `kind = 'game'` so they appear in History.

### 7.6 i18n
`km.ts` (default) and `en.ts` with the same keys; TypeScript enforces matching
keys. Numbers shown in Khmer digits in Khmer UI (helper `toKhmerDigits`).
Dates formatted with our own helper (do not depend on ICU data for km-KH being
present).

---

## 8. History, progress and data safety

### 8.1 What is recorded
Every finished or abandoned (> 20 s) activity → one `sessions` row + per-key
and confusion aggregates. Lessons also update `lesson_progress`; vocabulary
updates `words_learned`.

### 8.2 History screen
See §3.5. Queries use `idx_sessions_profile_time`; page size 50; trend uses a
single grouped query. Must load in < 100 ms with 10,000 sessions (add a test
that seeds 10,000 rows and times the query).

### 8.3 Achievements (P2)
Data-driven list in `content/achievements.json` (first lesson, 10 lessons,
100 words, 7-day streak, 95% accuracy test…). Evaluated in Rust after each
`session_save`.

### 8.4 Backup, restore, export
- `backup_create`: SQLite online backup API (`rusqlite::backup`) to a user-chosen
  `.svabak` file (it's just a SQLite file). Auto-backup weekly to `backups\`
  keeping the last 4.
- `backup_restore`: validate schema version, keep current DB as
  `pre-restore-<timestamp>.db`, then replace.
- `export_csv`: sessions for one profile or the whole class (UTF-8 with BOM so
  Excel shows Khmer correctly).
- Moving to a new PC: copy the backup file on a USB drive → Restore.

---

## 9. Performance requirements and how to meet them

| Area | Target (validate on the i3 2nd gen) |
|---|---|
| Idle RAM (whole process tree incl. WebView2) | ≤ 150 MB, aim ≤ 120 MB |
| Idle CPU | ~0% (no timers, no rAF, no polling) |
| Keystroke → screen | < 16 ms always, target < 4 ms |
| Cold start → interactive | < 3 s on SSD, < 5 s on HDD |
| Installer (without WebView2) | < 10 MB |
| Installed size | < 30 MB (excluding audio) |

How:
- **Rust release profile** (workspace `Cargo.toml`):
  ```toml
  [profile.release]
  opt-level = 3
  lto = "fat"
  codegen-units = 1
  panic = "abort"
  strip = true
  ```
  Default target `x86_64-pc-windows-msvc`, baseline x86-64 (no AVX2). Static CRT
  (`-C target-feature=+crt-static` in `.cargo/config.toml`) so no VC++
  redistributable is needed.
- **Frontend**: Vite build with `build.target = "chrome105"` (WebView2 is
  Chromium-based and evergreen), no sourcemaps in release, code-split screens
  with dynamic `import()` (teacher area, games, analysis load on demand). JS
  bundle for the first screen < 150 KB gzipped.
- **Fonts**: subset woff2, `font-display: block` for the Khmer font (avoid
  re-layout flash), preload the main one.
- **Low-end mode** (auto-on if `navigator.hardwareConcurrency <= 4` or the user
  picks it; Sandy Bridge i3 = 4 threads, so it will be on at school):
  no transitions except opacity, games capped at 30 fps, hands demo uses arrows,
  no sound preloading until a game starts.
- **Rendering**: Sandy Bridge iGPUs may be GPU-blocklisted by WebView2, meaning
  software compositing. Therefore: no blur/filters/backdrop/big shadows, no
  large animated areas, `contain: layout paint` on the text track and keyboard,
  avoid layout thrash (read then write).
- **Memory**: content loaded lazily; images are tiny SVGs; no global caches of
  sessions; history paginated.
- **Background**: when minimised or unfocused, pause games and stop any timer;
  in Rust, no background threads except the DB worker (idle when unused).
- **Startup**: show window only after first paint; load WASM with
  `WebAssembly.instantiateStreaming` from a local file; DB opens in parallel.
- Add `npm run perf` that builds release and prints: installer size, exe size,
  JS/WASM/CSS sizes, and fails if any budget above is exceeded.
- Add a hidden debug overlay (Ctrl+Shift+F9, release-safe) showing keystroke
  latency (p50/p95 over last 200 keys) and JS heap — for testing on school PCs.

---

## 10. Packaging and offline install (Windows 10/11 x64)

- WebView2: preinstalled on Windows 11 and on updated Windows 10 (since 2021),
  but **not guaranteed** on offline school PCs. Produce **two installers**:
  1. `SvaType-Setup-x64.exe` — `webviewInstallMode: { "type": "skip" }` plus a
     startup check: if WebView2 is missing, show a native Win32 message box (Rust,
     `windows` crate `MessageBoxW`) explaining to run the full installer.
  2. `SvaType-Setup-Full-x64.exe` — `webviewInstallMode: { "type":
     "offlineInstaller" }` (embeds the ~130 MB WebView2 standalone installer). Works
     on a PC that has never been online. This is the one to put on the USB drive.
- NSIS, `installMode: "currentUser"` (no admin), Start-menu + optional desktop
  shortcut, Khmer + English installer languages, uninstaller **keeps** the
  database unless the user ticks "Delete my data".
- Also produce a **portable zip** (exe + resources + `portable.txt`) for PCs that
  already have WebView2.
- Updates: no auto-update. New installer over the old one = upgrade in place;
  migrations run on first start (with auto-backup first). Document "update from
  USB" in `docs/TESTING_ON_SCHOOL_PCS.md`.
- Code signing: optional; document that unsigned installers show a SmartScreen
  warning and how to click through ("More info → Run anyway").
- Icons: `.ico` with 16/24/32/48/256 sizes.

---

## 11. Testing

Automated (run on every commit):
- `cargo test --workspace` (engine, storage with an in-memory DB, migrations
  v1→vN, content validation, 10k-session history query timing).
- `npm run check` (svelte-check, tsc strict, eslint).
- `npm run test` — Vitest for UI logic (router, i18n key parity, digit helpers,
  engine wrapper) with the WASM build.
- `npm run test:offline` — `scripts/check-offline`: fails if the built `dist/`
  or Rust sources contain `http://` or `https://` (allowlist:
  `http://ipc.localhost`, `http://tauri.localhost`, comments), `fetch(` to
  non-relative URLs, `XMLHttpRequest`, `WebSocket`, `navigator.sendBeacon`, or
  any `<link>`/`@import` to external hosts.
- `npm run perf` budget check (§9).

Manual (checklist in `docs/TESTING_ON_SCHOOL_PCS.md`, Codex writes it):
- Install Full installer on a clean Windows 10 (22H2) VM with networking
  **disabled**; also on Windows 11. Run every screen.
- Khmer rendering: subscripts, coeng-ro, ៉ ៊ placement, ុំ, ោះ; compare with
  Word/Notepad.
- Layout verification: with the Windows NiDA keyboard active, type every key in
  Notepad and in the app's "Layout check" screen (Settings → Keyboard → Check):
  the screen shows what the app thinks each key produces next to what Windows
  produced (captured via a hidden `<textarea>` in this one screen only) and
  flags differences. This catches wrong layout tables on real school keyboards.
- Measure RAM (Task Manager, sum of `SvaType.exe` + `msedgewebview2.exe` children)
  idle and during a 10-minute test; CPU idle; latency via debug overlay.
- Multiple profiles on one PC; restart persistence; kill the process mid-test
  (checkpoint restore); backup → restore on another PC; uninstall keeps data.

---

## 12. Milestones (do them in order; commit at the end of each)

Each milestone lists what "done" means. Don't move on until it's done.

**M0 — Repo setup**
`git init`, `.gitignore`, workspace `Cargo.toml`, Tauri 2 + Svelte 5 + TS scaffold,
eslint/prettier, `npm run dev` opens an empty window offline, CSP and capabilities
as in §6.1, `scripts/check-offline` passing. `docs/DECISIONS.md` created, including
the original-repo licence outcome.
✅ Window opens with no network; `check-offline` and `cargo test` pass.

**M1 — Khmer engine core**
`chars`, `cluster`, `normalize`, `layout`, `plan`, `session`, `metrics` with all
tests in §4.8; `dump-layout` tool (or hand-written tables marked unverified);
`nida.json` + `khmer_legacy.json` + `fingers.json`; `docs/KHMER_ENGINE.md`.
✅ ≥ 150 engine tests green; benchmarks within budget.

**M2 — WASM + typing screen skeleton**
`khmer_wasm`, `engine.ts`, Keyboard component, TextTrack component, a dev-only
"playground" screen typing a fixed Khmer paragraph with live highlight, both input
policies, result summary.
✅ Typing a paragraph feels instant; WASM < 200 KB; no IPC per keystroke.

**M3 — Storage, profiles, history**
DB + migrations, profile create/pick/switch, first-launch flow (§3.1), sessions
saved, History screen + detail + trend chart, settings, portable/shared mode,
backup/restore/export CSV, checkpointing.
✅ Restart keeps everything; 10k-session test < 100 ms; kill-mid-test loses ≤ 30 s.

**M4 — Typing Mode (complete)**
Free typing, Speed test (1/3/5/10 min), Custom practice (+ My texts), Random
practice, Mistake analysis with heatmap and "Drill these". Content: sentence banks.
✅ All five Typing Mode screens work and write History.

**M5 — Learning Mode core**
Paths (Grade 3/4/5 + skill paths), path map, lesson player with all step types,
stars, bookmarks (resume at step), Keyboard Intro, Hands component and finger
guidance, lessons content for all paths (`reviewed: false`), My Progress.
✅ A new profile can go from zero through Grade 3 path; progress shows correctly.

**M6 — Vocabulary**
11 categories, word cards, SVG images, optional audio handling, type-the-word
exercises, words-learned tracking.
✅ Every word plans on every layout (validation test); missing audio hides button.

**M7 — Performance & Windows hardening**
Low-end mode, release profile, code splitting, font subsetting, WebView2 settings
& args, accelerator/context-menu lockdown, single instance, WebView2-missing
message, debug latency overlay, `npm run perf` budgets.
✅ All budgets in §9 pass on this dev machine with Low-end mode and with CPU
throttled (document how); idle CPU 0%.

**M8 — Installers**
Both NSIS installers + portable zip, uninstaller keeps data, upgrade path with
migration auto-backup, `docs/TESTING_ON_SCHOOL_PCS.md`.
✅ Full installer works on a Windows 10 VM with network disabled.

**M9 — Games + achievements (P2)**
`GameLoop`, four games, achievements.
✅ Games pause when window hidden; 30 fps cap in low-end mode; sessions in History.

**M10 — Teacher area (P2)**
PIN gate, student management, class overview, needs-help flags, difficult
characters, assignments, content editor (overrides), class CSV export.
✅ Teacher can manage a class of 40 profiles on one PC without touching files.

**M11 — Polish**
High-contrast theme, font-size settings, all strings in Khmer and English, help
screens (metrics definitions, how to install Khmer keyboard in Windows, how to
reset PIN), about screen with font licences (OFL) listed.

---

## 13. Out of scope for v1 (do not build)
Online anything (sync, leaderboards across schools, accounts, updates), camera
finger tracking, TTS, AI-generated content at runtime, mobile/macOS/Linux builds
(the code should not prevent them, but don't spend time on them), printable
worksheets (later), 32-bit builds.

## 14. Open questions for the school (Codex: assume the default, keep going)
| Question | Default assumed |
|---|---|
| Which layout is installed on school PCs? | NiDA; legacy Khmer also available |
| Do students share one Windows account per PC? | Yes → in-app profiles; shared-machine mode available |
| Which textbooks/vocabulary list to align with? | Generic common words, `reviewed: false` until a teacher checks |
| Can we record audio? | Not for v1; audio slots exist |
| Do PCs have WebView2? | Unknown → ship the Full offline installer |
