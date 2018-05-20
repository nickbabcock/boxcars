//! Bitter takes a slice of byte data and reads little-endian bits platform agonistically. Bitter
//! has been optimized to be fast for reading 64 or fewer bits at a time, though it can still
//! extract an arbitrary number of bytes.
//!
//! There are two main APIs available: checked and unchecked functions. A checked function will
//! return a `Option` that will be `None` if there is not enough bits left in the stream.
//! Unchecked functions, which are denoted by having "unchecked" in their name, will panic if there
//! is not enough data left, but happen to be ~10% faster (your numbers
//! will vary depending on use case).
//!
//! Tips:
//!
//! - Prefer checked functions for all but the most performance critical code
//! - Group all unchecked functions in a single block guarded by a `approx_bytes_remaining` or
//!   `bits_remaining` call
//! - Prefer `read_u8()` over `read_u32_bits_unchecked(8)` as the specialized functions have a
//!   slight performance edge over the generic function
//!
//! ## Example
//!
//! ```rust
//! use bitter::BitGet;
//! let mut bitter = BitGet::new(&[0xff, 0x04]);
//! assert_eq!(bitter.read_bit(), Some(true));
//! assert_eq!(bitter.read_u8(), Some(0x7f));
//! assert_eq!(bitter.read_u32_bits(7), Some(0x02));
//! ```
//!
//! Below, is a demonstration of guarding against potential panics:
//!
//! ```rust
//! # use bitter::BitGet;
//! let mut bitter = BitGet::new(&[0xff, 0x04]);
//! if bitter.approx_bytes_remaining() >= 2 {
//!     assert_eq!(bitter.read_bit_unchecked(), true);
//!     assert_eq!(bitter.read_u8_unchecked(), 0x7f);
//!     assert_eq!(bitter.read_u32_bits_unchecked(7), 0x02);
//! }
//! # else {
//! #   panic!("Expected bytes")
//! # }
//! ```
//!
//! Another guard usage. `bits_remaining` is more accurate but involves a 3 operations to
//! calculate.
//!
//! ```rust
//! # use bitter::BitGet;
//! let mut bitter = BitGet::new(&[0xff, 0x04]);
//! if bitter.bits_remaining() >= 16 {
//!     for _ in 0..8 {
//!         assert_eq!(bitter.read_bit_unchecked(), true);
//!     }
//!     assert_eq!(bitter.read_u8_unchecked(), 0x04);
//! }
//! # else {
//! #   panic!("Expected bytes")
//! # }
//! ```
//!
//! ## Implementation
//!
//! Currently the implementation pre-fetches 64 bit chunks so that more operations can be performed
//! on a single primitive type (`u64`). Pre-fetching like this allows for operations that request
//! 4 bytes to be completed in, at best, a bit shift and mask instead of, at best, four bit
//! shifts and masks.
//!
//! ## Comparison to other libraries
//!
//! Bitter is hardly the first Rust library for handling bits.
//! [bitstream_io](https://crates.io/crates/bitstream-io) and
//! [bitreader](https://crates.io/crates/bitreader) are both crates one should consider. The reason
//! why someone would choose bitter over those two is speed. The other libraries lack a "trust me I
//! know what I'm doing" API, which bitter can give you a 10x performance increase. Additionally,
//! some libraries favor byte aligned reads (looking at you, bitstream_io), and since 7 out of 8
//! bits aren't byte aligned, there is a performance hit.

extern crate byteorder;

use byteorder::{ByteOrder, LittleEndian};

/// Yields consecutive bits as little endian primitive types
pub struct BitGet<'a> {
    data: &'a [u8],
    current: u64,
    position: usize,
}

const BIT_MASKS: [u32; 33] = [
    0x0000_0000,
    0x0000_0001,
    0x0000_0003,
    0x0000_0007,
    0x0000_000f,
    0x0000_001f,
    0x0000_003f,
    0x0000_007f,
    0x0000_00ff,
    0x0000_01ff,
    0x0000_03ff,
    0x0000_07ff,
    0x0000_0fff,
    0x0000_1fff,
    0x0000_3fff,
    0x0000_7fff,
    0x0000_ffff,
    0x0001_ffff,
    0x0003_ffff,
    0x0007_ffff,
    0x000f_ffff,
    0x001f_ffff,
    0x003f_ffff,
    0x007f_ffff,
    0x00ff_ffff,
    0x01ff_ffff,
    0x03ff_ffff,
    0x07ff_ffff,
    0x0fff_ffff,
    0x1fff_ffff,
    0x3fff_ffff,
    0x7fff_ffff,
    0xffff_ffff,
];

