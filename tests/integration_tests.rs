use boxcars::ParserBuilder;
use highway::HighwayHash;
use insta::{assert_json_snapshot, glob};
use std::{env, fs, ffi::OsString, io::BufWriter, collections::HashSet};


#[test]
fn test_replay_snapshots() {
    // Converting Strings to OsStrings because it allows for safe
    // comparisons as OsString cannot safely be converted into String
    // without losing information
    let target_replays: HashSet<OsString> = env::args()
        .skip(2)
        .filter(|arg| !arg.starts_with("--"))
        .map(OsString::from)
        .collect();

    glob!("../assets/replays/good", "*.replay", |path| {
        if target_replays.is_empty() || (
            path.file_stem().is_some_and(|x| target_replays.contains(x))
        ) {
            let data = fs::read(path).unwrap();
            let parsed = ParserBuilder::new(&data[..])
                .always_check_crc()
                .must_parse_network_data()
                .parse();
    
            let replay = match parsed {
                Ok(x) => x,
                Err(e) => panic!(
                    "failed parsing: (INSTA_GLOB_FILTER={}) {}",
                    path.file_name().unwrap().to_string_lossy(),
                    e
                ),
            };
    
            // Hash the output otherwise we'll have 2.5GB+ of snapshot data
            let hasher = highway::HighwayHasher::new(highway::Key::default());
    
            // HighwayHash is fast, but we still want to buffer writes as much as
            // possible. Makes tests run 3x faster in release mode.
            let mut writer = BufWriter::with_capacity(0x8000, hasher);
            serde_json::to_writer(&mut writer, &replay).unwrap();
            let hash = writer.into_inner().unwrap().finalize256();
            let hex = format!(
                "0x{:016x}{:016x}{:016x}{:016x}",
                hash[0], hash[1], hash[2], hash[3]
            );
    
            let snapshot = serde_json::json!({
                "frames": replay.network_frames.unwrap().frames.len(),
                "hex": hex
            });
            assert_json_snapshot!(snapshot);
        }
    });
}
