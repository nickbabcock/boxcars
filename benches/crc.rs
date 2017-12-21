#![feature(test)]

extern crate boxcars;
extern crate test;

use test::Bencher;
use boxcars::crc::calc_crc;

#[bench]
fn bench_parse_crc(b: &mut Bencher) {
    let data = include_bytes!("../assets/rumble.replay");
    b.iter(|| {
        assert_eq!(calc_crc(&data[..]), 2034487435);
    });
}
