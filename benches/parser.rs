#![feature(test)]

extern crate boxcars;
extern crate serde_json;
extern crate test;

use test::Bencher;
use boxcars::*;
use std::io;

#[bench]
fn bench_parse_crc_body(b: &mut Bencher) {
    let data = include_bytes!("../assets/rumble.replay");
    b.iter(|| {
        assert!(ParserBuilder::new(data).always_check_crc().must_parse_network_data().parse().is_ok());
    });
}

#[bench]
fn bench_parse_no_crc_body(b: &mut Bencher) {
    let data = include_bytes!("../assets/rumble.replay");
    b.iter(|| {
        assert!(ParserBuilder::new(data).on_error_check_crc().must_parse_network_data().parse().is_ok());
    });
}

#[bench]
fn bench_parse_no_crc_no_body(b: &mut Bencher) {
    let data = include_bytes!("../assets/rumble.replay");
    b.iter(|| {
        assert!(ParserBuilder::new(data).on_error_check_crc().never_parse_network_data().parse().is_ok());
    });
}

#[bench]
fn bench_parse_crc_json(b: &mut Bencher) {
    let data = include_bytes!("../assets/rumble.replay");
    b.iter(|| {
        let data = ParserBuilder::new(data).always_check_crc().parse().unwrap();
        assert!(serde_json::to_writer(&mut io::sink(), &data).is_ok());
    });
}

#[bench]
fn bench_parse_no_crc_json(b: &mut Bencher) {
    let data = include_bytes!("../assets/rumble.replay");
    b.iter(|| {
        let replay = ParserBuilder::new(data).on_error_check_crc().parse().unwrap();
        assert!(serde_json::to_writer(&mut io::sink(), &replay).is_ok());
    });
}
