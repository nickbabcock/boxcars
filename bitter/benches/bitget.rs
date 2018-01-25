extern crate bitter;
#[macro_use]
extern crate criterion;

use criterion::{black_box, Criterion};
use bitter::BitGet;

static DATA: [u8; 0x10_000] = [0; 0x10_000];

fn bench_read_u32_bits_unchecked(c: &mut Criterion) {
    c.bench_function("read_u32_bits_unchecked", |b| {
        b.iter(|| {
            let mut bits = BitGet::new(&DATA);
            for _ in 0..1000 {
                black_box(bits.read_u32_bits_unchecked(8));
            }
        })
    });
}

fn bench_read_u32_bits_unaligned(c: &mut Criterion) {
    c.bench_function("read_u32_bits_unaligned", |b| {
        b.iter(|| {
            let mut bits = BitGet::new(&DATA);
            for _ in 0..1000 {
                black_box(bits.read_u32_bits(7));
            }
        })
    });
}

fn bench_read_u32_bits_checked(c: &mut Criterion) {
    c.bench_function("read_u32_bits_checked", |b| {
        b.iter(|| {
            let mut bits = BitGet::new(&DATA);
            for _ in 0..1000 {
                black_box(bits.read_u32_bits(8));
            }
        })
    });
}

fn bench_read_u8_unchecked(c: &mut Criterion) {
    c.bench_function("read_u8_unchecked", |b| {
        b.iter(|| {
            let mut bits = BitGet::new(&DATA);
            for _ in 0..1000 {
                black_box(bits.read_u8_unchecked());
            }
        })
    });
}

fn bench_read_u8_checked(c: &mut Criterion) {
    c.bench_function("read_u8_checked", |b| {
        b.iter(|| {
            let mut bits = BitGet::new(&DATA);
            for _ in 0..1000 {
                black_box(bits.read_u8());
            }
        })
    });
}

fn bench_read_f32_checked(c: &mut Criterion) {
    c.bench_function("read_f32_unchecked", |b| {
        b.iter(|| {
            let mut bits = BitGet::new(&DATA);
            for _ in 0..1000 {
                black_box(bits.read_f32());
            }
        })
    });
}

fn bench_read_f32_unchecked(c: &mut Criterion) {
    c.bench_function("read_f32_checked", |b| {
        b.iter(|| {
            let mut bits = BitGet::new(&DATA);
            for _ in 0..1000 {
                black_box(bits.read_f32_unchecked());
            }
        })
    });
}

criterion_group!(
    benches,
    bench_read_u32_bits_unchecked,
    bench_read_u32_bits_unaligned,
    bench_read_u32_bits_checked,
    bench_read_u8_unchecked,
    bench_read_u8_checked,
    bench_read_f32_unchecked,
    bench_read_f32_checked
);

criterion_main!(benches);
