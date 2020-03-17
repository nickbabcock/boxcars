use boxcars::attributes::{ActiveActor, Demolish, Pickup, RigidBody, StatEvent, Welded};
use boxcars::{
    self, ActorId, NetworkError, ParseError, ParserBuilder, Quaternion, Trajectory, Vector3f,
    Vector3i,
};

#[test]
fn test_sample1() {
    let data = include_bytes!("../assets/replays/good/3d07e.replay");
    let replay = ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap();

    let frames = replay.network_frames.unwrap().frames;

    // random usage of the API
    let new_count = frames
        .iter()
        .flat_map(|x| x.new_actors.iter())
        .filter(|x| x.actor_id.0 != x.object_id.0)
        .count();
    assert_eq!(4545, new_count);

    let sleeping_rigid_bodies = frames
        .iter()
        .flat_map(|x| x.updated_actors.iter())
        .filter(|act| match act.attribute {
            boxcars::Attribute::RigidBody(body) => body.sleeping,
            _ => false,
        })
        .count();

    assert_eq!(32, sleeping_rigid_bodies);

    let first_actor = frames
        .iter()
        .flat_map(|x| x.new_actors.iter())
        .find(|_| true)
        .unwrap();
    let first_update = frames
        .iter()
        .flat_map(|x| x.updated_actors.iter())
        .find(|_| true)
        .unwrap();

    let first_actor_id: boxcars::ActorId = first_actor.actor_id;
    assert_eq!(0, first_actor_id.0);

    let first_object_id: boxcars::ObjectId = first_actor.object_id;
    assert_eq!(26, first_object_id.0);

    let first_stream_id: boxcars::StreamId = first_update.stream_id;
    assert_eq!(31, first_stream_id.0);
}

