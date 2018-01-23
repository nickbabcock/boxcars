extern crate bitreader;
#[macro_use]
extern crate criterion;

use criterion::{black_box, Criterion};
use bitreader::BitReader;

static DATA: [u8; 0x10_000] = [0; 0x10_000];

fn bench_bit_reader(c: &mut Criterion) {
    c.bench_function("bit_reader", |b| {
        b.iter(|| {
            let mut bits = BitReader::new(&DATA);
            for _ in 0..1000 {
                black_box(bits.read_u8(8).unwrap());
            }
        })
    });
}

criterion_group!(bitreader, bench_bit_reader);

criterion_main!(bitreader);
