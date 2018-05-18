#[macro_use]
extern crate criterion;
#[macro_use]
extern crate nom;

use criterion::Criterion;

static DATA: [u8; 0x10_000] = [0; 0x10_000];

fn bench_nom_aligned(c: &mut Criterion) {
    c.bench_function("nom_aligned", |b| {
        b.iter(|| {
            let mut d = &DATA[..];
            let mut pos = 0;
            for _ in 0..1000 {
                let ((left, new_pos), _) = take_bits!((&d[..], pos), u8, 8).unwrap();
                pos = new_pos;
                d = left;
            }
        })
    });
}

fn bench_nom_unaligned(c: &mut Criterion) {
    c.bench_function("nom_unaligned", |b| {
        b.iter(|| {
            let mut d = &DATA[..];
            let mut pos = 0;
            for _ in 0..1000 {
                let ((left, new_pos), _) = take_bits!((&d[..], pos), u8, 7).unwrap();
                pos = new_pos;
                d = left;
            }
        })
    });
}

criterion_group!(nom, bench_nom_aligned, bench_nom_unaligned);

criterion_main!(nom);
