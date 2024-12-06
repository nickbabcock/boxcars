use boxcars::ParserBuilder;
use std::io::{self, Read};

fn main() {
    let mut data = Vec::new();
    let mut stdin = io::stdin();
    stdin.read_to_end(&mut data).expect("to read stdin");

    let replay = ParserBuilder::new(&data)
        .always_check_crc()
        .must_parse_network_data()
        .parse();

    let replay = match replay {
        Ok(replay) => replay,
        Err(e) => {
            eprintln!("An error occurred: {}", e);
            ::std::process::exit(1);
        }
    };

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = serde_json::to_writer_pretty(&mut out, &replay);
}