macro_rules! gen_read_unchecked {
    ($name:ident, $t:ty) => (
    #[inline(always)]
    pub fn $name(&mut self) -> $t {
        let bits = ::std::mem::size_of::<$t>() * 8;
        if self.position <= BIT_WIDTH - bits {
            let res = (self.current >> self.position) as $t;
            self.position += bits;
            res
        } else if self.position < BIT_WIDTH {
            let shifted = self.position;
            let little = (self.current >> shifted) as $t;
            self.read_unchecked();
            let big = self.current >> self.position << (BIT_WIDTH - shifted);
            self.position += bits - (BIT_WIDTH - shifted);
            (big as $t) + little
        } else {
            self.read_unchecked();
            let res = (self.current >> self.position) as $t;
            self.position += bits;
            res
        }
    });
}

macro_rules! gen_read {
    ($name:ident, $t:ty) => (
    #[inline(always)]
    pub fn $name(&mut self) -> Option<$t> {
        let bits = ::std::mem::size_of::<$t>() * 8;
        if self.position <= BIT_WIDTH - bits {
            let res = (self.current >> self.position) as $t;
            self.position += bits;
            Some(res)
        } else if self.position < BIT_WIDTH {
            let shifted = self.position;
            let little = (self.current >> shifted) as $t;
            self.read().map(|_| {
                let big = self.current >> self.position << (BIT_WIDTH - shifted);
                self.position += bits - (BIT_WIDTH - shifted);
                (big as $t) + little
            })
        } else {
            self.read().map(|_| {
                let res = (self.current >> self.position) as $t;
                self.position += bits;
                res
            })
        }
    });
}

const BYTE_WIDTH: usize = ::std::mem::size_of::<u64>();
const BIT_WIDTH: usize = BYTE_WIDTH * 8;

impl<'a> BitGet<'a> {
    /// Creates a bitstream from a byte slice
    pub fn new(data: &'a [u8]) -> BitGet<'a> {
        BitGet {
            data,
            current: 0,
            position: BIT_WIDTH,
        }
    }

    gen_read!(read_i8, i8);
    gen_read!(read_u8, u8);
    gen_read!(read_i16, i16);
    gen_read!(read_u16, u16);
    gen_read!(read_i32, i32);
    gen_read!(read_u32, u32);
    gen_read!(read_i64, i64);
    gen_read!(read_u64, u64);

    gen_read_unchecked!(read_i8_unchecked, i8);
    gen_read_unchecked!(read_u8_unchecked, u8);
    gen_read_unchecked!(read_i16_unchecked, i16);
    gen_read_unchecked!(read_u16_unchecked, u16);
    gen_read_unchecked!(read_i32_unchecked, i32);
    gen_read_unchecked!(read_u32_unchecked, u32);
    gen_read_unchecked!(read_i64_unchecked, i64);
    gen_read_unchecked!(read_u64_unchecked, u64);

    #[inline(always)]
    pub fn read_i32_bits_unchecked(&mut self, bits: i32) -> i32 {
        self.read_u32_bits_unchecked(bits) as i32
    }

    #[inline(always)]
    pub fn read_i32_bits(&mut self, bits: i32) -> Option<i32> {
        self.read_u32_bits(bits).map(|x| x as i32)
    }

    /// Assumes that the number of bits are available in the bitstream and reads them into a u32
    #[inline(always)]
    pub fn read_u32_bits_unchecked(&mut self, bits: i32) -> u32 {
        let bts = bits as usize;
        if self.position <= BIT_WIDTH - bts {
            let res = ((self.current >> self.position) as u32) & BIT_MASKS[bts];
            self.position += bts;
            res
        } else if self.position < BIT_WIDTH {
            let shifted = self.position;
            let little = (self.current >> shifted) as u32;
            self.read_unchecked();
            let had_read = BIT_WIDTH - shifted;
            let to_read = bts - had_read;
            let big =
                ((self.current >> self.position << had_read) as u32) & BIT_MASKS[bts];
            self.position += to_read;
            big + little
        } else {
            self.read_unchecked();
            let res = ((self.current >> self.position) as u32) & BIT_MASKS[bts];
            self.position += bts;
            res
        }
    }

    /// If the number of bits are available from the bitstream, read them into a u32
    #[inline(always)]
    pub fn read_u32_bits(&mut self, bits: i32) -> Option<u32> {
        let bts = bits as usize;
        if self.position <= BIT_WIDTH - bts {
            let res = ((self.current >> self.position) as u32) & BIT_MASKS[bts];
            self.position += bts;
            Some(res)
        } else if self.position < BIT_WIDTH {
            let shifted = self.position;
            let little = (self.current >> shifted) as u32;
            self.read().map(|_| {
                let had_read = BIT_WIDTH - shifted;
                let to_read = bts - had_read;
                let big =
                    ((self.current >> self.position << had_read) as u32) & BIT_MASKS[bts];
                self.position += to_read;
                big + little
            })
        } else {
            self.read().map(|_| {
                let res = ((self.current >> self.position) as u32) & BIT_MASKS[bts];
                self.position += bts;
                res
            })
        }
    }

    /// Returns if the bitstream has no more bits left
    ///
    /// ```rust
    /// # use bitter::BitGet;
    /// let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
    /// assert_eq!(bitter.is_empty(), false);
    /// assert_eq!(bitter.read_u16_unchecked(), 0b0101_0101_1010_1010);
    /// assert_eq!(bitter.is_empty(), true);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.position == BIT_WIDTH
    }

