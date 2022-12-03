#![cfg_attr(rustfmt, rustfmt::skip)]
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
                Err(ref e) => panic!("{}", e),
            }
        }
    };
}

frame_len_test!(test_b0867_replay, "../assets/replays/good/b0867.replay", 8599);
frame_len_test!(test_small_frames, "../assets/replays/good/small-frames.replay", 231);
frame_len_test!(test_3381_replay, "../assets/replays/good/3381.replay", 13320);
frame_len_test!(test_3d07e_replay, "../assets/replays/good/3d07e.replay", 8727);
frame_len_test!(test_e7fb9_replay, "../assets/replays/good/e7fb9.replay", 7472);
frame_len_test!(test_e4598_replay, "../assets/replays/good/e4598.replay", 6898);
frame_len_test!(test_65e98_replay, "../assets/replays/good/65e98.replay", 7174);
frame_len_test!(test_7083_replay, "../assets/replays/good/7083.replay", 8346);
frame_len_test!(test_6688_replay, "../assets/replays/good/6688.replay", 0);
frame_len_test!(test_07e9_replay, "../assets/replays/good/07e9.replay", 319);
frame_len_test!(test_16d5_replay, "../assets/replays/good/16d5.replay", 405);
frame_len_test!(test_551c_replay, "../assets/replays/good/551c.replay", 8247);
frame_len_test!(test_2266_replay, "../assets/replays/good/2266.replay", 8136);
frame_len_test!(test_rumble_body, "../assets/replays/good/rumble.replay", 7744);
frame_len_test!(test_no_frames, "../assets/replays/good/no-frames.replay", 0);
frame_len_test!(test_net_version, "../assets/replays/good/netversion.replay", 7901);
frame_len_test!(test_159a4_replay, "../assets/replays/good/159a4.replay", 7104);
frame_len_test!(test_c0bca_replay, "../assets/replays/good/c0bca.replay", 7290);
frame_len_test!(test_db70_replay, "../assets/replays/good/db70.replay", 9781);
frame_len_test!(test_6cc24_replay, "../assets/replays/good/6cc24.replay", 7319);
frame_len_test!(test_57a6c_replay, "../assets/replays/good/57a6c.replay", 378);
frame_len_test!(test_01d3e5_replay, "../assets/replays/good/01d3e5.replay", 393);
frame_len_test!(test_a9df3_replay, "../assets/replays/good/a9df3.replay", 330);
frame_len_test!(test_419a_replay, "../assets/replays/good/419a.replay", 10183);
frame_len_test!(test_4bc3b_replay, "../assets/replays/good/4bc3b.replay", 9536);
frame_len_test!(test_d52eb_replay, "../assets/replays/good/d52eb.replay", 17815);
frame_len_test!(test_edbb_replay, "../assets/replays/good/edbb.replay", 356);
frame_len_test!(test_7256_replay, "../assets/replays/good/7256.replay", 9634);
frame_len_test!(test_5a06_replay, "../assets/replays/good/5a06.replay", 515);
frame_len_test!(test_60dfe_replay, "../assets/replays/good/60dfe.replay", 9737);
frame_len_test!(test_70865_replay, "../assets/replays/good/70865.replay", 8912);
frame_len_test!(test_72ae1_replay, "../assets/replays/good/72ae1.replay", 8545);
frame_len_test!(test_fc427_replay, "../assets/replays/good/fc427.replay", 9343);
frame_len_test!(test_7f79f_replay, "../assets/replays/good/7f79f.replay", 12273);
frame_len_test!(test_c23b0_replay, "../assets/replays/good/c23b0.replay", 9811);
frame_len_test!(test_c4abb_replay, "../assets/replays/good/c4abb.replay", 9093);
frame_len_test!(test_70204_replay, "../assets/replays/good/70204.replay", 9574);
frame_len_test!(test_74936_replay, "../assets/replays/good/74936.replay", 10609);
frame_len_test!(test_1ec9_replay, "../assets/replays/good/1ec9.replay", 332);
frame_len_test!(test_9a2cd_replay, "../assets/replays/good/9a2cd.replay", 2616);
frame_len_test!(test_9e35b_replay, "../assets/replays/good/9e35b.replay", 12859);
frame_len_test!(test_21a81_replay, "../assets/replays/good/21a81.replay", 13539);
frame_len_test!(test_d1d5_replay, "../assets/replays/good/d1d5.replay", 4454);
frame_len_test!(test_7588d_replay, "../assets/replays/good/7588d.replay", 0);
frame_len_test!(test_42f2_replay, "../assets/replays/good/42f2.replay", 11642);
frame_len_test!(test_qq_platform_0ca5_replay, "../assets/replays/good/0ca5.replay", 472);
frame_len_test!(test_tutorial_43a9_replay, "../assets/replays/good/43a9.replay", 9143);
frame_len_test!(test_dedicated_server_ip_43a9_replay, "../assets/replays/good/160c.replay", 9408);
frame_len_test!(test_hoops_mutator_d044_replay, "../assets/replays/good/d044.replay", 10497);
frame_len_test!(test_soccar_lan_replay, "../assets/replays/good/soccar-lan.replay", 7398);
frame_len_test!(test_c62cb_replay, "../assets/replays/good/c62cb.replay", 7136);
frame_len_test!(test_podium_51235_replay, "../assets/replays/good/51235.replay", 8002);
frame_len_test!(test_54aed_replay, "../assets/replays/good/54aed.replay", 7407);
frame_len_test!(test_ae466_replay, "../assets/replays/good/ae466.replay", 7067);
frame_len_test!(test_d5d6c_replay, "../assets/replays/good/d5d6c.replay", 11009);
frame_len_test!(test_128ed_replay, "../assets/replays/good/128ed.replay", 8313);
frame_len_test!(test_rlcs_replay, "../assets/replays/good/rlcs.replay", 7651);
frame_len_test!(test_many_actors_replay, "../assets/replays/good/many_actors.replay", 4179);
frame_len_test!(test_max_score_replay, "../assets/replays/good/a184.replay", 604);
frame_len_test!(test_is_distracted, "../assets/replays/good/e978.replay", 9501);
frame_len_test!(test_d4f3b_heat_replay, "../assets/replays/good/d4f3b_heat.replay", 8920);
frame_len_test!(test_ed6ce_heat_replay, "../assets/replays/good/ed6ce_heat.replay", 10920);
frame_len_test!(test_rl178_replay, "../assets/replays/good/rl-178.replay", 8901);
frame_len_test!(test_epic_replay, "../assets/replays/good/epic.replay", 7231);
frame_len_test!(test_tourny_replay, "../assets/replays/good/tourny.replay", 10275);
frame_len_test!(test_difficulty_replay, "../assets/replays/good/difficulty.replay", 3897);
frame_len_test!(test_rumble_pickup_replay, "../assets/replays/good/140a5.replay", 9082);
frame_len_test!(test_rlcs2_replay, "../assets/replays/good/rlcs2.replay", 10664);
frame_len_test!(test_00bb_replay, "../assets/replays/good/00bb.replay", 8175);
frame_len_test!(test_gridiron_replay, "../assets/replays/good/gridiron.replay", 11118);
frame_len_test!(test_rl_2_replay, "../assets/replays/good/5f97d.replay", 10780);
frame_len_test!(test_fecd, "../assets/replays/good/fecd.replay", 12180);
frame_len_test!(test_436d, "../assets/replays/good/436d.replay", 10006);
frame_len_test!(test_voice_update, "../assets/replays/good/voice_update.replay", 786);
frame_len_test!(test_4742, "../assets/replays/good/4742.replay", 9875);
frame_len_test!(test_drop_shot, "../assets/replays/good/204c.replay", 9377);
frame_len_test!(test_59d3, "../assets/replays/good/59d3.replay", 10309);
