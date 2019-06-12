#[macro_use]
extern crate criterion;
use serde_json;

use boxcars::crc::calc_crc;
use boxcars::*;
use criterion::{black_box, Criterion};
use std::io;

fn bench_crc(c: &mut Criterion) {
    c.bench_function("bench_crc", |b| {
        let data = include_bytes!("../assets/replays/good/rumble.replay");
        b.iter(|| {
            black_box(calc_crc(&data[..]));
        })
    });
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
        b.iter(|| {
            let data = ParserBuilder::new(data).always_check_crc().parse().unwrap();
            black_box(serde_json::to_writer(&mut io::sink(), &data).is_ok());
        });
    });
}

fn bench_parse_no_crc_json(c: &mut Criterion) {
    c.bench_function("bench_parse_no_crc_json", |b| {
        let data = include_bytes!("../assets/replays/good/3381.replay");
        b.iter(|| {
            let replay = ParserBuilder::new(data)
                .on_error_check_crc()
                .parse()
                .unwrap();
            black_box(serde_json::to_writer(&mut io::sink(), &replay).is_ok());
        });
    });
}

criterion_group!(
    benches,
    bench_crc,
    bench_parse_crc_body,
    bench_parse_no_crc_body,
    bench_parse_no_crc_no_body,
    bench_parse_crc_json,
    bench_parse_no_crc_json
);

criterion_main!(benches);
