#![no_main]

use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    let _ = boxcars::ParserBuilder::new(&data)
        .on_error_check_crc()
        .never_parse_network_data()
        .parse();
});
