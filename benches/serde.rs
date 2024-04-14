use criterion::{black_box, criterion_group, criterion_main, Criterion};
use talk_serde_dyn_schema::{array_def, fast, slow, struct_def, ty::Ty};

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
    let binary = slow::deserialize(&schema, &doc).unwrap();

    c.bench_function("deserialize", |b| {
        b.iter(|| {
            let doc = serde_json::from_str(&json).unwrap();
            slow::deserialize(black_box(&schema), black_box(&doc))
        })
    });

    let mut group = c.benchmark_group("serialize");
    group.bench_function("serialize_slow", |b| {
        b.iter(|| {
            let doc = slow::serialize(black_box(&schema), black_box(&binary)).unwrap();
            serde_json::to_string_pretty(&doc)
        })
    });
    group.bench_function("serialize_fast", |b| {
        b.iter(|| {
            let mut buffer = vec![];
            let mut ser = serde_json::Serializer::pretty(&mut buffer);
            fast::serialize(&mut ser, black_box(&schema), black_box(&binary)).unwrap();
            String::from_utf8(buffer).unwrap()
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
