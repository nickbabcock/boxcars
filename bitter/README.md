# Bitter

Bitter takes a slice of byte data and reads little-endian bits platform agonistically. Bitter has
been optimized to be fast for reading 64 or fewer bits at a time, though it can still extract
an arbitrary number of bytes.

There are two main APIs available: checked and unchecked functions. A checked function will
return a `Option` that will be `None` if there is not enough bits left in the stream.
Unchecked functions, which are denoted by having "unchecked" in their name, will panic if there
is not enough data left, but happen to be ~10% faster (your numbers
will vary depending on use case).

Tips:

- Prefer checked functions for all but the most performance critical code
- Group all unchecked functions in a single block guarded by a `approx_bytes_remaining` or
  `bits_remaining` call
- Prefer `read_u8()` over `read_u32_bits_unchecked(8)` as the specialized functions have a
  slight performance edge over the generic function

## Example

```rust
use bitter::BitGet;
let mut bitter = BitGet::new(&[0xff, 0x04]);
assert_eq!(bitter.read_bit(), Some(true));
assert_eq!(bitter.read_u8(), Some(0x7f));
assert_eq!(bitter.read_u32_bits(7), Some(0x02));
```

Below, is a demonstration of guarding against potential panics:

```rust
let mut bitter = BitGet::new(&[0xff, 0x04]);
if bitter.approx_bytes_remaining() >= 2 {
    assert_eq!(bitter.read_bit_unchecked(), true);
    assert_eq!(bitter.read_u8_unchecked(), 0x7f);
    assert_eq!(bitter.read_u32_bits_unchecked(7), 0x02);
}
```

## Implementation

Currently the implementation pre-fetches 64 bit chunks so that more operations can be performed
on a single primitive type (`u64`). Pre-fetching like this allows for operations that request
4 bytes to be completed in, at best, a bit shift and mask instead of, at best, four bit
shifts and masks.

## Comparison to other libraries

Bitter is hardly the first Rust library for handling bits.
[bitstream_io](https://crates.io/crates/bitstream-io) and
[bitreader](https://crates.io/crates/bitreader) are both crates one should consider. The reason
why someone would choose bitter over those two is speed. The other libraries lack a "trust me I
know what I'm doing" API, which bitter can give you a 10x performance increase. Additionally,
some libraries favor byte aligned reads (looking at you, bitstream_io), and since 7 out of 8
bits aren't byte aligned, there is a performance hit.