    /// Approximately the number of bytes left (an underestimate). This is the preferred method
    /// when guarding against multiple unchecked reads. Currently the underestimate is capped at 4
    /// bytes, though this may be subject to change
    ///
    /// ```rust
    /// # use bitter::BitGet;
    /// let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
    /// assert_eq!(bitter.approx_bytes_remaining(), 2);
    /// assert_eq!(bitter.read_bit_unchecked(), false);
    /// assert_eq!(bitter.approx_bytes_remaining(), 0);
    /// ```
    pub fn approx_bytes_remaining(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of bits left in the bitstream (exact)
    ///
    /// ```rust
    /// # use bitter::BitGet;
    /// let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
    /// assert_eq!(bitter.bits_remaining(), 16);
    /// assert_eq!(bitter.read_bit_unchecked(), false);
    /// assert_eq!(bitter.bits_remaining(), 15);
    /// ```
    pub fn bits_remaining(&self) -> usize {
        (BIT_WIDTH - self.position) + self.data.len() * 8
    }

    /// Advances bitstream to the next section if availablet. Don't assume that `position` is zero
    /// after this method, as the tail of the stream is packed into the highest bits.
    #[inline(always)]
    fn read(&mut self) -> Option<()> {
        if self.data.len() < BYTE_WIDTH {
            if self.data.is_empty() {
                None
            } else {
                self.position = BIT_WIDTH - (self.data.len() * 8);
                self.current = 0;
                for i in 0..self.data.len() {
                    self.current += u64::from(self.data[i]) << (i * 8)
                }
                self.current <<= 8 * (BYTE_WIDTH - self.data.len());
                self.data = &self.data[self.data.len()..];
                Some(())
            }
        } else {
            self.current = LittleEndian::read_u64(self.data);
            self.position = 0;
            self.data = &self.data[BYTE_WIDTH..];
            Some(())
        }
    }

    /// Advances bitstream to the next section. Will panic if no more data is present. Don't assume
    /// that `position` is zero after this method, as the tail of the stream is packed into the
    /// highest bits.
    #[inline(always)]
    fn read_unchecked(&mut self) {
        if self.data.len() < BYTE_WIDTH {
            if self.data.is_empty() {
                panic!("Unchecked read when no data")
            } else {
                self.position = BIT_WIDTH - (self.data.len() * 8);
                self.current = 0;
                for i in 0..self.data.len() {
                    self.current += u64::from(self.data[i]) << (i * 8)
                }
                self.current <<= 8 * (BYTE_WIDTH - self.data.len());
                self.data = &self.data[self.data.len()..];
            }
        } else {
            self.current = LittleEndian::read_u64(self.data);
            self.position = 0;
            self.data = &self.data[BYTE_WIDTH..];
        }
    }

    /// Reads a bit from the bitstream if available
    ///
    /// ```rust
    /// # use bitter::BitGet;
    /// let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
    /// assert_eq!(bitter.read_bit(), Some(false));
    /// ```
    #[inline(always)]
    pub fn read_bit(&mut self) -> Option<bool> {
        self.ensure_current().map(|_| {
            let res = self.current & (1 << self.position);
            self.position += 1;
            res != 0
        })
    }

    fn ensure_current(&mut self) -> Option<()> {
        if self.position == BIT_WIDTH {
            self.read()
        } else {
            Some(())
        }
    }

    /// *Assumes* there is at least one bit left in the stream.
    ///
    /// ```rust
    /// # use bitter::BitGet;
    /// let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
    /// assert_eq!(bitter.read_bit_unchecked(), false);
    /// ```
    #[inline(always)]
    pub fn read_bit_unchecked(&mut self) -> bool {
        if self.position == BIT_WIDTH {
            self.read_unchecked();
        }

        let res = self.current & (1 << self.position);
        self.position += 1;
        res != 0
    }

    /// Reads a `f32` from the bitstream if available
    #[inline(always)]
    pub fn read_f32(&mut self) -> Option<f32> {
        self.read_u32().map(f32::from_bits)
    }

    /// Reads a `f32` from the bitstream
    #[inline(always)]
    pub fn read_f32_unchecked(&mut self) -> f32 {
        f32::from_bits(self.read_u32_unchecked())
    }

    /// Reads a value that takes up at most `bits` bits and doesn't exceed `max`. This function
    /// *assumes* that `max` has the same bitwidth as `bits`. It doesn't make sense to call this
    /// function `bits = 8` and `max = 30`, you'd change your argument to `bits = 5`. If `bits` are
    /// not available return `None`
    ///
    /// ```rust
    /// # use bitter::BitGet;
    /// // Reads 5 bits or stops if the 5th bit would send the accumulator over 20
    /// let mut bitter = BitGet::new(&[0b1111_1000]);
    /// assert_eq!(bitter.read_bits_max(5, 20), Some(8));
    /// ```
    #[inline(always)]
    pub fn read_bits_max(&mut self, bits: i32, max: i32) -> Option<u32> {
        self.read_u32_bits(bits - 1).and_then(|data| {
            let max = max as u32;
            let up = data + (1 << (bits - 1));
            if up >= max {
                Some(data)
            } else {
                // Check the next bit
                self.read_bit().map(|x| if x { up } else { data })
            }
        })
    }

    /// Reads a value that takes up at most `bits` bits and doesn't exceed `max`. This function
    /// *assumes* that `max` has the same bitwidth as `bits`. It doesn't make sense to call this
    /// function `bits = 8` and `max = 30`, you'd change your argument to `bits = 5`
    ///
    /// ```rust
    /// # use bitter::BitGet;
    /// // Reads 5 bits or stops if the 5th bit would send the accumulator over 20
    /// let mut bitter = BitGet::new(&[0b1111_1000]);
    /// assert_eq!(bitter.read_bits_max(5, 20), Some(8));
    /// ```
    #[inline(always)]
    pub fn read_bits_max_unchecked(&mut self, bits: i32, max: i32) -> u32 {
        let data = self.read_u32_bits_unchecked(bits - 1);
        let max = max as u32;

        // If the next bit is on, what would our value be
        let up = data + (1 << (bits - 1));

        // If we have the potential to equal or exceed max don't read the next bit, else read the
        // next bit
        if up >= max || !self.read_bit_unchecked() {
            data
        } else {
            up
        }
    }

    /// If the next bit is available and on, decode the next chunk of data (which can return None).
    /// The return value can be one of the following:
    ///
    /// - None: Not enough data was available
    /// - Some(None): Bit was off so data not decoded
    /// - Some(x): Bit was on and data was decoded
    ///
    /// ```rust
    /// # use bitter::BitGet;
    /// let mut bitter = BitGet::new(&[0xff, 0x04]);
    /// assert_eq!(bitter.if_get(BitGet::read_u8), Some(Some(0x7f)));
    /// assert_eq!(bitter.if_get(BitGet::read_u8), Some(None));
    /// assert_eq!(bitter.if_get(BitGet::read_u8), None);
    /// ```
    #[cfg_attr(feature = "cargo-clippy", allow(option_option))]
    pub fn if_get<T, F>(&mut self, mut f: F) -> Option<Option<T>>
    where
        F: FnMut(&mut Self) -> Option<T>,
    {
        self.read_bit()
            .and_then(|bit| if bit { f(self).map(Some) } else { Some(None) })
    }

    /// If the next bit is available and on, decode the next chunk of data.  The return value can
    /// be one of the following:
    ///
    /// - Some(None): Bit was off so data not decoded
    /// - Some(x): Bit was on and data was decoded
    ///
    /// ```rust
    /// # use bitter::BitGet;
    /// let mut bitter = BitGet::new(&[0xff, 0x04]);
    /// assert_eq!(bitter.if_get_unchecked(BitGet::read_u8_unchecked), Some(0x7f));
    /// assert_eq!(bitter.if_get_unchecked(BitGet::read_u8_unchecked), None);
    /// ```
    /// # Panics
    ///
    /// Will panic if no data is left for the bit or for the data to be decoded.
    pub fn if_get_unchecked<T, F>(&mut self, mut f: F) -> Option<T>
    where
        F: FnMut(&mut Self) -> T,
    {
        if self.read_bit_unchecked() {
            Some(f(self))
        } else {
            None
        }
    }

    /// If the number of requested bytes are available return them to the client. Since the current
    /// bit position may not be byte aligned, return an owned vector of all the bits shifted
    /// appropriately.
    ///
    /// ```rust
    /// # use bitter::BitGet;
    /// let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
    /// assert_eq!(bitter.read_bit_unchecked(), false);
    /// assert_eq!(bitter.read_bytes(1), Some(vec![0b1101_0101]));
    /// ```
    pub fn read_bytes(&mut self, bytes: i32) -> Option<Vec<u8>> {
        let off = if self.position % 8 == 0 { 0 } else { 1 };
        let bytes_in_position = BYTE_WIDTH - self.position / 8;
        let bts = bytes as usize;
        if (bytes_in_position - off) + self.data.len() < bts {
            None
        } else {
            let mut res = Vec::with_capacity(bts);
            for _ in 0..bts {
                res.push(self.read_u8_unchecked());
            }
            Some(res)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BitGet;

    #[test]
    fn test_bit_reads() {
        let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
        assert_eq!(bitter.approx_bytes_remaining(), 2);
        assert_eq!(bitter.bits_remaining(), 16);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.approx_bytes_remaining(), 0);
        assert_eq!(bitter.bits_remaining(), 15);
        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.read_bit().unwrap(), true);

        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);

        assert_eq!(bitter.read_bit(), None);
    }

