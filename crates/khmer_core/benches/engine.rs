use criterion::{black_box, criterion_group, criterion_main, Criterion};
use khmer_core::{
    layout::Layout,
    plan::plan,
    session::{KeyEvent, Policy, Session},
};
fn engine(c: &mut Criterion) {
    let layout: Layout =
        serde_json::from_str(include_str!("../../../content/layouts/nida.json")).unwrap();
    let layout = layout.prepare().unwrap();
    let text = "កខគឃងចឆជឈញ".repeat(200);
    c.bench_function("plan_2000_chars", |b| {
        b.iter(|| plan(black_box(&text), &layout).unwrap())
    });
    let target = "ក".repeat(10_000);
    let mut session = Session::new(&target, layout, Policy::StopOnError, false).unwrap();
    let code = session.expected_next().unwrap().code.clone();
    let mut time = 0;
    c.bench_function("on_key", |b| {
        b.iter(|| {
            time += 1;
            if session.finished() {
                session.restart();
            }
            black_box(session.on_key(KeyEvent::new(&code, time)));
        })
    });
}
criterion_group!(benches, engine);
criterion_main!(benches);
