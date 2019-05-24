use boxcars::ParserBuilder;

macro_rules! run {
    ($test_name:ident, $replay:expr) => {
        #[test]
        fn $test_name() {
            let data = include_bytes!($replay);

            let parsed = ParserBuilder::new(&data[..])
                .always_check_crc()
                .must_parse_network_data()
                .parse();
            
            match parsed {
                Ok(_replay) => { },
                Err(ref e) => panic!(format!("{}", e))
            }
        }
    };
}

run!(not_working_1, "../assets/not_working (1).replay");
run!(not_working_2, "../assets/not_working (2).replay");
run!(not_working_3, "../assets/not_working (3).replay");
run!(not_working_4, "../assets/not_working (4).replay");