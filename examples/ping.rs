//! This example demonstrates searching the network data, associating player cars with their names
//! in the effort to track ping times. The input is consumed as stdin.

use boxcars::{ActorId, Attribute, ObjectId, ParserBuilder, Replay};
use std::collections::HashMap;
use std::error;
use std::io::{self, Read};

#[derive(Debug, Clone)]
struct PlayerPings {
    pub actor_id: ActorId,
    pub name: Option<String>,
    pub pings: Vec<u8>,
}

fn find_object_id(replay: &Replay, name: &str) -> Result<ObjectId, Box<dyn error::Error>> {
    let id = replay
        .objects
        .iter()
        .position(|val| val == name)
        .map(|index| boxcars::ObjectId(index as i32))
        .ok_or_else(|| format!("Expected {} to be present in replay", name))?;
    Ok(id)
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut data = Vec::new();
    io::stdin().read_to_end(&mut data)?;

    let replay = ParserBuilder::new(&data[..])
        .on_error_check_crc()
        .must_parse_network_data()
        .parse()?;

    // This may be super confusing, but this is what we're doing:
    //
    // The network data sees a new car:
    //
    // ```
    // {
    //   "actor_id": 15,
    //   "name_id": 15,
    //   "object_id": 250, // Archetypes.Car.Car_Default
    //   "initial_trajectory": {
    //   }
    // }
    // ```
    //
    // This new car is updated with replication info that contains a flagged attribute with actor
    // id's that contain the player name and the ping info.
    //
    // ```
    // {
    //   "actor_id": 15,
    //   "stream_id": 30,
    //   "object_id": 39, // Engine.Pawn:PlayerReplicationInfo
    //   "attribute": {
    //     "Flagged": [
    //       true,
    //       24           // actor id of player info
    //     ]
    //   }
    // }
    // ```
    //
    // Now when we see an updated attribute targeting actor id "24" OR "15", we know which car /
    // player it's updating:
    //
    // ```
    // {
    //   "actor_id": 24,   // the actor id of the car OR player info seen here.
    //   "stream_id": 34,
    //   "object_id": 153, // "Engine.PlayerReplicationInfo:PlayerName"
    //   "attribute": {
    //     "String": "Torment"
    //   }
    // }
    // ```
    //
    // We take it one step farther by tying the ping and the player name together.
    //
    // Note that this example may be incomplete as actor ids are reused when actors are deleted
    // from frames. This attempts to work around this with a vector of the latest info. This way we
    // can simplify the example by not needing to keep track of new cars / player replication info.

    let player_name_id = find_object_id(&replay, "Engine.PlayerReplicationInfo:PlayerName")?;
    let ping_id = find_object_id(&replay, "Engine.PlayerReplicationInfo:Ping")?;

    eprintln!("player_name_id: {}, ping_id: {}", player_name_id, ping_id);

    let mut actor_pings: Vec<PlayerPings> = Vec::new();
    let network = replay.network_frames.unwrap();

    for frame in network.frames {
        for attr in frame.updated_actors {
            if attr.object_id == player_name_id {
                let act_id = attr.actor_id;
                if let Attribute::String(name) = attr.attribute {
                    // Fill in the name of the latest entry with the same
                    // actor id that either has the same name or no name.
                    let entry = actor_pings.iter().rev().rposition(|x| {
                        x.actor_id == act_id && x.name.as_ref().map_or(true, |n| n == &name)
                    });
                    if let Some(position) = entry {
                        actor_pings[position].name.replace(name);
                    } else {
                        actor_pings.push(PlayerPings {
                            actor_id: attr.actor_id,
                            name: Some(name),
                            pings: vec![],
                        });
                    }
                } else {
                    return Err("expected player name to be a string")?;
                }
            } else if attr.object_id == ping_id {
                if let Attribute::Byte(ping) = attr.attribute {
                    let entry = actor_pings
                        .iter()
                        .rev()
                        .rposition(|x| x.actor_id == attr.actor_id);

                    if let Some(position) = entry {
                        actor_pings[position].pings.push(ping);
                    } else {
                        actor_pings.push(PlayerPings {
                            actor_id: attr.actor_id,
                            name: None,
                            pings: vec![ping],
                        });
                    }
                } else {
                    return Err("expected ping to be a byte")?;
                }
            }
        }
    }

    // Group the ping data by player name
    let mut pings: HashMap<String, Vec<u8>> = HashMap::new();
    for player in actor_pings {
        let key = player
            .name
            .clone()
            .unwrap_or_else(|| String::from("<unknown>"));

        pings
            .entry(key)
            .and_modify(|e| {
                e.extend_from_slice(&player.pings);
            })
            .or_insert_with(Vec::new);
    }

    for (player, pings) in &pings {
        println!("{}: {:?}", player, pings);
    }

    Ok(())
}
