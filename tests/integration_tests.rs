use boxcars::ParserBuilder;

macro_rules! frame_len_test {
    ($test_name:ident, $test_asset:expr, $frame_len:expr) => {
        #[test]
        fn $test_name() {
            let data = include_bytes!($test_asset);
            let parsing = ParserBuilder::new(&data[..])
                .always_check_crc()
                .must_parse_network_data()
                .parse();

            match parsing {
                Ok(replay) => assert_eq!(replay.network_frames.unwrap().frames.len(), $frame_len),
                Err(ref e) => panic!(format!("{}", e)),
            }
        }
    };
}

frame_len_test!(test_b0867_replay, "../assets/b0867.replay", 8599);
frame_len_test!(test_small_frames, "../assets/small-frames.replay", 231);
frame_len_test!(test_3381_replay, "../assets/3381.replay", 13320);
frame_len_test!(test_3d07e_replay, "../assets/3d07e.replay", 8727);
frame_len_test!(test_e7fb9_replay, "../assets/e7fb9.replay", 7472);
frame_len_test!(test_e4598_replay, "../assets/e4598.replay", 6898);
frame_len_test!(test_65e98_replay, "../assets/65e98.replay", 7174);
frame_len_test!(test_7083_replay, "../assets/7083.replay", 8346);
frame_len_test!(test_6688_replay, "../assets/6688.replay", 0);
frame_len_test!(test_07e9_replay, "../assets/07e9.replay", 319);
frame_len_test!(test_16d5_replay, "../assets/16d5.replay", 405);
frame_len_test!(test_551c_replay, "../assets/551c.replay", 8247);
frame_len_test!(test_2266_replay, "../assets/2266.replay", 8136);
frame_len_test!(test_rumble_body, "../assets/rumble.replay", 7744);
frame_len_test!(test_no_frames, "../assets/no-frames.replay", 0);
frame_len_test!(test_net_version, "../assets/netversion.replay", 7901);
frame_len_test!(test_159a4_replay, "../assets/159a4.replay", 7104);
frame_len_test!(test_c0bca_replay, "../assets/c0bca.replay", 7290);
frame_len_test!(test_db70_replay, "../assets/db70.replay", 9781);
frame_len_test!(test_6cc24_replay, "../assets/6cc24.replay", 7319);
frame_len_test!(test_57a6c_replay, "../assets/57a6c.replay", 378);
frame_len_test!(test_01d3e5_replay, "../assets/01d3e5.replay", 393);
frame_len_test!(test_a9df3_replay, "../assets/a9df3.replay", 330);
frame_len_test!(test_419a_replay, "../assets/419a.replay", 10183);
frame_len_test!(test_4bc3b_replay, "../assets/4bc3b.replay", 9536);
frame_len_test!(test_d52eb_replay, "../assets/d52eb.replay", 17815);
frame_len_test!(test_edbb_replay, "../assets/edbb.replay", 356);
frame_len_test!(test_7256_replay, "../assets/7256.replay", 9634);
frame_len_test!(test_5a06_replay, "../assets/5a06.replay", 515);
frame_len_test!(test_60dfe_replay, "../assets/60dfe.replay", 9737);
frame_len_test!(test_70865_replay, "../assets/70865.replay", 8912);
frame_len_test!(test_72ae1_replay, "../assets/72ae1.replay", 8545);
frame_len_test!(test_fc427_replay, "../assets/fc427.replay", 9343);
frame_len_test!(test_7f79f_replay, "../assets/7f79f.replay", 12273);
frame_len_test!(test_c23b0_replay, "../assets/c23b0.replay", 9811);
frame_len_test!(test_c4abb_replay, "../assets/c4abb.replay", 9093);
frame_len_test!(test_70204_replay, "../assets/70204.replay", 9574);
frame_len_test!(test_74936_replay, "../assets/74936.replay", 10609);
