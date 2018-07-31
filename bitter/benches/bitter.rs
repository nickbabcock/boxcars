extern crate bitreader;
extern crate bitstream_io;
extern crate bitter;
#[macro_use]
extern crate criterion;
#[macro_use]
extern crate nom;

use bitreader::BitReader;
use bitstream_io::{BitReader as bio_br, LE};
use bitter::BitGet;
use criterion::{black_box, Bencher, Benchmark, Criterion, Throughput};
use std::io::Cursor;

static DATA: [u8; 0x10_000] = [0; 0x10_000];

const ITER: u32 = 1000;

macro_rules! ben {
    ($ex1:expr, $ex2:expr) => {
        |b| {
            b.iter(|| {
                let mut bits = $ex1;
                for _ in 0..ITER {
                    black_box($ex2(&mut bits));
                }
            });
        }
    };
}

fn nom_bench(b: &mut Bencher, bits: usize) {
    b.iter(|| {
        let mut d = &DATA[..];
        let mut pos = 0;
        for _ in 0..ITER {
            let ((left, new_pos), _) = take_bits!((&d[..], pos), u8, bits).unwrap();
            pos = new_pos;
            d = left;
        }
    })
}

fn bitstream_bench(b: &mut Bencher, num_bits: u32) {
    b.iter(|| {
        let mut cursor = Cursor::new(&DATA[..]);
        {
            let mut bits = bio_br::<LE>::new(&mut cursor);
            for _ in 0..ITER {
                black_box(bits.read::<u8>(num_bits).unwrap());
            }
        }
    })
}

fn eight_bits(c: &mut Criterion) {
    let bench = Benchmark::new(
        "bitter_arbitrary_unchecked",
        ben!(BitGet::new(&DATA), |x: &mut BitGet| x
            .read_u32_bits_unchecked(8)),
    ).with_function(
        "bitter_arbitrary_checked",
        ben!(BitGet::new(&DATA), |x: &mut BitGet| x.read_u32_bits(8)),
    ).with_function(
        "bitter_byte_unchecked",
        ben!(BitGet::new(&DATA), |x: &mut BitGet| x.read_u8_unchecked()),
    ).with_function(
        "bitter_byte_checked",
        ben!(BitGet::new(&DATA), |x: &mut BitGet| x.read_u8()),
    ).with_function(
        "bitreader",
        ben!(BitReader::new(&DATA), |x: &mut BitReader| x
            .read_u8(8)
            .unwrap()),
    ).with_function("nom", |b| nom_bench(b, 8))
    .with_function("bitstream_io", |b| bitstream_bench(b, 8))
    .throughput(Throughput::Bytes(8 * ITER));

    c.bench("eight_bits", bench);
}

fn seven_bits(c: &mut Criterion) {
    let bench = Benchmark::new(
        "bitter_arbitrary_unchecked",
        ben!(BitGet::new(&DATA), |x: &mut BitGet| x
            .read_u32_bits_unchecked(7)),
    ).with_function(
        "bitter_arbitrary_checked",
        ben!(BitGet::new(&DATA), |x: &mut BitGet| x.read_u32_bits(7)),
    ).with_function(
        "bitreader",
        ben!(BitReader::new(&DATA), |x: &mut BitReader| x
            .read_u8(7)
            .unwrap()),
    ).with_function("nom", |b| nom_bench(b, 7))
    .with_function("bitstream_io", |b| bitstream_bench(b, 7))
    .throughput(Throughput::Bytes(7 * ITER));

    c.bench("seven_bits", bench);
}

criterion_group!(benches, eight_bits, seven_bits);

criterion_main!(benches);
