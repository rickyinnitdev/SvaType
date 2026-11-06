use super::{
    backup,
    db::{self, Database},
    profiles::{self, ProfileInput},
    sessions::{self, Checkpoint, HistoryFilter, SessionInput},
};
use khmer_core::{
    layout::Layer,
    metrics::{Confusion, KeyStats, SessionSummary},
};
use rusqlite::Connection;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "svatype-test-{}-{}-{}",
            std::process::id(),
            db::now(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        if self.0.starts_with(std::env::temp_dir())
            && self
                .0
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("svatype-test-")
        {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}
fn memory() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    db::configure(&conn).unwrap();
    db::migrate(&mut conn).unwrap();
    conn
}
fn input(name: &str) -> ProfileInput {
    ProfileInput {
        name: name.into(),
        avatar: "0".into(),
        grade: Some(3),
        path_id: Some("grade3".into()),
        layout_id: "nida".into(),
        preferred_mode: "learning".into(),
    }
}
fn sample(profile_id: i64, uid: &str) -> SessionInput {
    SessionInput {
        uid: uid.into(),
        profile_id,
        mode: "learning".into(),
        kind: "lesson".into(),
        ref_id: Some("test_lesson".into()),
        layout_id: "nida".into(),
        started_at: 1000,
        completed: true,
        target_text: "ខ្មែរ".into(),
        bookmark_step: None,
        summary: SessionSummary {
            duration_ms: 60_000,
            active_ms: 60_000,
            keystrokes: 100,
            correct: 95,
            errors: 5,
            kpm: 95.0,
            accuracy: 95.0,
            stars: 3,
            clusters_done: 2,
            per_key: vec![KeyStats {
                code: "KeyK".into(),
                layer: Layer::Base,
                hits: 95,
                errors: 5,
                avg_hes_ms: 120.0,
            }],
            confusions: vec![Confusion {
                expected: "ក".into(),
                typed: "ខ".into(),
                count: 5,
            }],
            ..Default::default()
        },
    }
}
#[test]
fn migrations_are_idempotent() {
    let mut c = memory();
    db::migrate(&mut c).unwrap();
    assert_eq!(db::version(&c).unwrap(), db::VERSION);
}
#[test]
fn migration_preserves_v1_profiles_and_sessions() {
    let mut c = Connection::open_in_memory().unwrap();
    c.execute_batch(db::MIGRATIONS[0]).unwrap();
    let p = profiles::create(&c, input("សុខា")).unwrap();
    c.execute("INSERT INTO sessions(profile_id,mode,kind,layout_id,started_at,duration_ms,active_ms,keystrokes,correct,errors,backspaces,kpm,accuracy,completed) VALUES(?1,'typing','test','nida',1,100,100,1,1,0,0,600,100,1)",[p.id]).unwrap();
    db::migrate(&mut c).unwrap();
    assert_eq!(profiles::list(&c).unwrap()[0].name, "សុខា");
    assert_eq!(
        sessions::history(&c, p.id, &HistoryFilter::default(), 0)
            .unwrap()
            .total,
        1
    );
}
#[test]
fn rejects_future_schema() {
    let mut c = memory();
    c.execute("UPDATE schema_version SET version=999", [])
        .unwrap();
    assert!(db::migrate(&mut c).is_err());
}
#[test]
fn profile_validation() {
    let c = memory();
    assert!(profiles::create(&c, input(" ")).is_err());
    let mut bad = input("name");
    bad.grade = Some(2);
    assert!(profiles::create(&c, bad).is_err());
    let mut bad = input("name");
    bad.avatar = "12".into();
    assert!(profiles::create(&c, bad).is_err());
}
#[test]
fn profile_update_archive_touch() {
    let c = memory();
    let p = profiles::create(&c, input("name")).unwrap();
    profiles::update(&c, p.id, input("ថ្មី")).unwrap();
    profiles::touch(&c, p.id).unwrap();
    assert_eq!(profiles::list(&c).unwrap()[0].name, "ថ្មី");
    profiles::archive(&c, p.id).unwrap();
    assert!(profiles::list(&c).unwrap().is_empty());
}
#[test]
fn saves_detail_and_progress_atomically() {
    let mut c = memory();
    let p = profiles::create(&c, input("name")).unwrap();
    let s = sample(p.id, "one");
    let id = sessions::save(&mut c, &s).unwrap();
    let d = sessions::detail(&c, id).unwrap();
    assert_eq!(d.target_text, "ខ្មែរ");
    assert_eq!(d.summary.per_key[0].hits, 95);
    assert_eq!(d.summary.confusions[0].count, 5);
    assert_eq!(d.summary.clusters_done, 2);
    let stars: u8 = c
        .query_row("SELECT best_stars FROM lesson_progress", [], |r| r.get(0))
        .unwrap();
    assert_eq!(stars, 3);
}
#[test]
fn duplicate_save_is_idempotent() {
    let mut c = memory();
    let p = profiles::create(&c, input("name")).unwrap();
    let s = sample(p.id, "one");
    assert_eq!(
        sessions::save(&mut c, &s).unwrap(),
        sessions::save(&mut c, &s).unwrap()
    );
    assert_eq!(
        sessions::history(&c, p.id, &HistoryFilter::default(), 0)
            .unwrap()
            .total,
        1
    );
    let attempts: u32 = c
        .query_row("SELECT attempts FROM lesson_progress", [], |r| r.get(0))
        .unwrap();
    assert_eq!(attempts, 1);
}
#[test]
fn rollback_on_duplicate_key_aggregate() {
    let mut c = memory();
    let p = profiles::create(&c, input("name")).unwrap();
    let mut s = sample(p.id, "one");
    s.summary.per_key.push(s.summary.per_key[0].clone());
    assert!(sessions::save(&mut c, &s).is_err());
    assert_eq!(
        sessions::history(&c, p.id, &HistoryFilter::default(), 0)
            .unwrap()
            .total,
        0
    );
}
#[test]
fn delete_cascades_aggregates() {
    let mut c = memory();
    let p = profiles::create(&c, input("name")).unwrap();
    let id = sessions::save(&mut c, &sample(p.id, "one")).unwrap();
    sessions::delete(&c, id).unwrap();
    let count: i64 = c
        .query_row("SELECT COUNT(*) FROM session_keys", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}
#[test]
fn profiles_are_isolated() {
    let mut c = memory();
    let a = profiles::create(&c, input("A")).unwrap();
    let b = profiles::create(&c, input("B")).unwrap();
    sessions::save(&mut c, &sample(a.id, "one")).unwrap();
    assert_eq!(
        sessions::history(&c, b.id, &HistoryFilter::default(), 0)
            .unwrap()
            .total,
        0
    );
}
#[test]
fn filters_and_pagination() {
    let mut c = memory();
    let p = profiles::create(&c, input("A")).unwrap();
    for i in 0..61 {
        let mut s = sample(p.id, &format!("s{i}"));
        s.started_at = i;
        s.mode = if i % 2 == 0 { "typing" } else { "learning" }.into();
        sessions::save(&mut c, &s).unwrap();
    }
    assert_eq!(
        sessions::history(&c, p.id, &HistoryFilter::default(), 0)
            .unwrap()
            .rows
            .len(),
        50
    );
    assert_eq!(
        sessions::history(&c, p.id, &HistoryFilter::default(), 1)
            .unwrap()
            .rows
            .len(),
        11
    );
    let f = HistoryFilter {
        mode: Some("typing".into()),
        from: Some(10),
        to: Some(20),
        ..Default::default()
    };
    assert_eq!(sessions::history(&c, p.id, &f, 0).unwrap().total, 6);
}
#[test]
fn history_10000_under_100ms() {
    let mut c = memory();
    let p = profiles::create(&c, input("A")).unwrap();
    let tx = c.transaction().unwrap();
    for i in 0..10_000 {
        tx.execute("INSERT INTO sessions(profile_id,mode,kind,layout_id,started_at,duration_ms,active_ms,keystrokes,correct,errors,backspaces,kpm,accuracy,completed) VALUES(?1,'typing','test','nida',?2,100,100,1,1,0,0,600,100,1)",rusqlite::params![p.id,i]).unwrap();
    }
    tx.commit().unwrap();
    let start = Instant::now();
    let page = sessions::history(&c, p.id, &HistoryFilter::default(), 0).unwrap();
    assert_eq!(page.total, 10000);
    assert_eq!(page.rows.len(), 50);
    assert!(
        start.elapsed().as_millis() < 100,
        "history took {:?}",
        start.elapsed()
    );
}
#[test]
fn checkpoint_survives_restart_and_save_clears_it() {
    let dir = Temp::new();
    let database = Database::open(dir.0.clone()).unwrap();
    let p = profiles::create(&database.connection.lock().unwrap(), input("A")).unwrap();
    let cp = Checkpoint {
        session: sample(p.id, "check"),
        engine: "snapshot".into(),
        remaining_ms: Some(30_000),
    };
    sessions::checkpoint(&database.connection.lock().unwrap(), &cp).unwrap();
    drop(database);
    let database = Database::open(dir.0.clone()).unwrap();
    let mut c = database.connection.lock().unwrap();
    assert_eq!(
        sessions::checkpoints(&c, p.id).unwrap()[0].remaining_ms,
        Some(30_000)
    );
    sessions::save(&mut c, &cp.session).unwrap();
    assert!(sessions::checkpoints(&c, p.id).unwrap().is_empty());
}
#[test]
fn finished_session_cannot_gain_stale_checkpoint() {
    let mut c = memory();
    let p = profiles::create(&c, input("A")).unwrap();
    let s = sample(p.id, "done");
    sessions::save(&mut c, &s).unwrap();
    sessions::checkpoint(
        &c,
        &Checkpoint {
            session: s,
            engine: "snapshot".into(),
            remaining_ms: None,
        },
    )
    .unwrap();
    assert!(sessions::checkpoints(&c, p.id).unwrap().is_empty());
}
#[test]
fn restart_keeps_history() {
    let dir = Temp::new();
    let database = Database::open(dir.0.clone()).unwrap();
    let mut c = database.connection.lock().unwrap();
    let p = profiles::create(&c, input("A")).unwrap();
    sessions::save(&mut c, &sample(p.id, "restart")).unwrap();
    drop(c);
    drop(database);
    let database = Database::open(dir.0.clone()).unwrap();
    assert_eq!(
        sessions::history(
            &database.connection.lock().unwrap(),
            p.id,
            &HistoryFilter::default(),
            0
        )
        .unwrap()
        .total,
        1
    );
}
#[test]
fn backup_restore_keeps_pre_restore_copy() {
    let dir = Temp::new();
    let database = Database::open(dir.0.join("data")).unwrap();
    let p = profiles::create(&database.connection.lock().unwrap(), input("A")).unwrap();
    let path = dir.0.join("copy.svabak");
    backup::create(&database.connection.lock().unwrap(), &path).unwrap();
    profiles::create(&database.connection.lock().unwrap(), input("B")).unwrap();
    backup::restore(&database, &path).unwrap();
    assert_eq!(
        profiles::list(&database.connection.lock().unwrap())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        profiles::list(&database.connection.lock().unwrap()).unwrap()[0].id,
        p.id
    );
    assert!(std::fs::read_dir(&database.dir).unwrap().any(|f| f
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with("pre-restore-")));
}
#[test]
fn invalid_restore_leaves_database_untouched() {
    let dir = Temp::new();
    let database = Database::open(dir.0.join("data")).unwrap();
    profiles::create(&database.connection.lock().unwrap(), input("A")).unwrap();
    let bad = dir.0.join("bad.svabak");
    std::fs::write(&bad, b"not sqlite").unwrap();
    assert!(backup::restore(&database, &bad).is_err());
    assert_eq!(
        profiles::list(&database.connection.lock().unwrap())
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn export_has_utf8_bom_and_formula_protection() {
    let dir = Temp::new();
    let mut c = memory();
    let p = profiles::create(&c, input("=1+2")).unwrap();
    sessions::save(&mut c, &sample(p.id, "csv")).unwrap();
    let file = dir.0.join("test.csv");
    backup::export_csv(&c, &file, Some(p.id)).unwrap();
    let bytes = std::fs::read(&file).unwrap();
    assert!(bytes.starts_with(b"\xef\xbb\xbf"));
    assert!(String::from_utf8(bytes).unwrap().contains("'=1+2"));
    assert!(backup::export_csv(&c, &file, Some(p.id)).is_err());
}
#[test]
fn mode_paths_priority() {
    let root = Temp::new();
    let exe = root.0.join("exe");
    let local = root.0.join("local");
    let shared = root.0.join("shared");
    std::fs::create_dir_all(&exe).unwrap();
    std::fs::create_dir_all(shared.join("SvaType")).unwrap();
    assert_eq!(
        crate::paths::resolve(&exe, &local, &shared),
        local.join("SvaType")
    );
    std::fs::write(shared.join("SvaType/shared.txt"), "").unwrap();
    assert_eq!(
        crate::paths::resolve(&exe, &local, &shared),
        shared.join("SvaType")
    );
    std::fs::write(exe.join("portable.txt"), "").unwrap();
    assert_eq!(
        crate::paths::resolve(&exe, &local, &shared),
        exe.join("data")
    );
}
#[test]
fn bundled_content_schema_and_layout_coverage() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let result = std::process::Command::new("node")
        .arg("scripts/validate-content.mjs")
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let content = crate::content::Content::new(root.join("content"));
    for level in 1..=5 {
        let bank = content.bank(level).unwrap();
        assert!(!bank.reviewed);
        for id in ["nida", "khmer_legacy"] {
            let layout = content.layout(id).unwrap();
            for text in bank
                .words
                .iter()
                .chain(bank.sentences.iter())
                .chain(bank.paragraphs.iter())
            {
                khmer_core::plan::validate_target(&khmer_core::plan::prepare_target(text), &layout)
                    .unwrap_or_else(|error| panic!("{id} level{level} {text}: {error:?}"));
            }
        }
    }
}
#[test]
fn custom_texts_are_profile_scoped_and_validated() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let content = crate::content::Content::new(root.join("content"));
    let conn = memory();
    let a = profiles::create(&conn, input("A")).unwrap();
    let b = profiles::create(&conn, input("B")).unwrap();
    let entry = super::typing::CustomInput {
        profile: a.id,
        title: "My text".into(),
        body: "ក\u{200b}ខ".into(),
        layout_id: "nida".into(),
    };
    let id = super::typing::custom_save(&conn, &content, entry.clone()).unwrap();
    assert_eq!(
        super::typing::custom_list(&conn, a.id).unwrap()[0].body,
        "កខ"
    );
    assert!(super::typing::custom_list(&conn, b.id).unwrap().is_empty());
    super::typing::custom_delete(&conn, b.id, id).unwrap();
    assert_eq!(super::typing::custom_list(&conn, a.id).unwrap().len(), 1);
    let mut bad = entry;
    bad.body = "🙂".into();
    assert!(super::typing::custom_save(&conn, &content, bad).is_err());
}
#[test]
fn analysis_reads_recent_aggregate_data() {
    let mut c = memory();
    let p = profiles::create(&c, input("A")).unwrap();
    sessions::save(&mut c, &sample(p.id, "analysis")).unwrap();
    let result = super::typing::analysis(&c, p.id, 50).unwrap();
    assert_eq!(result.weak[0].code, "KeyK");
    assert_eq!(result.confusions[0].count, 5);
}

#[test]
fn lesson_draft_resume_and_partial_upgrade() {
    let mut c = memory();
    let p = profiles::create(&c, input("Learner")).unwrap();
    let mut d = super::learning::Draft {
        profile_id: p.id,
        lesson_id: "test_lesson".into(),
        uid: "lesson-attempt".into(),
        step: 2,
        summaries: vec![],
        texts: vec![],
        started_at: 1000,
        layout_id: "nida".into(),
    };
    super::learning::set(&mut c, &d).unwrap();
    assert_eq!(
        super::learning::get(&c, p.id, "test_lesson")
            .unwrap()
            .unwrap()
            .step,
        2
    );
    let mut partial = sample(p.id, &d.uid);
    partial.completed = false;
    partial.bookmark_step = Some(2);
    let id = sessions::save(&mut c, &partial).unwrap();
    assert_eq!(
        c.query_row("SELECT attempts FROM lesson_progress", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    d.step = 3;
    d.summaries.push(partial.summary.clone());
    super::learning::set(&mut c, &d).unwrap();
    let mut complete = partial.clone();
    complete.completed = true;
    complete.bookmark_step = None;
    complete.summary = super::learning::summary(&d, Some(partial.summary));
    assert_eq!(sessions::save(&mut c, &complete).unwrap(), id);
    assert!(super::learning::get(&c, p.id, "test_lesson")
        .unwrap()
        .is_none());
    assert_eq!(
        c.query_row("SELECT attempts,best_stars FROM lesson_progress", [], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
        })
        .unwrap(),
        (1, 3)
    );
    assert_eq!(
        sessions::history(&c, p.id, &HistoryFilter::default(), 0)
            .unwrap()
            .total,
        1
    );
    assert_eq!(sessions::detail(&c, id).unwrap().summary.keystrokes, 200);
}
#[test]
fn every_lesson_target_plans_and_paths_resolve() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let content = crate::content::Content::new(root.join("content"));
    let index = content.json("", "index").unwrap();
    let mut types = std::collections::HashSet::new();
    for lesson in index["lessons"].as_array().unwrap() {
        let id = lesson["id"].as_str().unwrap();
        let value = content.json("lessons", id).unwrap();
        assert_eq!(value["reviewed"], false);
        for step in value["steps"].as_array().unwrap() {
            let kind = step["type"].as_str().unwrap();
            types.insert(kind.to_owned());
            if ["explain", "demo"].contains(&kind) {
                continue;
            }
            let text = step["text"].as_str().map(str::to_owned).unwrap_or_else(|| {
                step["items"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .collect::<Vec<_>>()
                    .join(if kind == "find_key" { "" } else { " " })
            });
            for layout in ["nida", "khmer_legacy"] {
                khmer_core::plan::validate_target(&text, &content.layout(layout).unwrap())
                    .unwrap_or_else(|e| panic!("{id} {layout} {text}: {e:?}"));
            }
        }
    }
    assert_eq!(types.len(), 8);
    for path in index["paths"].as_array().unwrap() {
        let value = content.json("paths", path["id"].as_str().unwrap()).unwrap();
        for unit in value["units"].as_array().unwrap() {
            for id in unit["lessons"].as_array().unwrap() {
                assert!(index["lessons"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|l| l["id"] == *id));
            }
        }
    }
}
