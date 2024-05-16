use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use talk_serde_dyn_schema::{
    array_def, binc, fast, flatbin::{Builder, FlatbinBuf}, slow, struct_def, ty::Ty
};

fn criterion_benchmark(c: &mut Criterion) {
    let schema = struct_def!({
        "name": Ty::String,
        "age": Ty::U64,
        "hobbies": array_def!(Ty::String),
        "languages": array_def!(struct_def!({
            "name": Ty::String,
            "liked": Ty::Bool,
            "experience": Ty::U64,
        }))
    });

    let doc = serde_json::json!({
        "name": "Alexander",
        "age": 27,
        "hobbies": [
            "Music",
            "Programming",
            "Reading"
        ],
        "languages": [{
            "name": "Rust",
            "liked": true,
            "experience": 5
        }, {
            "name": "Typescript",
            "liked": true,
            "experience": 4
        }, {
            "name": "PHP",
            "liked": false,
            "experience": 2
        }, {
            "name": "Java",
            "liked": false,
            "experience": 1
        }]
    });

    let json = serde_json::to_string_pretty(&doc).unwrap();
    let binary = slow::deserialize_alloc(&schema, &doc).unwrap();
    let binc_binary = binc::serialize(&schema, &doc).unwrap();

    let mut buffer = FlatbinBuf::new();
    let mut binc_buffer = Vec::new();

    let mut group = c.benchmark_group("deserialize");
    group.measurement_time(Duration::from_secs(30));
    group.bench_function("deserialize_slow", |b| {
        b.iter(|| {
            buffer.clear();
            let doc = serde_json::from_str(&json).unwrap();
            slow::deserialize(black_box(&schema), black_box(&doc), Builder::new(&mut buffer)).unwrap()
        })
    });
    group.bench_function("deserialize_slow_with_doc", |b| {
        b.iter(|| {
            buffer.clear();
            slow::deserialize(black_box(&schema), black_box(&doc), Builder::new(&mut buffer)).unwrap()
        })
    });
    group.bench_function("deserialize_binc", |b| {
        b.iter(|| {
            binc_buffer.clear();
            let doc = serde_json::from_str(&json).unwrap();
            binc::serialize_into(black_box(&schema), black_box(&doc), &mut binc_buffer).unwrap()
        })
    });
    group.bench_function("deserialize_binc_with_doc", |b| {
        b.iter(|| {
            binc_buffer.clear();
            binc::serialize_into(black_box(&schema), black_box(&doc), &mut binc_buffer).unwrap()
        })
    });
    group.bench_function("deserialize_fast", |b| {
        b.iter(|| {
            buffer.clear();
            fast::deserialize_into(black_box(&schema), black_box(&json), &mut buffer).unwrap()
        })
    });
    group.finish();

    let mut group = c.benchmark_group("serialize");
    group.measurement_time(Duration::from_secs(30));
    group.bench_function("serialize_slow", |b| {
        b.iter(|| {
            slow::serialize(black_box(&schema), black_box(&binary)).unwrap()
        })
    });
    group.bench_function("serialize_binc", |b| {
        b.iter(|| {
            binc::deserialize(black_box(&schema), black_box(&binc_binary)).unwrap()
        })
    });
    group.bench_function("serialize_fast", |b| {
        b.iter(|| {
            fast::serialize(serde_json::value::Serializer, black_box(&schema), black_box(&binary)).unwrap()
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