fn extract_online_id(replay: &boxcars::Replay, user: &str) -> (u64, boxcars::attributes::RemoteId) {
    let (_, stats) = replay
        .properties
        .iter()
        .find(|(prop, _)| *prop == "PlayerStats")
        .unwrap();

    let online_id = match stats {
        boxcars::HeaderProp::Array(arr) => {
            let our_player = arr
                .iter()
                .find(|properties| {
                    properties
                        .iter()
                        .find(|(prop, val)| {
                            *prop == "Name" && *val == boxcars::HeaderProp::Str(String::from(user))
                        })
                        .is_some()
                })
                .unwrap();

            let (_, online_id) = our_player
                .iter()
                .find(|(prop, _val)| *prop == "OnlineID")
                .unwrap();
            if let boxcars::HeaderProp::QWord(oid) = online_id {
                *oid
            } else {
                panic!("unexpected property");
            }
        }
        _ => panic!("Expected array"),
    };

    let frames = &replay.network_frames.as_ref().unwrap().frames;
    let reservation = frames
        .iter()
        .flat_map(|x| {
            x.updated_actors.iter().filter_map(|x| {
                if let boxcars::Attribute::Reservation(r) = &x.attribute {
                    if r.name.as_ref().map(|x| x == user).unwrap_or(false) {
                        Some(&r.unique_id.remote_id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
        .last()
        .unwrap();

    (online_id, reservation.clone())
}

#[test]
fn test_long_psynet_id() {
    let data = include_bytes!("../assets/replays/good/d52eb.replay");
    let replay = ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap();

    let (header_oid, network_oid) = extract_online_id(&replay, "FunFactJac");
    assert_eq!(15633594671552264637, header_oid);

    if let boxcars::attributes::RemoteId::PsyNet(psy) = network_oid {
        assert_eq!(header_oid, psy.online_id);
    } else {
        panic!("Needed psynet remote_id");
    }
}

#[test]
fn test_short_psynet_id() {
    let data = include_bytes!("../assets/replays/good/60dfe.replay");
    let replay = ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap();

    let (header_oid, network_oid) = extract_online_id(&replay, "Shope");
    assert_eq!(18091002852234862424, header_oid);

    if let boxcars::attributes::RemoteId::PsyNet(psy) = network_oid {
        assert_eq!(header_oid, psy.online_id);
    } else {
        panic!("Needed psynet remote_id");
    }
}

#[test]
fn test_switch_id() {
    let data = include_bytes!("../assets/replays/good/7083.replay");
    let replay = ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap();

    let (header_oid, network_oid) = extract_online_id(&replay, "Leon");
    assert_eq!(13979735202661301154, header_oid);

    if let boxcars::attributes::RemoteId::Switch(switch) = network_oid {
        assert_eq!(header_oid, switch.online_id);
    } else {
        panic!("Needed switch remote_id");
    }
}

#[test]
fn test_long_ps4_id() {
    let data = include_bytes!("../assets/replays/good/159a4.replay");
    let replay = ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap();

    let (header_oid, network_oid) = extract_online_id(&replay, "SyCoz-Chaos");
    assert_eq!(3373421750759248985, header_oid);

    if let boxcars::attributes::RemoteId::PlayStation(ps) = network_oid {
        assert_eq!(header_oid, ps.online_id);
        assert_eq!(ps.name, "SyCoz-Chaos");
    } else {
        panic!("Needed playstation remote_id");
    }
}

#[test]
fn test_short_ps4_id() {
    let data = include_bytes!("../assets/replays/good/3d07e.replay");
    let replay = ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap();

    let (header_oid, network_oid) = extract_online_id(&replay, "TheGoldenGarp");
    assert_eq!(0, header_oid);

    if let boxcars::attributes::RemoteId::PlayStation(ps) = network_oid {
        assert_eq!(1, ps.online_id);
        assert_eq!(ps.name, "TheGoldenGarp");
    } else {
        panic!("Needed playstation remote_id");
    }
}

#[test]
fn test_preserve_endian() {
    let data = include_bytes!("../assets/replays/good/fc427.replay");
    let replay = ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap();

    let frames = &replay.network_frames.as_ref().unwrap().frames;
    let new_paints: Vec<u32> = frames
        .iter()
        .flat_map(|x| x.updated_actors.iter())
        .filter_map(|ac| match &ac.attribute {
            boxcars::Attribute::LoadoutsOnline(x) => Some(x.blue.iter().flat_map(|pr| pr.iter())),
            _ => None,
        })
        .flat_map(|x| x)
        .filter_map(|x| match x.value {
            boxcars::attributes::ProductValue::NewPaint(p) => Some(p),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(*new_paints.get(0).unwrap(), 11);
}

#[test]
fn test_error_extraction() {
    let data = include_bytes!("../assets/replays/bad/fuzz-large-object-id.replay");
    let err = ParserBuilder::new(&data[..])
        .never_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_err();

    let ne = match err {
        ParseError::NetworkError(e) => e,
        _ => panic!("Expecting network error"),
    };

    match *ne {
        NetworkError::ObjectIdOutOfRange(obj) => {
            assert!(obj.0 != 0);
        }
        x => panic!("Expecting object id out of range. not {:?}", x),
    }
}

#[test]
fn test_quaternions() {
    let data = include_bytes!("../assets/replays/good/01d3e5.replay");
    let replay = ParserBuilder::new(&data[..])
        .never_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap();

    let frames = &replay.network_frames.as_ref().unwrap().frames;

    let trajectories: Vec<Trajectory> = frames
        .iter()
        .flat_map(|x| x.new_actors.iter())
        .map(|x| x.initial_trajectory)
        .collect();

    let bodies: Vec<&RigidBody> = frames
        .iter()
        .flat_map(|x| x.updated_actors.iter())
        .filter_map(|x| {
            if let boxcars::Attribute::RigidBody(r) = &x.attribute {
                Some(r)
            } else {
                None
            }
        })
        .collect();

    // values cross referenced from bakkes
    assert_eq!(
        bodies[1].rotation,
        Quaternion {
            x: -0.004410246,
            y: 0.0018207438,
            z: 0.923867,
            w: 0.38268402
        }
    );

    assert_eq!(
        bodies[1].location,
        Vector3f {
            x: 1951.99,
            y: -2463.98,
            z: 17.01,
        }
    );

    assert_eq!(
        bodies[1].linear_velocity.unwrap(),
        Vector3f {
            x: -0.07,
            y: 0.07,
            z: 8.32,
        }
    );

    assert_eq!(
        bodies[1].angular_velocity.unwrap(),
        Vector3f {
            x: -0.04,
            y: -0.02,
            z: 0.0,
        }
    );

    assert_eq!(
        trajectories[7].location.unwrap(),
        Vector3i {
            x: 1952,
            y: -2464,
            z: 17,
        }
    );

    let events: Vec<StatEvent> = frames
        .iter()
        .flat_map(|x| {
            x.updated_actors.iter().filter_map(|x| {
                if let boxcars::Attribute::StatEvent(x) = x.attribute {
                    Some(x)
                } else {
                    None
                }
            })
        })
        .collect();
    assert_eq!(events[1].object_id, -1);
}

#[test]
fn test_compressed_quaternions() {
    let data = include_bytes!("../assets/replays/good/07e9.replay");
    let replay = ParserBuilder::new(&data[..])
        .never_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap();

    let frames = &replay.network_frames.as_ref().unwrap().frames;
    let rotations: Vec<Quaternion> = frames
        .iter()
        .flat_map(|x| {
            x.updated_actors.iter().filter_map(|x| {
                if let boxcars::Attribute::RigidBody(r) = &x.attribute {
                    Some(r.rotation)
                } else {
                    None
                }
            })
        })
        .collect();

    // value cross referenced with bakkes
    assert_eq!(
        rotations[1],
        Quaternion {
            x: -1.0000305,
            y: 0.4998932,
            z: -1.0000305,
            w: 0.0
        }
    )
}

#[test]
fn test_active_actor() {
    let data = include_bytes!("../assets/replays/good/3d07e.replay");
    let replay = ParserBuilder::new(&data[..])
        .never_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap();

    let frames = &replay.network_frames.as_ref().unwrap().frames;
    let active_actors: Vec<ActiveActor> = frames
        .iter()
        .flat_map(|x| {
            x.updated_actors.iter().filter_map(|x| {
                if let boxcars::Attribute::ActiveActor(x) = x.attribute {
                    Some(x)
                } else {
                    None
                }
            })
        })
        .collect();

    assert_eq!(active_actors[73].actor, ActorId(-1));
}

#[test]
fn test_rumble_actor_id() {
    let data = include_bytes!("../assets/replays/good/rumble.replay");
    let replay = ParserBuilder::new(&data[..])
        .never_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap();

    let frames = &replay.network_frames.as_ref().unwrap().frames;
    let welds: Vec<Welded> = frames
        .iter()
        .flat_map(|x| {
            x.updated_actors.iter().filter_map(|x| {
                if let boxcars::Attribute::Welded(x) = x.attribute {
                    Some(x)
                } else {
                    None
                }
            })
        })
        .collect();

    assert_eq!(welds[5].actor, ActorId(-1));

    let demolish: Vec<&Box<Demolish>> = frames
        .iter()
        .flat_map(|x| {
            x.updated_actors.iter().filter_map(|x| {
                if let boxcars::Attribute::Demolish(ref x) = x.attribute {
                    Some(x)
                } else {
                    None
                }
            })
        })
        .collect();
    assert_eq!(demolish[3].attacker, ActorId(-1));

    let pickups: Vec<Pickup> = frames
        .iter()
        .flat_map(|x| {
            x.updated_actors.iter().filter_map(|x| {
                if let boxcars::Attribute::Pickup(x) = x.attribute {
                    Some(x)
                } else {
                    None
                }
            })
        })
        .collect();
    assert_eq!(pickups[264].instigator, Some(ActorId(-1)));
}
