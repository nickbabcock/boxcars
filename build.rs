use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn gen_crc_table(poly: u32, size: usize) -> Vec<Vec<u32>> {
    let mut table = vec![vec![0; 256]; size];
    for i in 0..256 {
        let crc = (0..8).fold(i << 24, |acc, _x| {
            if acc & 0x8000_0000 > 0 {
                (acc << 1) ^ poly
            } else {
                acc << 1
            }
        });
        table[0][i as usize] = crc.swap_bytes()
    }

    for i in 0..256 {
        let mut crc = table[0][i].swap_bytes();
        for j in 1..size {
            crc = (table[0][(crc >> 24) as usize]).swap_bytes() ^ (crc << 8);
            table[j][i] = crc.swap_bytes()
        }
    }
    table
}

fn write_crc_table() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("generated_crc.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());
    let poly = 0x04c1_1db7;
    let size = 16;
    let table = gen_crc_table(poly, size);

    writeln!(
        &mut file,
        "pub (crate) const CRC_TABLE: [[u32; 256]; {}] = [",
        size
    )
    .unwrap();
    for row in table {
        writeln!(&mut file, "\t[").unwrap();
        for i in row {
            write!(&mut file, "0x{:x}, ", i).unwrap()
        }
        writeln!(&mut file, "\t],").unwrap();
    }
    writeln!(&mut file, "];").unwrap();
}

fn main() {
    write_crc_table();
}
