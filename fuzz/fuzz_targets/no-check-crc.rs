#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate boxcars;
fuzz_target!(|data: &[u8]| {
    let _ = boxcars::ParserBuilder::new(&data).on_error_check_crc().parse();
});
