use boxcars::crc::calc_crc;
use boxcars::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn bench_crc(c: &mut Criterion) {
    let data = include_bytes!("../assets/replays/good/rumble.replay");
    let mut group = c.benchmark_group("crc_throughput");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function("bench_crc", |b| {
        let data = include_bytes!("../assets/replays/good/rumble.replay");
        b.iter(|| {
            black_box(calc_crc(&data[..]));
        })
    });
    group.finish();
}

fn bench_json_serialization(c: &mut Criterion) {
    let data = include_bytes!("../assets/replays/good/3381.replay");
    let json_data_bytes = 19484480_u64;

    let mut group = c.benchmark_group("json_throughput");
    group.sample_size(10);
    group.throughput(Throughput::Bytes(json_data_bytes));
    group.bench_function("bench_json_serialization", |b| {
        let mut bytes = Vec::new();
        let replay = ParserBuilder::new(data)
            .on_error_check_crc()
            .parse()
            .unwrap();
        serde_json::to_writer(&mut bytes, &replay).unwrap();
        assert!(
            json_data_bytes == bytes.len() as u64,
            "update json benchmark with latest throughput size: {}",
            bytes.len()
        );
        unsafe {
            bytes.set_len(0);
        };

        b.iter(|| {
            black_box(serde_json::to_writer(&mut bytes, &replay).unwrap());
            unsafe {
                bytes.set_len(0);
            };
        })
    });
    group.finish();
}

fn bench_parse_crc_body(c: &mut Criterion) {
    let data = include_bytes!("../assets/replays/good/3381.replay");
    let mut group = c.benchmark_group("parse_crc_body");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.sample_size(20);
    group.bench_function("bench_parse_crc_body", |b| {
        b.iter(|| {
            black_box(
                ParserBuilder::new(data)
                    .always_check_crc()
                    .must_parse_network_data()
                    .parse()
                    .unwrap(),
            )
        });
    });
    group.finish();
}

fn bench_parse_no_crc_body(c: &mut Criterion) {
    let data = include_bytes!("../assets/replays/good/3381.replay");
    let mut group = c.benchmark_group("parse_no_crc_body");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.sample_size(20);
    group.bench_function("bench_parse_no_crc_body", |b| {
        b.iter(|| {
            black_box(
                ParserBuilder::new(data)
                    .on_error_check_crc()
                    .must_parse_network_data()
                    .parse()
                    .unwrap(),
            )
        });
    });
    group.finish();
}

fn bench_parse_no_crc_no_body(c: &mut Criterion) {
    // Throughput not included in this benchmark as it is a bit confusing what the number reporting
    // represents. If parsing the header reports based on the entire size of the replay, it will be
    // some very large number (20 GB/s), else it may look deceptively small
    let data = include_bytes!("../assets/replays/good/3381.replay");
    let mut group = c.benchmark_group("parse_no_crc_no_body");
    group.bench_function("bench_parse_no_crc_no_body", |b| {
        b.iter(|| {
            black_box(
                ParserBuilder::new(data)
                    .on_error_check_crc()
                    .never_parse_network_data()
                    .parse()
                    .unwrap(),
            )
        });
    });
    group.finish();
}

fn bench_parse_crc_json(c: &mut Criterion) {
    let data = include_bytes!("../assets/replays/good/3381.replay");
    let mut group = c.benchmark_group("parse_crc_json");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.sample_size(10);
    group.bench_function("bench_parse_crc_json", |b| {
        // allocate a buffer big enough to hold all of the serialized data
        let mut bytes = Vec::with_capacity(2_usize.pow(25));
        b.iter(|| {
            let data = ParserBuilder::new(data).always_check_crc().parse().unwrap();
            black_box(serde_json::to_writer(&mut bytes, &data).unwrap());
            unsafe {
                bytes.set_len(0);
            }
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_crc,
    bench_json_serialization,
    bench_parse_crc_body,
    bench_parse_no_crc_body,
    bench_parse_no_crc_no_body,
    bench_parse_crc_json,
);

criterion_main!(benches);
