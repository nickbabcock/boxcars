#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate nom;

extern crate crc;

use nom::{HexDisplay,Needed,IResult,ErrorKind,le_i32,le_u32,le_u8,le_u16,length_value,FileProducer};
use nom::Err;
use nom::IResult::*;

struct A {
  a: u8,
  b: u8
}

named!(length_encoded,
    chain!(
        size: le_u32 ~
        crc: le_u32 ~
        data: take!(size),
        || {data}
    )
);

named!(f<&[u8],A>,
    chain!(
        header_size:  le_i32 ~
        header_crc:   le_i32 ~
        header:       take!(header_size) ~
        content_size: le_i32 ~
        content_crc:  le_i32 ~
        content:      take!(content_size),
        || {A {a: 0, b: 0}}
    )
);


#[cfg(test)]
mod tests {
    use nom::IResult::{Done, Error, Incomplete};
    use nom::Needed::Size;

    #[test]
    fn it_works() {
        let data = include_bytes!("../assets/missing-header-data.replay");
        let r = super::length_encoded(data);
        let expected : &[u8] = &data[..];
        assert_eq!(r, Incomplete(Size(3534)));
    }

    #[test]
    fn it_works2() {
        let data = include_bytes!("../assets/incomplete-header-data.replay");
        let r = super::length_encoded(data);
        let expected : &[u8] = &data[..];
        assert_eq!(r, Incomplete(Size(3534)));
    }

    #[test]
    fn it_works3() {
        let data : [u8; 0] = [];
        let r = super::length_encoded(&data);
        let expected : &[u8] = &data[..];
        assert_eq!(r, Incomplete(Size(4)));
    }

    #[test]
    fn it_works4() {
        let data = include_bytes!("../assets/missing-header-crc.replay");
        let r = super::length_encoded(data);
        let expected : &[u8] = &data[..];
        assert_eq!(r, Incomplete(Size(8)));
    }

    #[test]
    fn it_works5() {
        let data = [0, 0, 0, 0, 0, 0, 0, 0];
        let r = super::length_encoded(&data);
        let expected : &[u8] = &data[..];
        assert_eq!(r, Done(&[][..], &[][..]));
    }
}
