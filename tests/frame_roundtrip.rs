//! Verifies that decoded network frames (and the `Attribute` payloads within
//! them) survive a `serde_json` serialize -> deserialize round trip unchanged.
//!
//! The top-level `Replay` JSON is intentionally lossy/one-way, but the network
//! `Frame` path is now fully round-trippable so downstream tooling can persist
//! trimmed "replay clips" as fixtures and read them back into real boxcars data.

use boxcars::{Frame, ParserBuilder};
use insta::glob;
use std::fs;

#[test]
fn network_frames_roundtrip_through_json() {
    glob!("../assets/replays/good", "*.replay", |path| {
        let data = fs::read(path).unwrap();
        let replay = ParserBuilder::new(&data[..])
            .always_check_crc()
            .must_parse_network_data()
            .parse()
            .unwrap();

        let frames = match replay.network_frames {
            Some(nf) => nf.frames,
            None => return,
        };

        let json = serde_json::to_string(&frames).unwrap();
        let restored: Vec<Frame> = serde_json::from_str(&json).unwrap();

        assert_eq!(
            frames,
            restored,
            "frames did not round trip for {}",
            path.file_name().unwrap().to_string_lossy()
        );
    });
}
