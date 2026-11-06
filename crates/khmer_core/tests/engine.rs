use khmer_core::{
    analysis::{drill_generate, weak_keys},
    chars::{classify, deprecated, Class},
    cluster::segment,
    layout::{Layer, Layout, PreparedLayout},
    metrics::{self, KeyStats},
    normalize::{canonicalize, equivalent, EQUIVALENTS},
    plan::{plan, validate_target},
    session::{KeyEvent, Policy, Session},
};
use proptest::prelude::*;

fn layout(id: &str) -> PreparedLayout {
    let json = match id {
        "nida" => include_str!("../../../content/layouts/nida.json"),
        _ => include_str!("../../../content/layouts/khmer_legacy.json"),
    };
    serde_json::from_str::<Layout>(json)
        .unwrap()
        .prepare()
        .unwrap()
}
fn type_text(s: &mut Session, text: &str, time: &mut u64) {
    let strokes = plan(text, &s.layout).unwrap();
    for stroke in strokes {
        *time += 100;
        let mut event = KeyEvent::new(&stroke.code, *time);
        event.shift = matches!(stroke.layer, Layer::Shift | Layer::ShiftAltgr);
        event.altgr = matches!(stroke.layer, Layer::Altgr | Layer::ShiftAltgr);
        assert!(s.on_key(event).correct, "{text}: {stroke:?}");
    }
}
macro_rules! words {($($name:ident:[$($cluster:literal),+]),+ $(,)?)=>{$(#[test]fn $name(){let expected=[$($cluster),+];let text=expected.concat();let spans=segment(&text);let actual:Vec<_>=spans.iter().map(|c|&text[c.start..c.end]).collect();assert_eq!(actual,expected);for id in ["nida","khmer_legacy"]{let l=layout(id);let strokes=plan(&text,&l).unwrap();assert_eq!(canonicalize(&text),strokes.iter().map(|s|s.produces.as_str()).collect::<String>());let mut session=Session::new(&text,l,Policy::StopOnError,false).unwrap();type_text(&mut session,&text,&mut 0);assert!(session.finished());}})+};}
words! {
    woman:["ស្ត្រី"],lord:["ព្រះ"],khmer:["ខ្មែ","រ"],woman_ro_first:["ស្រ្តី"],king:["ក្ស","ត្រ"],eat:["ញ៉ាំ"],pen:["ប៊ិ","ច"],father:["ឪ","ពុ","ក"],mother:["ម្តា","យ"],mother_da:["ម្ដា","យ"],book:["សៀ","វ","ភៅ"],dog:["ឆ្កែ"],cat:["ឆ្មា"],bird:["ប","ក្សី"],fish:["ត្រី"],cow:["គោ"],horse:["សេះ"],pig:["ជ្រូ","ក"],chicken:["មា","ន់"],duck:["ទា"],rice:["បា","យ"],water:["ទឹ","ក"],house:["ផ្ទះ"],school:["សា","លា"],student:["សិ","ស្ស"],teacher:["គ្រូ"],flower:["ផ្កា"],tree:["ដើ","ម"],leaf:["ស្លឹ","ក"],sky:["មេ","ឃ"],sun:["ថ្ងៃ"],moon:["ខែ"],star:["ផ្កា","យ"],rain:["ភ្លៀ","ង"],wind:["ខ្យ","ល់"],fire:["ភ្លើ","ង"],soil:["ដី"],mountain:["ភ្នំ"],forest:["ព្រៃ"],road:["ផ្លូ","វ"],car:["ឡា","ន"],boat:["ទូ","ក"],hand:["ដៃ"],leg:["ជើ","ង"],eye:["ភ្នែ","ក"],ear:["ត្រ","ចៀ","ក"],nose:["ច្រ","មុះ"],mouth:["មា","ត់"],head:["ក្បា","ល"],hair:["ស","ក់"],red:["ក្រ","ហ","ម"],green:["បៃ","ត","ង"],white:["ស"],black:["ខ្មៅ"],yellow:["លឿ","ង"],blue:["ខៀ","វ"],one:["មួ","យ"],two:["ពី","រ"],three:["បី"],four:["បួ","ន"],five:["ប្រាំ"],six:["ប្រាំ","មួ","យ"],banana:["ចេ","ក"],mango:["ស្វា","យ"],coconut:["ដូ","ង"],orange:["ក្រូ","ច"],salt:["អំ","បិ","ល"],sugar:["ស្ក","រ"],friend:["មិ","ត្ត"],brother:["ប","ង"],younger:["ប្អូ","ន"],grandmother:["យា","យ"],grandfather:["តា"],digits:["១","២","៣"],punctuation:["។"," ","៕"]
}

#[test]
fn all_unicode_classes() {
    for (start, end, expected) in [
        (0x1780, 0x17a2, Class::Consonant),
        (0x17a3, 0x17b3, Class::IndependentVowel),
        (0x17b6, 0x17c5, Class::DependentVowel),
        (0x17d4, 0x17da, Class::Punctuation),
        (0x17e0, 0x17e9, Class::Digit),
        (0x17f0, 0x17f9, Class::LekAttak),
        (0x19e0, 0x19ff, Class::Lunar),
    ] {
        for cp in start..=end {
            assert_eq!(classify(char::from_u32(cp).unwrap()), expected, "{cp:x}");
        }
    }
}
#[test]
fn shifters_override_sign_range() {
    assert_eq!(classify('៉'), Class::Register);
    assert_eq!(classify('៊'), Class::Register);
    assert_eq!(classify('្'), Class::Coeng);
    for c in ['ំ', 'ះ', 'ៈ', '់', '៌', '៍', '៎', '៏', '័', '៑', '៓', '៝'] {
        assert_eq!(classify(c), Class::Sign);
    }
}
#[test]
fn avoid_codepoints() {
    for cp in [0x17a3, 0x17a4, 0x17b4, 0x17b5, 0x17d3, 0x17d8] {
        assert!(deprecated(char::from_u32(cp).unwrap()));
    }
    assert!(!deprecated('ក'));
}
#[test]
fn whitespace_and_joiners() {
    for (c, class) in [
        (' ', Class::Space),
        ('\n', Class::Space),
        ('\u{200b}', Class::Zwsp),
        ('\u{200c}', Class::Zwnj),
        ('\u{200d}', Class::Zwj),
        ('A', Class::Other),
        ('🙂', Class::Other),
        ('\u{17ff}', Class::Other),
    ] {
        assert_eq!(classify(c), class);
    }
}
#[test]
fn empty_segment() {
    assert!(segment("").is_empty());
}
#[test]
fn malformed_never_swallows_base() {
    let text = "្ក្៉ិ🙂";
    let parts: Vec<_> = segment(text)
        .iter()
        .map(|c| &text[c.start..c.end])
        .collect();
    assert_eq!(parts, ["្", "ក", "្", "៉", "ិ", "🙂"]);
}
#[test]
fn three_coeng_limit() {
    let text = "ក្ ខ".replace(' ', "");
    assert_eq!(segment(&text).len(), 1);
    assert_eq!(segment("ក្គ្ខ្ឃ").len(), 3);
}
#[test]
fn unicode_scalar_ranges() {
    let spans = segment("🙂ខ្មែរ");
    assert_eq!(
        (
            spans[1].start,
            spans[1].end,
            spans[1].char_start,
            spans[1].char_end
        ),
        (4, 16, 1, 5)
    );
}
#[test]
fn strict_keeps_order() {
    assert!(!equivalent("ស្រ្តី", "ស្ត្រី", true));
    assert!(equivalent("ស្រ្តី", "ស្ត្រី", false));
}
#[test]
fn no_vowel_guessing() {
    assert!(!equivalent("កេា", "កោ", false));
    assert!(validate_target("កេា", &layout("nida")).is_err());
}
#[test]
fn no_arbitrary_coeng_swap() {
    assert!(!equivalent("ក្ខ្គ", "ក្គ្ខ", false));
}
#[test]
fn every_equivalence() {
    for &(a, b) in EQUIVALENTS {
        assert!(
            equivalent(&format!("ស{a}ី"), &format!("ស{b}ី"), false),
            "{a} -> {b}"
        );
        assert!(!equivalent(a, b, true));
    }
}
#[test]
fn normalize_idempotent() {
    assert_eq!(canonicalize(&canonicalize("ស្រ្តី ក្រ្ខ")), canonicalize("ស្រ្តី ក្រ្ខ"));
}
#[test]
fn planner_longest() {
    for id in ["nida", "khmer_legacy"] {
        let p = plan("កុំ", &layout(id)).unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!(p[1].produces, "ុំ");
    }
}
#[test]
fn planner_preserves_newline() {
    let p = plan("ក\nខ", &layout("nida")).unwrap();
    assert_eq!(p[1].code, "Enter");
    assert_eq!(p[1].cluster_index, 1);
}
#[test]
fn unknown_target_error() {
    let e = plan("ក🙂", &layout("nida")).unwrap_err();
    assert_eq!(e.offset, 3);
    assert_eq!(e.character, "🙂");
}
#[test]
fn layout_rejects_invalid_finger() {
    let mut l = layout("nida").layout;
    l.keys.get_mut("KeyK").unwrap().finger = "camera".into();
    assert!(l.prepare().is_err());
}
#[test]
fn layout_reverse_duplicates() {
    let l = layout("nida");
    assert!(!l.reverse("ក").is_empty());
    assert!(l.reverse("🙂").is_empty());
}
#[test]
fn correct_path() {
    let mut s = Session::new("កខ", layout("nida"), Policy::StopOnError, false).unwrap();
    type_text(&mut s, "កខ", &mut 0);
    assert!(s.finished());
    assert_eq!(s.typed(), "កខ");
    assert_eq!(s.summary().correct, 2);
}
#[test]
fn stop_on_error_does_not_advance() {
    let mut s = Session::new("ក", layout("nida"), Policy::StopOnError, false).unwrap();
    assert!(!s.on_key(KeyEvent::new("KeyA", 0)).correct);
    assert!(!s.on_key(KeyEvent::new("KeyA", 10)).correct);
    assert_eq!(s.cluster_index(), 0);
    assert_eq!(s.summary().errors, 1);
    assert_eq!(s.summary().keystrokes, 2);
    assert!(s.typed().is_empty());
}
#[test]
fn backspace_undoes_entire_multichar_key() {
    let mut s = Session::new("កុំខ", layout("nida"), Policy::Flow, false).unwrap();
    type_text(&mut s, "កុំ", &mut 0);
    s.on_key(KeyEvent::new("Backspace", 300));
    assert_eq!(s.typed(), "ក");
    assert_eq!(s.expected_next().unwrap().produces, "ុំ");
}
#[test]
fn flow_records_and_corrects_error() {
    let mut s = Session::new("កខ", layout("nida"), Policy::Flow, false).unwrap();
    let r = s.on_key(KeyEvent::new("KeyA", 0));
    assert!(!r.correct);
    assert_eq!(s.cluster_index(), 1);
    assert_eq!(s.typed(), "ា");
    s.on_key(KeyEvent::new("Backspace", 100));
    type_text(&mut s, "កខ", &mut 100);
    assert!(s.finished());
    assert_eq!(s.typed(), "កខ");
    assert_eq!(s.summary().backspaces, 1);
    assert!((s.summary().accuracy - 200.0 / 3.0).abs() < 1e-8);
}
#[test]
fn altgr_ctrl_alt_is_allowed() {
    let l = layout("nida");
    let output = l
        .outputs
        .iter()
        .find(|o| o.layer == Layer::Altgr)
        .unwrap()
        .clone();
    let mut s = Session::new(&output.produces, l, Policy::StopOnError, false).unwrap();
    let mut event = KeyEvent::new(&output.code, 0);
    event.ctrl = true;
    event.altgr = true;
    assert!(s.on_key(event).correct);
    assert!(s.finished());
}
#[test]
fn repeat_is_ignored() {
    let mut s = Session::new("កក", layout("nida"), Policy::Flow, false).unwrap();
    let mut e = KeyEvent::new("KeyK", 0);
    e.repeat = true;
    assert!(s.on_key(e).ignored);
    assert_eq!(s.summary().keystrokes, 0);
}
#[test]
fn modifier_only_is_ignored() {
    for code in [
        "ShiftLeft",
        "ShiftRight",
        "AltLeft",
        "AltRight",
        "ControlLeft",
        "ControlRight",
        "MetaLeft",
        "CapsLock",
        "Tab",
        "F1",
    ] {
        let mut s = Session::new("ក", layout("nida"), Policy::Flow, false).unwrap();
        assert!(s.on_key(KeyEvent::new(code, 0)).ignored, "{code}");
    }
}
#[test]
fn shortcuts_are_ignored() {
    for meta in [false, true] {
        let mut s = Session::new("ក", layout("nida"), Policy::Flow, false).unwrap();
        let mut e = KeyEvent::new("KeyK", 0);
        e.ctrl = !meta;
        e.meta = meta;
        assert!(s.on_key(e).ignored);
    }
}
#[test]
fn space_enter() {
    let mut s = Session::new("ក \nខ", layout("nida"), Policy::Flow, false).unwrap();
    type_text(&mut s, "ក \nខ", &mut 0);
    assert!(s.finished());
}
#[test]
fn empty_session_finished() {
    assert!(Session::new("", layout("nida"), Policy::Flow, false)
        .unwrap()
        .finished());
}
#[test]
fn finish_ignores_more_keys() {
    let mut s = Session::new("ក", layout("nida"), Policy::Flow, false).unwrap();
    type_text(&mut s, "ក", &mut 0);
    assert!(s.on_key(KeyEvent::new("KeyK", 300)).ignored);
    assert_eq!(s.summary().keystrokes, 1);
}
#[test]
fn finish_can_backspace() {
    let mut s = Session::new("ក", layout("nida"), Policy::Flow, false).unwrap();
    type_text(&mut s, "ក", &mut 0);
    s.on_key(KeyEvent::new("Backspace", 200));
    assert!(!s.finished());
    assert_eq!(s.cluster_index(), 0);
}
#[test]
fn backward_clock_ignored() {
    let mut s = Session::new("កក", layout("nida"), Policy::Flow, false).unwrap();
    s.on_key(KeyEvent::new("KeyK", 200));
    assert!(s.on_key(KeyEvent::new("KeyK", 100)).ignored);
}
#[test]
fn no_losing_wrong_key_on_backspace() {
    let mut s = Session::new("កខ", layout("nida"), Policy::StopOnError, false).unwrap();
    type_text(&mut s, "ក", &mut 0);
    s.on_key(KeyEvent::new("KeyA", 200));
    s.on_key(KeyEvent::new("Backspace", 300));
    assert_eq!(s.cluster_index(), 0);
}
#[test]
fn restart_clears_everything() {
    let mut s = Session::new("ក", layout("nida"), Policy::Flow, false).unwrap();
    type_text(&mut s, "ក", &mut 0);
    s.restart();
    assert_eq!(s.summary().keystrokes, 0);
    assert!(!s.finished());
    assert!(s.typed().is_empty());
}
#[test]
fn live_equivalent_order() {
    let l = layout("nida");
    let mut s = Session::new("ស្ត្រី", l.clone(), Policy::StopOnError, false).unwrap();
    for (i, c) in "ស្រ្តី".chars().enumerate() {
        let stroke = l.reverse(&c.to_string())[0];
        let mut e = KeyEvent::new(&stroke.code, i as u64 * 100);
        e.shift = matches!(stroke.layer, Layer::Shift | Layer::ShiftAltgr);
        e.altgr = matches!(stroke.layer, Layer::Altgr | Layer::ShiftAltgr);
        assert!(s.on_key(e).correct, "{c}");
    }
    assert!(s.finished());
}
#[test]
fn live_strict_rejects_equivalent_order() {
    let mut s = Session::new("ស្ត្រី", layout("nida"), Policy::StopOnError, true).unwrap();
    type_text(&mut s, "ស្", &mut 0);
    let stroke = s.layout.reverse("រ")[0].clone();
    assert!(!s.on_key(KeyEvent::new(&stroke.code, 400)).correct);
}
#[test]
fn metrics_hand_computed() {
    assert_eq!(metrics::rate(120, 60_000), 120.0);
    assert_eq!(metrics::accuracy(95, 100), 95.0);
    assert_eq!(metrics::rate(1, 0), 0.0);
    assert_eq!(metrics::accuracy(0, 0), 100.0);
}
#[test]
fn stars_boundaries() {
    for (accuracy, expected) in [
        (0.0, 0),
        (69.99, 0),
        (70.0, 1),
        (84.99, 1),
        (85.0, 2),
        (94.99, 2),
        (95.0, 3),
        (100.0, 3),
    ] {
        assert_eq!(metrics::stars(accuracy), expected);
    }
}
#[test]
fn pauses_excluded_and_hesitation_retained() {
    let mut s = Session::new("កកកក", layout("nida"), Policy::Flow, false).unwrap();
    for time in [100, 1100, 20100, 21100] {
        s.on_key(KeyEvent::new("KeyK", time));
    }
    let stats = s.summary();
    assert_eq!(stats.duration_ms, 21000);
    assert_eq!(stats.active_ms, 2000);
    assert_eq!(stats.kpm, 120.0);
    assert_eq!(stats.wpm_equivalent, 24.0);
    assert_eq!(stats.per_key[0].avg_hes_ms, 5250.0);
}
#[test]
fn configured_pause_threshold() {
    let mut s = Session::new("កក", layout("nida"), Policy::Flow, false).unwrap();
    s.set_pause_threshold(500);
    s.on_key(KeyEvent::new("KeyK", 0));
    s.on_key(KeyEvent::new("KeyK", 1000));
    assert_eq!(s.summary().active_ms, 0);
}
#[test]
fn weak_keys_aggregate_layers() {
    let stats = [
        KeyStats {
            code: "KeyK".into(),
            hits: 8,
            errors: 2,
            ..Default::default()
        },
        KeyStats {
            code: "KeyK".into(),
            hits: 0,
            errors: 10,
            layer: Layer::Shift,
            ..Default::default()
        },
        KeyStats {
            code: "KeyA".into(),
            hits: 100,
            errors: 0,
            ..Default::default()
        },
    ];
    let weak = weak_keys(&stats, 5);
    assert_eq!(weak.len(), 1);
    assert_eq!(weak[0].code, "KeyK");
}
#[test]
fn drills_are_seeded_and_weighted() {
    let bank = vec![
        ("ក".into(), vec!["KeyK".into()]),
        ("ា".into(), vec!["KeyA".into()]),
    ];
    let a = drill_generate(&bank, &["KeyK".into()], 42, 1000);
    assert_eq!(a, drill_generate(&bank, &["KeyK".into()], 42, 1000));
    assert_ne!(a, drill_generate(&bank, &["KeyK".into()], 43, 1000));
    assert!(a.split(' ').filter(|s| *s == "ក").count() > 700);
    assert_eq!(drill_generate(&[], &[], 0, 100), "");
}
#[test]
fn rejects_long_target() {
    assert!(Session::new(&"ក".repeat(20_001), layout("nida"), Policy::Flow, false).is_err());
}

proptest! {
    #[test]fn arbitrary_edits_never_panic(events in prop::collection::vec(0usize..8,0..300)) {
        let mut s=Session::new("ស្ត្រី កុំ ខ្មែរ",layout("nida"),Policy::Flow,false).unwrap();
        let codes=["KeyK","KeyA","Comma","KeyJ","KeyR","Backspace","Space","KeyS"];
        for (i,key) in events.iter().enumerate(){s.on_key(KeyEvent::new(codes[*key],i as u64*100));}
        let summary=s.summary();prop_assert!(summary.correct<=summary.keystrokes);prop_assert!((0.0..=100.0).contains(&summary.accuracy));
    }
    #[test]fn arbitrary_text_roundtrips(text in ".{0,512}"){let spans=segment(&text);prop_assert_eq!(spans.iter().map(|c|&text[c.start..c.end]).collect::<String>(),text);}
    #[test]fn arbitrary_khmer_roundtrips(points in prop::collection::vec(0x1780u32..0x1800,0..300)){let text:String=points.into_iter().map(|cp|char::from_u32(cp).unwrap()).collect();let spans=segment(&text);prop_assert_eq!(spans.iter().map(|c|&text[c.start..c.end]).collect::<String>(),text);}
    #[test]fn canonicalization_is_idempotent(text in ".{0,512}"){prop_assert_eq!(canonicalize(&canonicalize(&text)),canonicalize(&text));}
}

// Generated from Windows layout snapshots: each physical layer must complete
// its exact output in one input event, including standalone multi-part outputs.
macro_rules! output_case {
    ($name:ident,$id:literal,$code:literal,$layer:ident,$text:literal) => {
        #[test]
        fn $name() {
            let mut s = Session::new($text, layout($id), Policy::Flow, false).unwrap();
            let layer = Layer::$layer;
            let mut e = KeyEvent::new($code, 100);
            e.shift = matches!(layer, Layer::Shift | Layer::ShiftAltgr);
            e.altgr = matches!(layer, Layer::Altgr | Layer::ShiftAltgr);
            let result = s.on_key(e);
            assert!(result.correct, "{result:?}");
            assert!(s.finished());
            assert_eq!(s.typed(), $text);
        }
    };
}
include!("fixtures/layout_cases.rs");
