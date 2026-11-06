# Khmer engine

`khmer_core` is pure Rust. Its only runtime dependency is Serde for shared data
types. It does not open files, read clocks, access Windows or call Tauri. Callers
provide layout data and monotonic millisecond timestamps.

## Segmentation and matching

The parser recognizes a consonant or independent vowel followed by up to two
coeng/consonant pairs, an optional register shifter, dependent vowels and signs.
Other scalars (including malformed marks, emoji and joiners) remain individual
clusters. Ranges contain both UTF-8 byte offsets and Unicode scalar offsets;
JavaScript must use the returned cluster strings, not treat byte offsets as
UTF-16 indexes.

| Input         | Clusters            |
| ------------- | ------------------- |
| ខ្មែរ         | ខ្មែ · រ            |
| ស្ត្រី        | ស្ត្រី              |
| ស្រ្តី        | ស្រ្តី              |
| ព្រះ          | ព្រះ                |
| ញ៉ាំ          | ញ៉ាំ                |
| ប៊ិច          | ប៊ិ · ច             |
| សៀវភៅ         | សៀ · វ · ភៅ         |
| ម្តាយ / ម្ដាយ | ម្តា · យ / ម្ដា · យ |

The two mother spellings use different consonants and remain distinct. Both are
valid lesson candidates. Test vocabulary is not teacher-reviewed learning content.

Only the explicit `EQUIVALENTS` table permits alternate ordering: coeng-ro may
precede the other coeng, and the canonical form moves it last. The table covers
all other Khmer consonants, with a test for each entry. Strict mode keeps the
literal order. It never equates `េា` with `ោ`; planning rejects decomposed
targets. Valid parsed clusters are already in register/vowel/sign order, and
malformed input is preserved rather than silently rewritten.

## Layouts and planning

Run `cargo run -p dump-layout` from the repository root to regenerate the two
Windows snapshots. The tool finds Microsoft's NiDA DLL in the registry, loads
NiDA and legacy Khmer, maps physical scan codes to virtual keys, and invokes
`ToUnicodeEx` with each modifier state. Flag 4 avoids modifying keyboard state.
Control-character outputs are absent, not text input. Enter is handled explicitly.
No active keyboard layout is intentionally selected. The dumped snapshots are
verified against this development PC, not yet against school keyboards.

`Layout::prepare` builds a longest-output-first index. Planning greedily consumes
the canonical target and returns physical code, layer, produced text, cluster
index and finger. Base-layer ties win over Shift and AltGr. For example, NiDA
`កុំ` uses `KeyK` and `Comma`, with `Comma` producing two scalars `ុ` + `ំ`.
The engine accepts the same text typed using separate keys too. ISO
`IntlBackslash` is included in addition to the ANSI rows.

## Live input

The caller passes `KeyEvent`: physical code, Shift, AltGraph, Ctrl, Meta, repeat
and timestamp. AltGraph must come from `getModifierState('AltGraph')` so Windows
Ctrl+Alt can be distinguished from shortcuts. Modifiers, navigation/function keys,
repeat and Ctrl/Meta shortcuts do not affect metrics. Backspace undoes one whole
physical event, including a multi-scalar output, and restores alternate-order
matching state. Input is limited to 20,000 scalars, matching Custom Practice.

Stop-on-error retains the cursor and counts one error aggregate per position;
all attempts still enter the accuracy denominator. Flow inserts the produced
output and consumes the expected stroke on an error. Backspace can correct it,
but the original attempt remains in accuracy and confusion aggregates. A
session finishes when its target positions have been consumed. Empty sessions
are finished with zero speed and 100% accuracy. Additional keys after finish are
ignored except Backspace.

The hot path returns a small change with `produced`, correctness, cluster index
and the next expected stroke. `typed()` reconstructs full text only on request;
the UI must not request it on every key for long texts. Raw event records are
16 bytes, with a separate undo-index stack. Summary aggregation happens on demand,
not on each key. Neither records nor undo history are persisted to SQLite.

## Metrics

The first accepted input event starts the clock. Inter-event gaps over 10 seconds
(configurable) are excluded entirely from active time; duration retains them.
Hesitation measures time since the preceding correct stroke, including pauses;
the first correct stroke has zero hesitation. Backspace consumes elapsed time
but is excluded from accuracy's denominator. Correct hits include subsequent
corrections, so accuracy is correct attempts / all non-Backspace attempts.

KPM is correct hits × 60,000 / active milliseconds; WPM* is KPM / 5. Clusters per
minute uses consumed target clusters. Zero active time gives zero rates. Stars
use accuracy only at 70%, 85% and 95%. Weak-key ranking aggregates layers before
ranking error rate × log(1 + attempts). Drill sampling weights weak-key words
and is deterministic for a seed.

## Verification

M1 verification on 2026-11-06: 426 engine tests pass. Criterion (20 samples,
1-second warmup, 2-second measurement) reports planning at 349–365 µs and
`on_key` at 0.65–1.10 µs, both below budget. These ranges include scheduler noise.

`cargo test --workspace` runs table cases for real words, all extracted layout
outputs, equivalence, edits, input filtering, metric examples and property tests
over arbitrary Unicode and Khmer text. `cargo bench -p khmer_core --bench engine`
measures planning 2,000 scalars and the complete `on_key` call. Budgets are <1 ms
for planning and <5 µs per key on this development machine. School-PC measurements
remain separate from these native development benchmarks.
