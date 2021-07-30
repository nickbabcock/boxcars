//! This example of boxcars looks at the header of replays given as command line arguments and
//! counts how often a properties occurs across them.
use boxcars::{HeaderProp, ParserBuilder};
use std::borrow::Cow;
use std::collections::HashMap;
use std::error;
use std::fs::File;
use std::io::Read;

fn count_properties(
    props: &[(String, HeaderProp)],
    counter: &mut HashMap<String, usize>,
    prefix: &str,
) {
    for (key, prop) in props.iter() {
        let new_prefix = if prefix == "" {
            Cow::Borrowed(key)
        } else {
            Cow::Owned(format!("{}:{}", prefix, key))
        };

        let k = match prop {
            HeaderProp::Array(inner) => {
                for p in inner.iter() {
                    count_properties(p, counter, &new_prefix);
                }
                format!("{}:array", new_prefix)
            }
            HeaderProp::Bool(_) => format!("{}:bool", new_prefix),
            HeaderProp::Byte { .. } => format!("{}:byte", new_prefix),
            HeaderProp::Float(_) => format!("{}:float", new_prefix),
            HeaderProp::Int(_) => format!("{}:int", new_prefix),
            HeaderProp::Name(_) => format!("{}:name", new_prefix),
            HeaderProp::QWord(_) => format!("{}:qword", new_prefix),
            HeaderProp::Str(_) => format!("{}:str", new_prefix),
        };

        *counter.entry(k).or_default() += 1;
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut counter: HashMap<String, usize> = HashMap::new();
    let mut buffer: Vec<u8> = Vec::new();
    let mut files = 0;
    for file in std::env::args().skip(1) {
        let rs = File::open(&file).and_then(|mut f| f.read_to_end(&mut buffer));
        if let Err(e) = rs {
            println!("unable to read file {}: {}", &file, e);
            continue;
        }

        let replay = ParserBuilder::new(&buffer)
            .never_check_crc()
            .never_parse_network_data()
            .parse();

        let replay = if let Err(e) = replay {
            println!("unable to parse file {}: {}", &file, e);
            continue;
        } else {
            replay.unwrap()
        };

        files += 1;
        count_properties(&replay.properties, &mut counter, "");

        println!("Parsed: {}", file);
        buffer.clear();
    }

    println!("Parsed {} files", files);
    let mut counts = counter.iter().collect::<Vec<_>>();
    counts.sort();
    for (key, count) in counts.iter() {
        println!("{}: {}", key, count);
    }

    Ok(())
}
