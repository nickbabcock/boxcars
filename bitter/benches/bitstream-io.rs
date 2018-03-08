extern crate bitstream_io;
#[macro_use]
extern crate criterion;

use criterion::{black_box, Criterion};
use std::io::Cursor;
use bitstream_io::{BitReader, LE};

static DATA: [u8; 0x10_000] = [0; 0x10_000];

fn bench_bitstream_io_aligned(c: &mut Criterion) {
    c.bench_function("bitstream_io_aligned", |b| {
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

fn bench_bitstream_io_unaligned(c: &mut Criterion) {
    c.bench_function("bitstream_io_unaligned", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(&DATA[..]);
            {
                let mut bits = BitReader::<LE>::new(&mut cursor);
                for _ in 0..1000 {
                    black_box(bits.read::<u8>(7).unwrap());
                }
            }
        })
    });
}

criterion_group!(
    bitstream_io,
    bench_bitstream_io_aligned,
    bench_bitstream_io_unaligned
);

criterion_main!(bitstream_io);
