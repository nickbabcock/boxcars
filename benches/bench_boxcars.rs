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
    let mut bytes = Vec::new();
    let replay = ParserBuilder::new(data)
        .on_error_check_crc()
        .parse()
        .unwrap();
    serde_json::to_writer(&mut bytes, &replay).unwrap();

    let mut group = c.benchmark_group("json_throughput");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    unsafe { bytes.set_len(0); };

    group.bench_function("bench_json_serialization", |b| {
        b.iter(|| {
            black_box(serde_json::to_writer(&mut bytes, &replay).is_ok());
            unsafe { bytes.set_len(0); };
        })
    });
    group.finish();
}

fn bench_parse_crc_body(c: &mut Criterion) {
    c.bench_function("bench_parse_crc_body", |b| {
        let data = include_bytes!("../assets/replays/good/3381.replay");
        b.iter(|| {
            black_box(
                ParserBuilder::new(data)
                    .always_check_crc()
                    .must_parse_network_data()
                    .parse()
                    .is_ok(),
            )
        });
    });
}

fn bench_parse_no_crc_body(c: &mut Criterion) {
    c.bench_function("bench_parse_no_crc_body", |b| {
        let data = include_bytes!("../assets/replays/good/3381.replay");
        b.iter(|| {
            black_box(
                ParserBuilder::new(data)
                    .on_error_check_crc()
                    .must_parse_network_data()
                    .parse()
                    .is_ok(),
            )
        });
    });
}

fn bench_parse_no_crc_no_body(c: &mut Criterion) {
    c.bench_function("bench_parse_no_crc_no_body", |b| {
        let data = include_bytes!("../assets/replays/good/3381.replay");
        b.iter(|| {
            black_box(
                ParserBuilder::new(data)
                    .on_error_check_crc()
                    .never_parse_network_data()
                    .parse()
                    .is_ok(),
            )
        });
    });
}

fn bench_parse_crc_json(c: &mut Criterion) {
    c.bench_function("bench_parse_crc_json", |b| {
        let data = include_bytes!("../assets/replays/good/3381.replay");

        // allocate a buffer big enough to hold all of the serialized data
        let mut bytes = Vec::with_capacity(2_usize.pow(25));
        b.iter(|| {
            let data = ParserBuilder::new(data).always_check_crc().parse().unwrap();
            black_box(serde_json::to_writer(&mut bytes, &data).is_ok());
            unsafe { bytes.set_len(0); }
        });
    });
}

fn bench_parse_no_crc_json(c: &mut Criterion) {
    c.bench_function("bench_parse_no_crc_json", |b| {
        let data = include_bytes!("../assets/replays/good/3381.replay");

        // allocate a buffer big enough to hold all of the serialized data
        let mut bytes = Vec::with_capacity(2_usize.pow(25));
        b.iter(|| {
            let replay = ParserBuilder::new(data)
                .on_error_check_crc()
                .parse()
                .unwrap();
            black_box(serde_json::to_writer(&mut bytes, &replay).is_ok());
            unsafe { bytes.set_len(0); }
        });
    });
}

criterion_group!(
    benches,
    bench_crc,
    bench_json_serialization,
    bench_parse_crc_body,
    bench_parse_no_crc_body,
    bench_parse_no_crc_no_body,
    bench_parse_crc_json,
    bench_parse_no_crc_json
);

criterion_main!(benches);
