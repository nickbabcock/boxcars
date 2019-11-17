//! This example of boxcars extracts all the player names found in the "PlayerStats" property of the
//! header. This property may be absent in some replays or lack players that drop or join mid-game.
//! A more foolproof approach is to scour the network data for a specific property:
//! "Engine.PlayerReplicationInfo:PlayerName". This example shows both methods. The error handling
//! demonstrated is minimal, relying on stringly typed errors. In practice, prefer a richer error
//! type.
use boxcars::{HeaderProp, ParserBuilder};
use std::error;
use std::io::{self, Read};

/// Given an array of objects (represented as a slice of key-value pairs), find all the instances
/// of the "Name" key and extract the string value
fn names_in_header(stats: &[Vec<(String, HeaderProp)>]) -> impl Iterator<Item = &str> {
    stats
        .iter()
        .flat_map(|v| v.iter())
        .filter(|(prop_name, _)| *prop_name == "Name")
        .filter_map(|(_, prop_val)| prop_val.as_string())
}

/// Given network frames and the object id to "Engine.PlayerReplicationInfo:PlayerName", comb
/// through all the attributes looking for attributes that have our object id.
fn names_in_network(frames: &[boxcars::Frame], name_attribute_id: boxcars::ObjectId) -> Vec<&str> {
    let mut names = frames
        .iter()
        .flat_map(|x| x.updated_actors.iter())
        .filter(|attr| attr.object_id == name_attribute_id)
        .filter_map(|attr| {
            // PlayerName will be a string attribute
            if let boxcars::Attribute::String(ref s) = attr.attribute {
                Some(s.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // This list will contain many duplicates so we dedup it before returning.
    names.sort();
    names.dedup();
    names
}

/// This function looks up the object id for "Engine.PlayerReplicationInfo:PlayerName". The object
/// id is the same as the index of that value in the `replay.objects` array.
fn player_name_object_id(
    replay: &boxcars::Replay,
) -> Result<boxcars::ObjectId, Box<dyn error::Error>> {
    let id = replay
        .objects
        .iter()
        .position(|val| val == "Engine.PlayerReplicationInfo:PlayerName")
        .map(|index| boxcars::ObjectId(index as i32))
        .ok_or("Expected Engine.PlayerReplicationInfo:PlayerName to be present in replay")?;
    Ok(id)
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut data = Vec::new();
    io::stdin().read_to_end(&mut data)?;

    let replay = ParserBuilder::new(&data[..])
        .on_error_check_crc()
        .ignore_network_data_on_error()
        .parse()?;

    let stats_prop = replay
        .properties
        .iter()
        .find(|(prop, _)| *prop == "PlayerStats");

    if let Some((_, stats)) = stats_prop {
        println!("Names in the header data:");
        let stats = match stats {
            HeaderProp::Array(arr) => arr,
            _ => return Err("expected player stats to be an array".into()),
        };

        let header_names = names_in_header(&stats);
        for name in header_names {
            println!("{}", name);
        }
    } else {
        println!("No player names found in the header");
    }

    if let Some(network) = replay.network_frames.as_ref() {
        println!("Names in the network data:");
        let name_attribute_id = player_name_object_id(&replay)?;
        let names = names_in_network(&network.frames, name_attribute_id);
        for name in names {
            println!("{}", name);
        }
    } else {
        println!("No player names found in the header as network data couldn't be decoded")
    }

    Ok(())
}
