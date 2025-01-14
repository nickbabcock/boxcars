const fn gen_crc_table<const N: usize>(poly: u32) -> [[u32; 256]; N] {
    let mut table = [[0u32; 256]; N];

    let mut i = 0usize;
    while i < 256 {
        let mut crc = (i as u32) << 24;
        let mut j = 0;
        while j < 8 {
            crc = if crc & 0x8000_0000 > 0 {
                (crc << 1) ^ poly
            } else {
                crc << 1
            };

            j += 1;
        }
        table[0][i] = crc.swap_bytes();
        i += 1;
    }

    i = 0;
    while i < 256 {
        let mut crc = table[0][i].swap_bytes();
        let mut j = 1;
        while j < N {
            crc = (table[0][(crc >> 24) as usize]).swap_bytes() ^ (crc << 8);
            table[j][i] = crc.swap_bytes();
            j += 1;
        }
        i += 1;
    }
    table
}

// Prefer static over const to cut test times in half
// ref: https://github.com/srijs/rust-crc32fast/commit/e61ce6a39bbe9da495198a4037292ec299e8970f
static CRC_TABLE: [[u32; 256]; 16] = gen_crc_table(0x04c1_1db7);

/// Calculates the crc-32 for rocket league replays. Not all CRC algorithms are the same. The crc
/// algorithm can be generated with the following parameters (pycrc):
///
/// - `Width` = 32
/// - `Poly` = 0x04c11db7
/// - `XorIn` = 0x10340dfe
/// - `ReflectIn` = False
/// - `XorOut` = 0xffffffff
/// - `ReflectOut` = False
///
/// This implementation is a slice by 16 from the unreal engine seen in Bakkes' CPPRP
/// (https://github.com/Bakkes/CPPRP/blob/58fc19a972a7a0af059407982bdf553cfe091831/CPPRP/CRC.h#L245)
pub fn calc_crc(data: &[u8]) -> u32 {
    let mut crc = !(0xefcb_f201_u32.swap_bytes());
    crc = data.chunks_exact(16).fold(crc, |acc, sl| {
        let top = u32::from_le_bytes([sl[0], sl[1], sl[2], sl[3]]);
        let one = top ^ acc;
        CRC_TABLE[0][sl[15] as usize]
            ^ CRC_TABLE[1][sl[14] as usize]
            ^ CRC_TABLE[2][sl[13] as usize]
            ^ CRC_TABLE[3][sl[12] as usize]
            ^ CRC_TABLE[4][sl[11] as usize]
            ^ CRC_TABLE[5][sl[10] as usize]
            ^ CRC_TABLE[6][sl[9] as usize]
            ^ CRC_TABLE[7][sl[8] as usize]
            ^ CRC_TABLE[8][sl[7] as usize]
            ^ CRC_TABLE[9][sl[6] as usize]
            ^ CRC_TABLE[10][sl[5] as usize]
            ^ CRC_TABLE[11][sl[4] as usize]
            ^ CRC_TABLE[12][((one >> 24) & 0xFF) as usize]
            ^ CRC_TABLE[13][((one >> 16) & 0xFF) as usize]
            ^ CRC_TABLE[14][((one >> 8) & 0xFF) as usize]
            ^ CRC_TABLE[15][(one & 0xFF) as usize]
    });

    let left_over = data.len() % 16;
    crc = data[data.len() - left_over..].iter().fold(crc, |acc, &x| {
        (acc >> 8) ^ CRC_TABLE[0][(u32::from(x) ^ (acc & 0xFF)) as usize]
    });

    (!crc).swap_bytes()
}

#[cfg(test)]
mod tests {
    use crate::crc::calc_crc;

    #[test]
    fn crc_rumble_test() {
        let data = include_bytes!("../assets/replays/good/rumble.replay");
        assert_eq!(calc_crc(&data[..]), 2034487435);
    }

    #[test]
    fn single_byte_test() {
        assert_eq!(calc_crc(&[0xa0]), 0x76cc8c81);
    }
}
