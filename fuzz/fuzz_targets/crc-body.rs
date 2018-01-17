#![no_main]
extern crate boxcars;
#[macro_use]
extern crate libfuzzer_sys;

fuzz_target!(|data: &[u8]| {
    let _ = boxcars::ParserBuilder::new(&data)
        .always_check_crc()
        .must_parse_network_data()
        .parse();
});
