#[allow(unused_imports)]
use super::*;

mod infrared_datagram {

    use super::*;

    #[test]
    fn default() {
        let sut = Datagram::default();
        assert_eq!(0, sut.length_in_bit);
        assert_eq!(0, sut.buffer);
    }

    #[test]
    fn add_bit_datagram_full() {
        let mut sut = Datagram::default();
        sut.length_in_bit = 127;
        assert!(sut.add_bit(true, BitOrder::LittleEndian).is_err());
    }

    #[test]
    fn add_bit_some_bits_big_endian() {
        let mut sut = Datagram::default();
        assert!(sut.add_bit(false, BitOrder::LittleEndian).is_ok());
        assert_eq!(0b0, sut.buffer);
        assert!(sut.add_bit(true, BitOrder::LittleEndian).is_ok());
        assert_eq!(0b10, sut.buffer);
        assert!(sut.add_bit(false, BitOrder::LittleEndian).is_ok());
        assert_eq!(0b010, sut.buffer);
        assert!(sut.add_bit(true, BitOrder::LittleEndian).is_ok());
        assert_eq!(4, sut.length_in_bit);
        assert_eq!(0b1010, sut.buffer);
    }

    #[test]
    fn add_bit_some_bits_little_endian() {
        let mut sut = Datagram::default();
        assert!(sut.add_bit(true, BitOrder::BigEndian).is_ok());
        assert_eq!(0b1, sut.buffer);
        assert!(sut.add_bit(false, BitOrder::BigEndian).is_ok());
        assert_eq!(0b10, sut.buffer);
        assert!(sut.add_bit(true, BitOrder::BigEndian).is_ok());
        assert_eq!(0b101, sut.buffer);
        assert!(sut.add_bit(false, BitOrder::BigEndian).is_ok());
        assert_eq!(4, sut.length_in_bit);
        assert_eq!(0b1010, sut.buffer);
    }

    #[test]
    fn new() {
        let sut = Datagram::new("01-0111");
        assert_eq!(6, sut.length_in_bit);
        assert_eq!(0b010111, sut.buffer);

        let sut = Datagram::new("01110");
        assert_eq!(5, sut.length_in_bit);
        assert_eq!(0b01110, sut.buffer);

        let sut = Datagram::new("0111");
        assert_eq!(4, sut.length_in_bit);
        assert_eq!(0b111, sut.buffer);
    }

    #[test]
    fn compare() {
        let sut = Datagram::new("111-010");
        let mut other = Datagram::default();
        other.length_in_bit = 6;
        other.buffer = 0b111010;
        assert_eq!(sut, other);
    }

    #[test]
    fn index_access() {
        let sut = Datagram::new("0");
        assert_eq!(0, sut[0]);

        let sut = Datagram::new("1");
        assert_eq!(1, sut[0]);

        let sut = Datagram::new("10");
        assert_eq!(1, sut[1]);
        assert_eq!(0, sut[0]);
    }

    #[test]
    fn extract_data() {
        let sut = Datagram::new("010011");
        assert_eq!(0b11, sut.extract_data(0, 2));
        assert_eq!(0b011, sut.extract_data(0, 3));
        assert_eq!(0b01, sut.extract_data(1, 3));
        assert_eq!(0b0100, sut.extract_data(2, 6));
        assert_eq!(0b10011, sut.extract_data(0, 6));
    }

    #[test]
    #[should_panic]
    fn range_access_too_big_index() {
        let sut = Datagram::new("01");
        let _ = sut[2];
    }

    #[test]
    #[should_panic]
    fn extract_data_too_big_max_index() {
        let sut = Datagram::new("01");
        let _ = sut.extract_data(0, 4);
    }

    #[test]
    #[should_panic]
    fn extract_data_to_big_min_index() {
        let sut = Datagram::new("01111");
        let _ = sut.extract_data(5, 4);
    }
}

mod datagram_iterator {

    use super::*;

