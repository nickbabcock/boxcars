#![feature(test)]

extern crate test;
extern crate boxcars;
extern crate serde_json;

use test::Bencher;
use boxcars::*;
use std::io;

#[bench]
fn bench_parse_crc(b: &mut Bencher) {
    let data = include_bytes!("../assets/rumble.replay");
    b.iter(|| {
        assert!(parse(data, true).is_ok());
    });
}

#[bench]
fn bench_parse_no_crc(b: &mut Bencher) {
    let data = include_bytes!("../assets/rumble.replay");
    b.iter(|| {
        assert!(parse(data, false).is_ok());
    });
}

#[bench]
fn bench_parse_crc_json(b: &mut Bencher) {
    let data = include_bytes!("../assets/rumble.replay");
    b.iter(|| {
        let data = parse(data, true).unwrap();
        assert!(serde_json::to_writer(&mut io::sink(), &data).is_ok());
    });
}

#[bench]
fn bench_parse_no_crc_json(b: &mut Bencher) {
    let data = include_bytes!("../assets/rumble.replay");
    b.iter(|| {
        let data = parse(data, false).unwrap();
        assert!(serde_json::to_writer(&mut io::sink(), &data).is_ok());
    });
}