    #[test]
    fn test_bit_unchecked_reads() {
        let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);

        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);

        assert_eq!(bitter.read_bit(), None);
    }

    #[test]
    fn test_bit_unchecked_bits_reads() {
        let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);

        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);

        assert_eq!(bitter.read_bit(), None);
    }

    #[test]
    fn test_bit_bits_reads() {
        let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));

        assert_eq!(bitter.read_u32_bits(1), None);
    }

    #[test]
    fn test_read_bytes() {
        let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
        assert_eq!(bitter.read_bytes(2), Some(vec![0b1010_1010, 0b0101_0101]));

        let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bytes(2), None);
        assert_eq!(bitter.read_bytes(1), Some(vec![0b1101_0101]));
    }

    #[test]
    fn test_u8_reads() {
        let mut bitter = BitGet::new(&[0xff, 0xfe, 0xfa, 0xf7, 0xf5, 0xf0, 0xb1, 0xb2]);
        assert_eq!(bitter.read_u8(), Some(0xff));
        assert_eq!(bitter.read_u8(), Some(0xfe));
        assert_eq!(bitter.read_u8(), Some(0xfa));
        assert_eq!(bitter.read_u8(), Some(0xf7));
        assert_eq!(bitter.read_u8(), Some(0xf5));
        assert_eq!(bitter.read_u8(), Some(0xf0));
        assert_eq!(bitter.read_u8(), Some(0xb1));
        assert_eq!(bitter.read_u8(), Some(0xb2));
        assert_eq!(bitter.read_u8(), None);
    }

    #[test]
    fn test_u8_unchecked_reads() {
        let mut bitter = BitGet::new(&[0xff, 0xfe, 0xfa, 0xf7, 0xf5, 0xf0, 0xb1, 0xb2]);
        assert_eq!(bitter.read_u8_unchecked(), 0xff);
        assert_eq!(bitter.read_u8_unchecked(), 0xfe);
        assert_eq!(bitter.read_u8_unchecked(), 0xfa);
        assert_eq!(bitter.read_u8_unchecked(), 0xf7);
        assert_eq!(bitter.read_u8_unchecked(), 0xf5);
        assert_eq!(bitter.read_u8_unchecked(), 0xf0);
        assert_eq!(bitter.read_u8_unchecked(), 0xb1);
        assert_eq!(bitter.read_u8_unchecked(), 0xb2);
        assert_eq!(bitter.read_u8(), None);
    }

    #[test]
    fn test_u64_reads() {
        let mut bitter = BitGet::new(&[
            0xff, 0xfe, 0xfa, 0xf7, 0xf5, 0xf0, 0xb1, 0xb2, 0x01, 0xff, 0xfe, 0xfa, 0xf7, 0xf5,
            0xf0, 0xb1, 0xb3,
        ]);
        assert_eq!(bitter.read_u64(), Some(0xb2b1_f0f5_f7fa_feff));
        assert_eq!(bitter.read_u8(), Some(0x01));
        assert_eq!(bitter.read_u64(), Some(0xb3b1_f0f5_f7fa_feff));
    }

    #[test]
    fn test_u64_unchecked_reads() {
        let mut bitter = BitGet::new(&[
            0xff, 0xfe, 0xfa, 0xf7, 0xf5, 0xf0, 0xb1, 0xb2, 0x01, 0xff, 0xfe, 0xfa, 0xf7, 0xf5,
            0xf0, 0xb1, 0xb3,
        ]);
        assert_eq!(bitter.read_u64_unchecked(), 0xb2b1_f0f5_f7fa_feff);
        assert_eq!(bitter.read_u8_unchecked(), 0x01);
        assert_eq!(bitter.read_u64_unchecked(), 0xb3b1_f0f5_f7fa_feff);
    }

    #[test]
    fn test_u32_reads() {
        let mut bitter = BitGet::new(&[
            0xff,
            0x00,
            0xab,
            0xcd,
            0b1111_1110,
            0b0000_0001,
            0b0101_0110,
            0b1001_1011,
            0b0101_0101,
        ]);
        assert_eq!(bitter.read_u32(), Some(0xcdab00ff));
        assert_eq!(bitter.read_bit(), Some(false));
        assert_eq!(bitter.read_u32(), Some(0xcdab00ff));
        assert_eq!(bitter.read_bit(), Some(false));
        assert_eq!(bitter.read_u32(), None);
    }

    #[test]
    fn test_f32_reads() {
        let mut bitter = BitGet::new(&[
            0b0111_1011,
            0b0001_0100,
            0b1010_1110,
            0b0011_1101,
            0b1111_0110,
            0b0010_1000,
            0b0101_1100,
            0b0111_1011,
            0b0000_0010,
        ]);
        assert_eq!(bitter.read_f32(), Some(0.085));
        assert_eq!(bitter.read_bit(), Some(false));
        assert_eq!(bitter.read_f32(), Some(0.085));
    }

    #[test]
    fn test_u32_bits() {
        let mut bitter = BitGet::new(&[0xff, 0xdd, 0xee, 0xff, 0xdd, 0xee]);
        assert_eq!(bitter.read_u32_bits(10), Some(0x1ff));
        assert_eq!(bitter.read_u32_bits(10), Some(0x3b7));
        assert_eq!(bitter.read_u32_bits(10), Some(0x3fe));
        assert_eq!(bitter.read_u32_bits(10), Some(0x377));
        assert_eq!(bitter.read_u32_bits(10), None);
    }

    #[test]
    fn test_u32_unchecked() {
        let mut bitter = BitGet::new(&[
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff
        ]);
        assert_eq!(bitter.read_u32_unchecked(), 0xffff_ffff);
        assert_eq!(bitter.read_u32_bits_unchecked(30), 0x3fff_ffff);
        assert_eq!(bitter.read_u32_unchecked(), 0xffff_ffff);
    }

    #[test]
    fn test_u32_bits_unchecked() {
        let mut bitter = BitGet::new(&[0xff, 0xdd, 0xee, 0xff, 0xdd, 0xee, 0xaa, 0xbb, 0xcc, 0xdd]);
        assert_eq!(bitter.read_u32_bits_unchecked(10), 0x1ff);
        assert_eq!(bitter.read_u32_bits_unchecked(10), 0x3b7);
        assert_eq!(bitter.read_u32_bits_unchecked(10), 0x3fe);
        assert_eq!(bitter.read_u32_bits_unchecked(10), 0x377);
        assert_eq!(bitter.read_u32_bits_unchecked(8), 0xee);
        assert_eq!(bitter.read_u32_bits_unchecked(8), 0xaa);
        assert_eq!(bitter.read_u32_bits_unchecked(8), 0xbb);
        assert_eq!(bitter.read_u32_bits_unchecked(8), 0xcc);
        assert_eq!(bitter.read_u32_bits_unchecked(8), 0xdd);
        assert_eq!(bitter.read_bit(), None);
    }

    #[test]
    fn test_u32_bits_unchecked2() {
        let mut bitter = BitGet::new(&[
            0x9c, 0x73, 0xce, 0x39, 0xe7, 0x9c, 0x73, 0xce, 0x39, 0xe7, 0x9c, 0x73, 0xce, 0x39,
            0xe7,
        ]);
        for _ in 0..10 {
            assert_eq!(bitter.read_u32_bits_unchecked(5), 28);
        }
    }

    #[test]
    fn test_u32_bits2() {
        let mut bitter = BitGet::new(&[
            0x9c, 0x73, 0xce, 0x39, 0xe7, 0x9c, 0x73, 0xce, 0x39, 0xe7, 0x9c, 0x73, 0xce, 0x39,
            0xe7,
        ]);
        for _ in 0..10 {
            assert_eq!(bitter.read_u32_bits(5), Some(28));
        }
    }

    #[test]
    fn test_max_read() {
        let mut bitter = BitGet::new(&[0b1111_1000]);
        assert_eq!(bitter.read_bits_max(5, 20), Some(8));

        let mut bitter = BitGet::new(&[0b1111_0000]);
        assert_eq!(bitter.read_bits_max(5, 20), Some(16));

        let mut bitter = BitGet::new(&[0b1110_0010]);
        assert_eq!(bitter.read_bits_max(5, 20), Some(2));
    }

    #[test]
    fn test_max_read_unchecked() {
        let mut bitter = BitGet::new(&[0b1111_1000]);
        assert_eq!(bitter.read_bits_max_unchecked(5, 20), 8);

        let mut bitter = BitGet::new(&[0b1111_0000]);
        assert_eq!(bitter.read_bits_max_unchecked(5, 20), 16);

        let mut bitter = BitGet::new(&[0b1110_0010]);
        assert_eq!(bitter.read_bits_max_unchecked(5, 20), 2);
    }

    #[test]
    fn test_if_get() {
        let mut bitter = BitGet::new(&[0xff, 0x04]);
        assert_eq!(bitter.if_get(BitGet::read_u8), Some(Some(0x7f)));
        assert_eq!(bitter.if_get(BitGet::read_u8), Some(None));
        assert_eq!(bitter.if_get(BitGet::read_u8), None);
    }
}
