#![feature(test)]

extern crate bitter;
extern crate test;

use test::{black_box, Bencher};
use bitter::BitGet;

#[bench]
fn bench_read_u32_bits_unchecked(b: &mut Bencher) {
    b.iter(|| {
        let mut bits = BitGet::new(&[0; 0x10000]);
        for _ in 0..1000 {
            black_box(bits.read_u32_bits_unchecked(8));
        }
    });
}

#[bench]
fn bench_read_u32_bits_checked(b: &mut Bencher) {
    b.iter(|| {
        let mut bits = BitGet::new(&[0; 0x10000]);
        for _ in 0..1000 {
            black_box(bits.read_u32_bits(8));
        }
    });
}

#[bench]
fn bench_read_u8_unchecked(b: &mut Bencher) {
    b.iter(|| {
        let mut bits = BitGet::new(&[0; 0x10000]);
        for _ in 0..1000 {
            black_box(bits.read_u8_unchecked());
        }
    });
}

#[bench]
fn bench_read_u8_checked(b: &mut Bencher) {
    b.iter(|| {
        let mut bits = BitGet::new(&[0; 0x10000]);
        for _ in 0..1000 {
            black_box(bits.read_u8());
        }
    });
}

#[bench]
fn bench_read_f32_checked(b: &mut Bencher) {
    b.iter(|| {
        let mut bits = BitGet::new(&[0; 0x10000]);
        for _ in 0..1000 {
            black_box(bits.read_f32());
        }
    });
}

#[bench]
fn bench_read_f32_unchecked(b: &mut Bencher) {
    b.iter(|| {
        let mut bits = BitGet::new(&[0; 0x10000]);
        for _ in 0..1000 {
            black_box(bits.read_f32_unchecked());
        }
    });
}
