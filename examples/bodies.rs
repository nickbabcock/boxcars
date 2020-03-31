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

    let player_name_id = find_object_id(&replay, "Engine.PlayerReplicationInfo:PlayerName")?;
    let ball_id = find_object_id(&replay, "Archetypes.Ball.Ball_Default")?;
    let network = replay.network_frames.unwrap();
    let mut actors: HashMap<ActorId, ObjectId> = HashMap::new();
    let mut names: HashMap<ActorId, String> = HashMap::new();
    let mut count = 0;
    for frame in network.frames {
        for actor in frame.deleted_actors {
            if actors.remove(&actor).is_none() {
                println!("non-existent actor deleted");
            }
        }

        for actor in frame.new_actors {
            // There are a lot of reused actor ids so we just overwrite it
            actors.insert(actor.actor_id, actor.object_id);
        }

        for attr in frame.updated_actors {
            let act_id = i32::from(attr.actor_id);
            if attr.object_id == player_name_id {
                if let Attribute::String(name) = &attr.attribute {
                    println!(
                        "NAME: {} obj: {}",
                        name.clone(),
                        replay.objects[usize::from(*actors.get(&attr.actor_id).unwrap())]
                    );
                    names.insert(attr.actor_id, name.clone());
                }
            }

            if let Attribute::RigidBody(s) = attr.attribute {
                if let Some(obj_id) = actors.get(&attr.actor_id) {
                    if *obj_id == ball_id {
                        println!("ball {} body: {:?}", act_id, s);
                    }
                } else {
                    println!("unknown");
                }
                /*                if let Some(name) = names.get(&attr.actor_id) {
                    println!("known");
                } else {
                    println!("unknown");
                }*/
                count += 1;
            }
        }
    }

    println!("bodies: {}", count);
    Ok(())
}
