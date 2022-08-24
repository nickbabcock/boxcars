use bitter::{BitReader, LittleEndianReader};

#[inline]
pub(crate) const fn bit_width(input: u64) -> u32 {
    (core::mem::size_of::<u64>() as u32) * 8 - input.leading_zeros()
}

pub(crate) trait RlBits {
    fn peek_and_consume(&mut self, bits: u32) -> u64;
    fn peek_bits_max_computed(&mut self, bits: u32, max: u64) -> u64;
    fn read_bits_max_computed(&mut self, bits: u32, max: u64) -> Option<u64>;
    fn read_u64(&mut self) -> Option<u64>;
    fn read_i64(&mut self) -> Option<i64>;
    fn if_get<T, F>(&mut self, f: F) -> Option<Option<T>>
    where
        F: FnMut(&mut Self) -> Option<T>;
}

impl<'a> RlBits for LittleEndianReader<'a> {
    #[inline]
    fn read_u64(&mut self) -> Option<u64> {
        let mut out = [0u8; 8];
        if self.read_bytes(&mut out) {
            Some(u64::from_le_bytes(out))
        } else {
            None
        }
    }

    #[inline]
    fn read_i64(&mut self) -> Option<i64> {
        self.read_u64().map(|x| x as i64)
    }

    #[inline]
    fn peek_and_consume(&mut self, bits: u32) -> u64 {
        let res = self.peek(bits);
        self.consume(bits);
        res
    }

    #[inline]
    fn if_get<T, F>(&mut self, mut f: F) -> Option<Option<T>>
    where
        F: FnMut(&mut Self) -> Option<T>,
    {
        self.read_bit()
            .and_then(|bit| if bit { f(self).map(Some) } else { Some(None) })
    }

    #[inline]
    fn peek_bits_max_computed(&mut self, bits: u32, max: u64) -> u64 {
        debug_assert!(core::cmp::max(bit_width(max) as u32, 1) == bits + 1);

        let data = self.peek_and_consume(bits);
        let up = data + (1 << bits);
        if up >= max {
            data
        } else if self.peek_and_consume(1) != 0 {
            up
        } else {
            data
        }
    }

    #[inline]
    fn read_bits_max_computed(&mut self, bits: u32, max: u64) -> Option<u64> {
        debug_assert!(core::cmp::max(bit_width(max) as u32, 1) == bits + 1);
        self.read_bits(bits).and_then(|data| {
            let up = data + (1 << bits);
            if up >= max {
                Some(data)
            } else {
                // Check the next bit
                self.read_bit().map(|x| if x { up } else { data })
            }
        })
    }
}