    #[test]
    fn iterate_empty_big_endian() {
        let datagram = Datagram::new("");
        let mut sut = datagram.into_big_endian_iter();
        assert_eq!(None, sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_zero_big_endian() {
        let datagram = Datagram::new("0");
        let mut sut = datagram.into_big_endian_iter();
        assert_eq!(Some(false), sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_one_big_endian() {
        let datagram = Datagram::new("1");
        let mut sut = datagram.into_big_endian_iter();
        assert_eq!(Some(true), sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_zero_one_big_endian() {
        let datagram = Datagram::new("01");
        let mut sut = datagram.into_big_endian_iter();
        assert_eq!(Some(false), sut.next());
        assert_eq!(Some(true), sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_one_zero_big_endian() {
        let datagram = Datagram::new("10");
        let mut sut = datagram.into_big_endian_iter();
        assert_eq!(Some(true), sut.next());
        assert_eq!(Some(false), sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_zero_one_one_big_endian() {
        let datagram = Datagram::new("110");
        let mut sut = datagram.into_big_endian_iter();
        assert_eq!(Some(true), sut.next());
        assert_eq!(Some(true), sut.next());
        assert_eq!(Some(false), sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_zero_one_little_endian() {
        let datagram = Datagram::new("01");
        let mut sut = datagram.into_little_endian_iter();
        assert_eq!(Some(true), sut.next());
        assert_eq!(Some(false), sut.next());
        assert_eq!(None, sut.next());
    }
}

mod encoder {

    use super::*;

    #[test]
    fn iterate_empty() {
        let datagram = Datagram::new("");
        let mut sut = Encoder::<DatagramLittleEndianIterator>::new(datagram);
        assert_eq!(None, sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_zero() {
        let datagram = Datagram::new("0");
        let mut sut = Encoder::<DatagramLittleEndianIterator>::new(datagram);

        assert_eq!(Some(true), sut.next());
        assert_eq!(Some(false), sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_one() {
        let datagram = Datagram::new("1");
        let mut sut = Encoder::<DatagramLittleEndianIterator>::new(datagram);

        assert_eq!(Some(false), sut.next());
        assert_eq!(Some(true), sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_zero_zero() {
        let datagram = Datagram::new("00");
        let mut sut = Encoder::<DatagramLittleEndianIterator>::new(datagram);
        // first zero
        assert_eq!(Some(true), sut.next());
        assert_eq!(Some(false), sut.next());
        // second zero
        assert_eq!(Some(true), sut.next());
        assert_eq!(Some(false), sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_zero_one() {
        let datagram = Datagram::new("01");
        let mut sut = Encoder::<DatagramBigEndianIterator>::new(datagram);
        // first zero
        assert_eq!(Some(true), sut.next());
        assert_eq!(Some(false), sut.next());
        // second one
        assert_eq!(Some(false), sut.next());
        assert_eq!(Some(true), sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_zero_one_little_endian() {
        let mut sut = Encoder::<DatagramLittleEndianIterator>::new(Datagram::new("01"));
        // first one
        assert_eq!(Some(false), sut.next());
        assert_eq!(Some(true), sut.next());
        // second one
        assert_eq!(Some(true), sut.next());
        assert_eq!(Some(false), sut.next());
        assert_eq!(None, sut.next());
    }
}

mod decoder {

    use super::*;

    macro_rules! assert_receive_datagram {
        ($sut:expr, $sample:expr, $expected:expr) => {
            let expected = Datagram::new($expected);
            let signal = match $sample {
                '-' => true,
                '.' => false,
                _ => false,
            };
            match $sut.next(signal) {
                None => assert!(false, "None at compare"),
                Some(datagram) => assert_eq!(datagram, expected),
            };
        };
    }

    macro_rules! assert_signal_sampling {
        ($sut:expr, $signal:expr) => {
            for sample in $signal.bytes() {
                match sample {
                    b'-' => {
                        assert!($sut.next(true).is_none())
                    }
                    b'.' => {
                        assert!($sut.next(false).is_none())
                    }
                    _ => (),
                };
            }
        };
    }

    macro_rules! assert_reverse_signal_sampling {
        ($sut:expr, $signal:expr) => {
            for sample in $signal.bytes() {
                match sample {
                    b'-' => {
                        assert!($sut.next(false).is_none())
                    }
                    b'.' => {
                        assert!($sut.next(true).is_none())
                    }
                    _ => (),
                };
            }
        };
    }

    #[test]
    fn new() {
        let sut = Decoder::new(
            ActivityLevel::Low,
            SyncOnTurningEdge::First,
            BitOrder::LittleEndian,
        );
        assert_eq!(true, sut.previous_sample);

        let sut = Decoder::new(
            ActivityLevel::High,
            SyncOnTurningEdge::First,
            BitOrder::LittleEndian,
        );
        assert_eq!(false, sut.previous_sample);

        assert_eq!(NO_EDGE_EXIT_LIMIT, sut.edge_distance);
        assert_eq!(NO_EDGE_EXIT_LIMIT, sut.recording_distance);
        assert_eq!(sut.datagram, Datagram::default());
    }

    #[test]
    fn sample_on_first_datagram_1011() {
        let mut sut = Decoder::new(
            ActivityLevel::Low,
            SyncOnTurningEdge::First,
            BitOrder::BigEndian,
        );
        //          -----+-----+-----+-----+-----+-----+
        let input = "--------......------...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, '-', "1011");
    }

    #[test]
    fn sample_on_first_revere_high_active_datagram_0100() {
        let mut sut = Decoder::new(
            ActivityLevel::High,
            SyncOnTurningEdge::First,
            BitOrder::BigEndian,
        );
        //          -----+-----+-----+-----+-----+-----+
        let input = "--------......------...---...---------";
        assert_reverse_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, '.', "0100");
    }

    #[test]
    fn sample_on_first_edge_little_endian_datagram_1101() {
        let mut sut = Decoder::new(
            ActivityLevel::Low,
            SyncOnTurningEdge::First,
            BitOrder::LittleEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......------...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, '-', "1101");
    }

    #[test]
    fn sample_datagram_011() {
        let mut sut = Decoder::new(
            ActivityLevel::Low,
            SyncOnTurningEdge::Second,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "-----...------...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, '-', "011");
    }

    // tests about activity and edge level

    #[test]
    fn sample_low_active_on_first_edge_datagram_with_one_bit() {
        let mut sut = Decoder::new(
            ActivityLevel::Low,
            SyncOnTurningEdge::First,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+
        let input = "--------...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, '-', "1");
    }

    #[test]
    fn sample_low_active_on_second_edge_datagram_with_one_bit() {
        let mut sut = Decoder::new(
            ActivityLevel::Low,
            SyncOnTurningEdge::Second,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+
        let input = "-----...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, '-', "0");
    }

    #[test]
    fn sample_high_active_on_first_edge_datagram_with_one_bit() {
        let mut sut = Decoder::new(
            ActivityLevel::High,
            SyncOnTurningEdge::First,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+
        let input = "........---.........";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, '.', "0");
    }

    #[test]
    fn sample_high_active_on_second_edge_datagram_with_one_bit() {
        let mut sut = Decoder::new(
            ActivityLevel::High,
            SyncOnTurningEdge::Second,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+
        let input = ".....---.........";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, '.', "1");
    }

    #[test]
    fn logic() {
        let sample = false;
        let high_activity = false;
        assert!(true && (sample ^ !high_activity));
    }

    #[test]
    fn sample_failure_high_inactive_starts_with_low_sample() {
        let mut sut = Decoder::new(
            ActivityLevel::Low,
            SyncOnTurningEdge::First,
            BitOrder::LittleEndian,
        );
        let input = "............";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn sample_failure_low_inactive_start_with_low_sample() {
        let mut sut = Decoder::new(
            ActivityLevel::High,
            SyncOnTurningEdge::First,
            BitOrder::LittleEndian,
        );
        let input = "......................";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn sample_failure_low_inactive_starts_with_high_sample() {
        let mut sut = Decoder::new(
            ActivityLevel::High,
            SyncOnTurningEdge::First,
            BitOrder::LittleEndian,
        );
        let input = "-----------------";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn sample_failure_high_inactive_start_with_high_sample() {
        let mut sut = Decoder::new(
            ActivityLevel::Low,
            SyncOnTurningEdge::First,
            BitOrder::LittleEndian,
        );
        let input = "-----------------------";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn sample_no_datagram_low_inactive_one_edge_only() {
        let mut sut = Decoder::new(
            ActivityLevel::High,
            SyncOnTurningEdge::First,
            BitOrder::LittleEndian,
        );
        let input = "...........-----------";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn sample_no_datagram_high_inactive_one_edge_only() {
        let mut sut = Decoder::new(
            ActivityLevel::Low,
            SyncOnTurningEdge::First,
            BitOrder::LittleEndian,
        );
        let input = "------------............";
        assert_signal_sampling!(&mut sut, input);
    }
}
