use chrono::DateTime;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fluent_ecs::fluent_ecs_filter_rust;
use std::{fs, sync::Once};

static INIT_LOGGER: Once = Once::new();

fn init_logger() {
    INIT_LOGGER.call_once(|| {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::max())
            .is_test(true)
            .try_init();
    })
}

pub fn fluent_ecs_bench(c: &mut Criterion) {
    init_logger();

    let some_time = match DateTime::parse_from_rfc3339("2023-11-16T13:27:38.555+01:00") {
        Ok(time) => time,
        Err(err) => panic!("Could not parse date: {}", err),
    };

    let mut group = c.benchmark_group("fluent_ecs_bench");
    for test_case in ["postfix/smtpd_auth_failed", "metallb/speaker_partial_join", "kubernetes_statefulset"].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(test_case),
            test_case,
            |b, &test_case| {
                let input = match fs::read(format!("examples/{}-in.json", test_case)) {
                    Ok(input) => input,
                    Err(err) => panic!("Input file could not be read: {}", err),
                };

                b.iter(|| fluent_ecs_filter_rust(black_box(&input), black_box(some_time)))
            },
        );
    }
}

criterion_group!(benches, fluent_ecs_bench);
criterion_main!(benches);
