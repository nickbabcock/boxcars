extern crate bitstream_io;
#[macro_use]
extern crate criterion;

use criterion::{black_box, Criterion};
use std::io::{Cursor};
use bitstream_io::{LE, BitReader};

static DATA: [u8; 0x10_000] = [0; 0x10_000];

fn bench_bitstream_io(c: &mut Criterion) {
    c.bench_function("bitstream_io", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(&DATA[..]);
            {
                let mut bits = BitReader::<LE>::new(&mut cursor);
                for _ in 0..1000 {
                    black_box(bits.read::<u8>(8).unwrap());
                }
            }
        })
    });
}

criterion_group!(bitstream_io, bench_bitstream_io);

criterion_main!(bitstream_io);
