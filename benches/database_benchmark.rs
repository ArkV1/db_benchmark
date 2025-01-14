use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use nosql_db::{Database, Value};
use redis::Commands;
use tokio::runtime::Runtime;
use std::fs;
use std::time::Duration;

fn benchmark_put(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("Put Operation");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(50);

    // Your NoSQL DB
    group.bench_function(BenchmarkId::new("YourDB", "Put"), |b| {
        let db_file = format!("benchmark_db_{}.json", std::process::id());
        let _ = fs::remove_file(&db_file);
        let db = rt.block_on(async {
            Database::new(&db_file).await.expect("Failed to create database")
        });

        b.iter(|| {
            rt.block_on(async {
                db.put(black_box("key".to_string()), black_box(Value { data: "value".to_string() })).await.expect("Failed to put value");
            });
        });

        let _ = fs::remove_file(&db_file);
    });

    // Redis
    group.bench_function(BenchmarkId::new("Redis", "Put"), |b| {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();

        b.iter(|| {
            let _: () = con.set(black_box("key"), black_box("value")).unwrap();
        });
    });

    group.finish();
}

fn benchmark_get(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("Get Operation");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(50);
    
    // Your NoSQL DB
    group.bench_function(BenchmarkId::new("YourDB", "Get"), |b| {
        let db = rt.block_on(async {
            let db = Database::new("benchmark_db.json").await.unwrap();
            db.put("benchmark_key".to_string(), Value { data: "benchmark_value".to_string() }).await
                .expect("Failed to insert benchmark key");
            db
        });

        b.iter(|| {
            let _ = black_box(db.get("benchmark_key").unwrap());
        });
    });

    // Redis
    group.bench_function(BenchmarkId::new("Redis", "Get"), |b| {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();
        con.set::<_, _, ()>("key", "value").unwrap();

        b.iter(|| {
            let _: String = con.get(black_box("key")).unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_put, benchmark_get);
criterion_main!(benches);
