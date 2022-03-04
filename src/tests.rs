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
        assert!(sut.add_bit(true, BitOrder::BigEndian).is_err());
    }

    #[test]
    fn add_bit_some_bits_big_endian() {
        let mut sut = Datagram::default();
        assert!(sut.add_bit(false, BitOrder::BigEndian).is_ok());
        assert_eq!(0b0, sut.buffer);
        assert!(sut.add_bit(true, BitOrder::BigEndian).is_ok());
        assert_eq!(0b10, sut.buffer);
        assert!(sut.add_bit(false, BitOrder::BigEndian).is_ok());
        assert_eq!(0b010, sut.buffer);
        assert!(sut.add_bit(true, BitOrder::BigEndian).is_ok());
        assert_eq!(4, sut.length_in_bit);
        assert_eq!(0b1010, sut.buffer);
    }

    #[test]
    fn add_bit_some_bits_little_endian() {
        let mut sut = Datagram::default();
        assert!(sut.add_bit(true, BitOrder::LittleEndian).is_ok());
        assert_eq!(0b1, sut.buffer);
        assert!(sut.add_bit(false, BitOrder::LittleEndian).is_ok());
        assert_eq!(0b10, sut.buffer);
        assert!(sut.add_bit(true, BitOrder::LittleEndian).is_ok());
        assert_eq!(0b101, sut.buffer);
        assert!(sut.add_bit(false, BitOrder::LittleEndian).is_ok());
        assert_eq!(4, sut.length_in_bit);
        assert_eq!(0b1010, sut.buffer);
    }

    #[test]
    fn new() {
        let sut = Datagram::new("01-0111");
        assert_eq!(6, sut.length_in_bit);
        assert_eq!(0b111010, sut.buffer);

        let sut = Datagram::new("01110");
        assert_eq!(5, sut.length_in_bit);
        assert_eq!(0b01110, sut.buffer);

        let sut = Datagram::new("0111");
        assert_eq!(4, sut.length_in_bit);
        assert_eq!(0b1110, sut.buffer);
    }

    #[test]
    fn compare() {
        let sut = Datagram::new("01-0111");
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

        let sut = Datagram::new("01");
        assert_eq!(0, sut[1]);
        assert_eq!(1, sut[0]);
    }

    #[test]
    fn extract_data() {
        let sut = Datagram::new("110010");
        assert_eq!(0b01, sut.extract_data(0, 2));
        assert_eq!(0b0011, sut.extract_data(2, 6));
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
        let datagram = Datagram::new("011");
        let mut sut = datagram.into_big_endian_iter();
        assert_eq!(Some(false), sut.next());
        assert_eq!(Some(true), sut.next());
        assert_eq!(Some(true), sut.next());
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
        let mut sut = Encoder::<DatagramBigEndianIterator>::new(datagram);
        assert_eq!(None, sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_zero() {
        let datagram = Datagram::new("0");
        let mut sut = Encoder::<DatagramBigEndianIterator>::new(datagram);

        assert_eq!(Some(true), sut.next());
        assert_eq!(Some(false), sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_one() {
        let datagram = Datagram::new("1");
        let mut sut = Encoder::<DatagramBigEndianIterator>::new(datagram);

        assert_eq!(Some(false), sut.next());
        assert_eq!(Some(true), sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn iterate_zero_zero() {
        let datagram = Datagram::new("00");
        let mut sut = Encoder::<DatagramBigEndianIterator>::new(datagram);
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
        ($sut:expr, $signal:expr, $expected:expr) => {
            let expected = Datagram::new($expected);
            match $sut.next($signal) {
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

    #[test]
    fn new() {
        let sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        assert_eq!(true, sut.high_inactivity);
        assert_eq!(true, sut.previous_sample);

        let sut = Decoder::new(
            InactivityLevel::Low,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        assert_eq!(false, sut.high_inactivity);
        assert_eq!(false, sut.previous_sample);

        assert_eq!(NO_EDGE_EXIT_LIMIT, sut.edge_distance);
        assert_eq!(NO_EDGE_EXIT_LIMIT, sut.recording_distance);
        assert_eq!(sut.datagram, Datagram::default());
    }

    #[test]
    fn zero_bit() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "0");
    }

    #[test]
    fn zero_zero_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "00");
    }

    #[test]
    fn zero_zero_zero_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------...---...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "000");
    }

    #[test]
    fn zero_one_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "01");
    }

    #[test]
    fn zero_one_zero_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......------...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "010");
    }

    #[test]
    fn zero_one_zero_zero_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......------...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "0100");
    }

    #[test]
    fn zero_one_one_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......---...---------";
        // 01       put = "--------......---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "011");
    }

    #[test]
    fn zero_one_one_zero_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......---...------...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "0110");
    }

    #[test]
    fn zero_one_one_one_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......---...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "0111");
    }

    #[test]
    fn zero_one_one_one_zero_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......---...---...------...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "01110");
    }

    #[test]
    fn zero_one_one_one_one_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......---...---...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "01111");
    }

    #[test]
    fn low_inactivity_zero_bit() {
        let mut sut = Decoder::new(
            InactivityLevel::Low,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = ".........---.........";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, false, "0");
    }

    #[test]
    fn low_inactivity_zero_one_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::Low,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "........------.........";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, false, "01");
    }

    #[test]
    fn low_inactivity_zero_zero_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::Low,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "........---...---.........";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, false, "00");
    }

    #[test]
    fn failure_high_inactive_starts_with_low_sample() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        let input = "............";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn failure_low_inactive_start_with_low_sample() {
        let mut sut = Decoder::new(
            InactivityLevel::Low,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        let input = "......................";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn failure_low_inactive_starts_with_high_sample() {
        let mut sut = Decoder::new(
            InactivityLevel::Low,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        let input = "-----------------";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn failure_high_inactive_start_with_high_sample() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        let input = "-----------------------";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn no_datagram_low_inactive_one_edge_only() {
        let mut sut = Decoder::new(
            InactivityLevel::Low,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        let input = "...........-----------";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn no_datagram_high_inactive_one_edge_only() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::Zero,
            BitOrder::BigEndian,
        );
        let input = "------------............";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn one_bit() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::One,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "1");
    }

    #[test]
    fn one_one_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::One,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "11");
    }

    #[test]
    fn one_zero_zero_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::High,
            FirstBitExpectation::One,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------...------...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "100");
    }

    #[test]
    fn low_inactivity_one_zero_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::Low,
            FirstBitExpectation::One,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "........---......---.........";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, false, "10");
    }

    #[test]
    fn low_inactivity_one_one_bits() {
        let mut sut = Decoder::new(
            InactivityLevel::Low,
            FirstBitExpectation::One,
            BitOrder::BigEndian,
        );
        //           -----+-----+-----+-----+-----+-----+
        let input = "........---...---.........";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, false, "11");
    }
}
