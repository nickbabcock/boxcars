//! This example demonstrates extracting demolition events from the network data.
//!
//! This is not intended to be comprehensive example of replay data modelling.
//!
//! Example output from rlcs2.replay
//!
//! ```plain
//! Total demolitions: 7
//! t=33.75s  Kaydop  demolished  Turbopolsa
//! t=191.85s  jstn.  demolished  Scrub Killa
//! t=310.60s  Scrub Killa  demolished  jstn.
//! t=315.04s  Scrub Killa  demolished  jstn.
//! t=352.29s  Fairy Peak!  demolished  Turbopolsa
//! t=386.55s  Turbopolsa  demolished  Kaydop
//! t=388.61s  Turbopolsa  demolished  Kaydop
//! ```

use boxcars::attributes::ActiveActor;
use boxcars::{ActorId, Attribute, ObjectId, ParserBuilder, Replay};
use std::collections::HashMap;
use std::error;
use std::io::{self, Read};

struct DemoEvent {
    time: f32,
    attacker: ActorId,
    victim: ActorId,
}

fn find_object_id(replay: &Replay, name: &str) -> Option<ObjectId> {
    replay
        .objects
        .iter()
        .position(|val| val == name)
        .map(|index| boxcars::ObjectId(index as i32))
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut data = Vec::new();
    io::stdin().read_to_end(&mut data)?;

    let replay = ParserBuilder::new(&data[..])
        .on_error_check_crc()
        .must_parse_network_data()
        .parse()?;

    let pri_name_id = find_object_id(&replay, "Engine.PlayerReplicationInfo:PlayerName");
    let car_pri_id = find_object_id(&replay, "Engine.Pawn:PlayerReplicationInfo");

    // Replays use one of three demolish formats depending on game version:
    //   - ReplicatedDemolish: oldest format
    //   - ReplicatedDemolishGoalExplosion / ReplicatedDemolish_CustomFX
    //   - ReplicatedDemolishExtended: latest
    let demolish_id = find_object_id(&replay, "TAGame.Car_TA:ReplicatedDemolish");
    let demolish_fx_id = find_object_id(&replay, "TAGame.Car_TA:ReplicatedDemolishGoalExplosion");
    let demolish_custom_fx_id =
        find_object_id(&replay, "TAGame.Car_TA:ReplicatedDemolish_CustomFX");
    let demolish_extended_id = find_object_id(&replay, "TAGame.Car_TA:ReplicatedDemolishExtended");

    // car actor id -> PRI actor id
    let mut car_to_pri: HashMap<ActorId, ActorId> = HashMap::new();
    // PRI actor id -> player name
    let mut pri_to_name: HashMap<ActorId, String> = HashMap::new();
    let mut demo_events: Vec<DemoEvent> = Vec::new();

    let network = replay.network_frames.unwrap();
    for frame in &network.frames {
        for attr in &frame.updated_actors {
            if Some(attr.object_id) == car_pri_id {
                if let Attribute::ActiveActor(ActiveActor { actor, .. }) = attr.attribute {
                    if actor != ActorId(-1) {
                        car_to_pri.insert(attr.actor_id, actor);
                    }
                }
            } else if Some(attr.object_id) == pri_name_id {
                if let Attribute::String(ref name) = attr.attribute {
                    pri_to_name.insert(attr.actor_id, name.clone());
                }
            } else if Some(attr.object_id) == demolish_id {
                if let Attribute::Demolish(d) = &attr.attribute {
                    demo_events.push(DemoEvent {
                        time: frame.time,
                        attacker: d.attacker,
                        victim: d.victim,
                    });
                }
            } else if Some(attr.object_id) == demolish_fx_id
                || Some(attr.object_id) == demolish_custom_fx_id
            {
                if let Attribute::DemolishFx(d) = &attr.attribute {
                    demo_events.push(DemoEvent {
                        time: frame.time,
                        attacker: d.attacker,
                        victim: d.victim,
                    });
                }
            } else if Some(attr.object_id) == demolish_extended_id {
                if let Attribute::DemolishExtended(d) = &attr.attribute {
                    demo_events.push(DemoEvent {
                        time: frame.time,
                        attacker: d.attacker_pri.actor,
                        victim: d.victim.actor,
                    });
                }
            }
        }
    }

    // Note: car actor IDs can be recycled when a player respawns, so a demo
    // event's car actor might map to a different player by the time we resolve
    // names here. The correct fix is to eagerly resolve car_to_pri at demo
    // time, but that's complicated by attribute ordering within a frame (the
    // PRI link may appear after the demolish attribute).
    let resolve_name = |actor_id: ActorId| -> String {
        if actor_id == ActorId(-1) {
            return String::from("<environment>");
        }
        if let Some(name) = pri_to_name.get(&actor_id) {
            return name.clone();
        }
        if let Some(pri_id) = car_to_pri.get(&actor_id) {
            if let Some(name) = pri_to_name.get(pri_id) {
                return name.clone();
            }
        }
        format!("<unknown {}>", actor_id.0)
    };

    println!("Total demolitions: {}", demo_events.len());
    for event in &demo_events {
        let attacker = resolve_name(event.attacker);
        let victim = resolve_name(event.victim);
        println!("t={:.2}s  {}  demolished  {}", event.time, attacker, victim);
    }

    Ok(())
}
