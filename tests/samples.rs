use boxcars::{self, ParserBuilder};

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
