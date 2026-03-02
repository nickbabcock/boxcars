use boxcars::ParserBuilder;
use gungraun::{library_benchmark, library_benchmark_group};

#[library_benchmark]
#[bench::three381(include_bytes!("../assets/replays/good/3381.replay").as_slice())]
fn parse_crc_body(data: &[u8]) -> boxcars::Replay {
    ParserBuilder::new(data)
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap()
}

library_benchmark_group!(name = parse_crc_body_benches, benchmarks = [parse_crc_body,]);

gungraun::main!(library_benchmark_groups = parse_crc_body_benches);
