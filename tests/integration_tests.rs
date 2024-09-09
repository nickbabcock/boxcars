use boxcars::ParserBuilder;
use highway::HighwayHash;
use insta::{assert_json_snapshot, glob};
use std::fs;

#[test]
fn test_replay_snapshots() {
    glob!("../assets/replays/good", "*.replay", |path| {
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
        let mut hasher = highway::HighwayHasher::new(highway::Key::default());
        serde_json::to_writer(&mut hasher, &replay).unwrap();
        let hash = hasher.finalize256();
        let out = hash
            .iter()
            .map(|x| format!("{:016x}", x))
            .collect::<String>();
        let hex = format!("0x{}", out);

        let snapshot = serde_json::json!({
            "frames": replay.network_frames.unwrap().frames.len(),
            "hex": hex
        });
        assert_json_snapshot!(snapshot);
    });
}
