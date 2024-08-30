use boxcars::ParserBuilder;
use std::io::{self, Read};

fn main() {
    let mut data = Vec::new();
    let mut stdin = io::stdin();
    stdin.read_to_end(&mut data).expect("to read stdin");

    let replay = ParserBuilder::new(&data)
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap();

    let stdout = io::stdout();
    let mut out = stdout.lock();
    serde_json::to_writer_pretty(&mut out, &replay).unwrap();
}
