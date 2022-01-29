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
        assert!(sut.add_bit(true).is_err());
    }

    #[test]
    fn add_bit_some_bits() {
        let mut sut = Datagram::default();
        assert!(sut.add_bit(true).is_ok());
        assert!(sut.add_bit(false).is_ok());
        assert!(sut.add_bit(true).is_ok());
        assert!(sut.add_bit(false).is_ok());
        assert_eq!(4, sut.length_in_bit);
        assert_eq!(0b1010, sut.buffer);
    }

    #[test]
    fn new() {
        let sut = Datagram::new("01-0111");
        assert_eq!(6, sut.length_in_bit);
        assert_eq!(0b10111, sut.buffer);

        let sut = Datagram::new("01110");
        assert_eq!(5, sut.length_in_bit);
        assert_eq!(0b01110, sut.buffer);

        let sut = Datagram::new("0111");
        assert_eq!(4, sut.length_in_bit);
        assert_eq!(0b0111, sut.buffer);
    }

    #[test]
    fn compare() {
        let sut = Datagram::new("01-0111");
        let mut other = Datagram::default();
        other.length_in_bit = 6;
        other.buffer = 0b10111;
        assert_eq!(sut, other);
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
        }
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
        }
    }

    #[test]
    fn new() {
        let sut = Decoder::new(true);
        assert_eq!(true, sut.high_inactivity);
        assert_eq!(true, sut.previous_sample);

        let sut = Decoder::new(false);
        assert_eq!(false, sut.high_inactivity);
        assert_eq!(false, sut.previous_sample);

        assert_eq!(0, sut.edge_distance);
        assert_eq!(0, sut.recording_distance);
        assert_eq!(sut.datagram, Datagram::default());
    }

    #[test]
    fn zero_bit() {
        let mut sut = Decoder::new(true);
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "0");
    }

    #[test]
    fn zero_zero_bits() {
        let mut sut = Decoder::new(true);
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "00");
    }

    #[test]
    fn zero_zero_zero_bits() {
        let mut sut = Decoder::new(true);
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------...---...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "000");
    }

    #[test]
    fn zero_one_bits() {
        let mut sut = Decoder::new(true);
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "01");
    }

    #[test]
    fn zero_one_zero_bits() {
        let mut sut = Decoder::new(true);
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......------...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "010");
    }

    #[test]
    fn zero_one_zero_zero_bits() {
        let mut sut = Decoder::new(true);
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......------...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "0100");
    }

    #[test]
    fn zero_one_one_bits() {
        let mut sut = Decoder::new(true);
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......---...---------";
        // 01       put = "--------......---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "011");
    }

    #[test]
    fn zero_one_one_zero_bits() {
        let mut sut = Decoder::new(true);
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......---...------...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "0110");
    }

    #[test]
    fn zero_one_one_one_bits() {
        let mut sut = Decoder::new(true);
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......---...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "0111");
    }

    #[test]
    fn zero_one_one_one_zero_bits() {
        let mut sut = Decoder::new(true);
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......---...---...------...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "01110");
    }

    #[test]
    fn zero_one_one_one_one_bits() {
        let mut sut = Decoder::new(true);
        //           -----+-----+-----+-----+-----+-----+
        let input = "--------......---...---...---...---------";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, true, "01111");
    }

    #[test]
    fn low_inactivity_zero_bit() {
        let mut sut = Decoder::new(false);
        //           -----+-----+-----+-----+-----+-----+
        let input = ".........---.........";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, false, "0");
    }

    #[test]
    fn low_inactivity_zero_one_bits() {
        let mut sut = Decoder::new(false);
        //           -----+-----+-----+-----+-----+-----+
        let input = "........------.........";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, false, "01");
    }

    #[test]
    fn low_inactivity_zero_zero_bits() {
        let mut sut = Decoder::new(false);
        //           -----+-----+-----+-----+-----+-----+
        let input = "........---...---.........";
        assert_signal_sampling!(&mut sut, input);
        assert_receive_datagram!(&mut sut, false, "00");
    }

    #[test]
    fn failure_high_inactive_starts_with_low_sample() {
        let mut sut = Decoder::new(true);
        let input = "............";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn failure_low_inactive_start_with_low_sample() {
        let mut sut = Decoder::new(false);
        let input = "......................";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn failure_low_inactive_starts_with_high_sample() {
        let mut sut = Decoder::new(false);
        let input = "-----------------";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn failure_high_inactive_start_with_high_sample() {
        let mut sut = Decoder::new(true);
        let input = "-----------------------";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn no_datagram_low_inactive_one_edge_only() {
        let mut sut = Decoder::new(false);
        let input = "...........-----------";
        assert_signal_sampling!(&mut sut, input);
    }

    #[test]
    fn no_datagram_high_inactive_one_edge_only() {
        let mut sut = Decoder::new(true);
        let input = "------------............";
        assert_signal_sampling!(&mut sut, input);
    }
}
