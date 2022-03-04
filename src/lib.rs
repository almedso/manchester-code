//! # Manchester Decoder
//!
//! Features:
//!
//! * Decode monotonically sampled data stream that is Manchester modulated
//!   like it is used in Philips RC5
//!
//!   <https://techdocs.altium.com/display/FPGA/Philips+RC5+Infrared+Transmission+Protocol>
//!
//! * High/low IN-Activitity configuration
//! * Automatic start and end of datagram detection
//! * Sampling needs to be 3 times the length of half a bit. (i.e. only a
//!   single periodic timer is needed), for a infrared receiver
//!   889 µs halfbit period => the periodic timer should run all 297 µs.
//!
//! # Manchester Modulation
//!
//! <https://en.wikipedia.org/wiki/Manchester_code>
//!
//!
//! * A datagram starts after a pause period longer than the time of two bits
//! * A datagram is finished if their is no edge anymore longer than a bit
//!   period and all subsequent samples are at inactive level
//! * all other situations are treated as errors and are rejected
//! * Bit order of a datagram:
//!   * The first bit received is the most significant bit (MSB) and
//!     the last bit
//!
//!
//! ## Receiving Algorithm Details
//!
//! A Periodic sampling is used.
//!
//! * Three samples per half bit period, will do. It gives a one third (of half period)
//!   tolerance. And allows for one third (of half period) where the signal is
//!   expected to be stable.
//!
//!   Thus, the Philips half bit time can vary 889 µs +/- 296 µs = [595; 1175] µs
//!
//! * For every bit there is an edge at the transition from first half bit to
//!   second half bit. This is period is used to synchronize bit value measurement
//!
//! * The first bit value must be pre-known, because it determines where the
//!   synchronization edges are to be expected:
//!
//!   |
//! # Example
//!
//! The lib runs in no_std environments
//!
//! ```ignore
//! #![deny(warnings)]
//! #![deny(unsafe_code)]
//! #![no_main]
//! #![no_std]
//!
//! use nucleo_stm32g071rb as board; //  it also includes mem, defmt
//!
//! use board::hal::prelude::*;
//! use board::hal::stm32;
//! use board::hal::nb::block;
//!
//! use manchester_code::Decoder;
//!
//! #[cortex_m_rt::entry]
//! fn main() -> ! {
//!     let dp = stm32::Peripherals::take().expect("cannot take peripherals");
//!     let mut rcc = dp.RCC.constrain();
//!
//!     let gpioa = dp.GPIOA.split(&mut rcc);
//!     let infrared = gpioa.pa8.into_pull_up_input();
//!
//!     let mut timer = dp.TIM17.timer(&mut rcc);
//!     timer.start(297.us());
//!     let mut receiver = Decoder::new(true);
//!     loop {
//!         match receiver.next(infrared.is_high().unwrap()) {
//!             None => (),
//!             Some(t) => if t.length() > 2 {
//!                 defmt::println!("Datagram: {:?}",  t ); },
//!         };
//!         block!(timer.wait()).unwrap();
//!     }
//! }
//! ```

#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]

use defmt::Format;

use core::iter::Iterator;
use core::ops::Index;

use embedded_hal::Pwm;

#[derive(Copy, Clone, Debug)]
pub enum BitOrder {
    BigEndian,    // MSB is transmitted first; LSB is transmitted last
    LittleEndian, // LSB is transmitted first; MSB is transmitted last
}

/// Representation of a datagram
///
/// The total length is limited to 128 bits
/// The bits of a telegram are internally enumerated from 0 to 127.
/// A default datagram is expected to be empty (i.e. containing zero bits)
#[derive(Default, Copy, Clone, Debug)]
pub struct Datagram {
    length_in_bit: u8,
    buffer: u128,
}

#[derive(Debug)]
struct Error;

impl Datagram {
    /// Add a bit to a datagram
    ///
    /// The new bit is placed at index zero.
    /// The index of all previously added bits gets increased by one.
    ///
    /// # Arguments
    ///
    /// * `bit` - The bit value to record at index 0
    /// * `bit_order`- The bit order either BigEndian or LittleEndian determines
    ///                if the bit is added at the MSB or LSB position
    ///
    /// # Returns
    ///
    /// * Error - if the datagram is already filled up to its capacity.
    /// * () - if the bit was successfully added
    fn add_bit(&mut self, bit: bool, order: BitOrder) -> Result<(), Error> {
        if self.length_in_bit == 127 {
            Err(Error)
        } else {
            match order {
                BitOrder::LittleEndian => {
                    self.buffer <<= 1;
                    if bit {
                        self.buffer += 1;
                    };
                }
                BitOrder::BigEndian => {
                    if bit {
                        self.buffer += 1 << self.length_in_bit;
                    }
                }
            }
            self.length_in_bit += 1;
            Ok(())
        }
    }

    pub fn len(&self) -> u8 {
        self.length_in_bit
    }

    pub fn is_empty(&self) -> bool {
        0 == self.length_in_bit
    }

    /// Extract a data slice of from the datagram
    ///
    /// Byteorder is maintained
    pub fn extract_data(&self, min: u8, max: u8) -> u128 {
        if max > self.length_in_bit {
            panic!("Max index to big");
        }
        if min >= max {
            panic!("Min index to greater than max index");
        }

        let mut value = 0_u128;
        for index in min..max {
            let mask: u128 = 1 << (self.length_in_bit - 1 - index);
            let bit = if mask & self.buffer == 0 { &0 } else { &1 };
            value <<= 1;
            value += bit;
        }
        value
    }

    /// Create a new datagram from "binary" string
    ///
    /// # Arguments
    ///
    /// * `bit_repr` - Bit representation as string of zeros and ones.
    ///                Arbitrary delimiter signs (for readability) are ignored
    /// # Example
    ///
    /// ```rust
    /// use manchester_code::Datagram;
    ///
    /// let datagram = Datagram::new("0-111_10101_00001111");
    /// ```
    pub fn new(bit_repr: &str) -> Self {
        let mut datagram = Datagram::default();
        for bit in bit_repr.bytes() {
            match bit {
                b'0' => datagram.add_bit(false, BitOrder::BigEndian).unwrap(),
                b'1' => datagram.add_bit(true, BitOrder::BigEndian).unwrap(),
                _ => (),
            }
        }
        datagram
    }

    fn into_big_endian_iter(self) -> DatagramBigEndianIterator {
        DatagramBigEndianIterator {
            datagram: self,
            index: self.len(),
        }
    }

    fn into_little_endian_iter(self) -> DatagramLittleEndianIterator {
        DatagramLittleEndianIterator {
            datagram: self,
            index: 0,
        }
    }
}

impl Index<u8> for Datagram {
    type Output = u128;

    /// Access the n-th element via index
    ///
    /// # Panics
    ///
    /// * if the index is out of range
    ///
    /// # Example
    ///
    /// ```rust
    /// use manchester_code::Datagram;
    ///
    /// let datagram = Datagram::new("0-111_10101_00001111");  // LittleEndian representation
    /// assert_eq!(1, datagram[0]);
    /// assert_eq!(0, datagram[5]);
    /// ```
    fn index(&self, index: u8) -> &Self::Output {
        if index >= self.length_in_bit {
            panic!("Wrong Index")
        }
        let mask: u128 = 1 << (self.length_in_bit - 1 - index);
        if mask & self.buffer == 0 {
            &0
        } else {
            &1
        }
    }
}

impl PartialEq for Datagram {
    fn eq(&self, other: &Self) -> bool {
        self.buffer == other.buffer && self.length_in_bit == other.length_in_bit
    }
}

impl Eq for Datagram {}

impl Format for Datagram {
    fn format(&self, f: defmt::Formatter) {
        for index in 0..self.length_in_bit {
            if 0 == index % 4 {
                defmt::write!(f, "-");
            }
            if 1 == self[self.length_in_bit - 1 - index] {
                defmt::write!(f, "1");
            } else {
                defmt::write!(f, "0");
            }
        }
    }
}

#[derive(Debug)]
pub struct DatagramBigEndianIterator {
    datagram: Datagram,
    index: u8,
}

impl Iterator for DatagramBigEndianIterator {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if 0 < self.index {
            self.index -= 1;
            Some(1 == self.datagram[self.index])
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct DatagramLittleEndianIterator {
    datagram: Datagram,
    index: u8,
}

impl Iterator for DatagramLittleEndianIterator {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.datagram.len() > self.index {
            self.index += 1;
            Some(1 == self.datagram[self.index - 1])
        } else {
            None
        }
    }
}

/// Encodes a datagram to Manchester code
///
/// The encoder turns into an iterator.
/// the encoding happens by calling the iterator.
///
/// # Example
///
/// ```rust
/// use manchester_code::{
///     Datagram,
///     DatagramBigEndianIterator,
///     Encoder};
///
/// let mut encoder = Encoder::<DatagramBigEndianIterator>::new(Datagram::new("01"));
/// assert_eq!(Some(true), encoder.next());
/// assert_eq!(Some(false), encoder.next());
/// assert_eq!(Some(false), encoder.next());
/// assert_eq!(Some(true), encoder.next());
/// assert_eq!(None, encoder.next());
/// ```

#[derive(Debug)]
pub struct Encoder<I> {
    datagram_iter: I,
    first_half_bit: bool,
    last_value: Option<bool>,
}

impl Encoder<DatagramBigEndianIterator> {
    /// Create a new Encoder ready to encode the datagram passed along
    ///
    /// # Arguments
    ///
    /// * `datagram` - the datagram to be encoded
    pub fn new(d: Datagram) -> Self {
        let mut datagram_iter = d.into_big_endian_iter();
        let last_value = datagram_iter.next();
        Encoder::<DatagramBigEndianIterator> {
            datagram_iter,
            first_half_bit: true,
            last_value,
        }
    }
}

impl Encoder<DatagramLittleEndianIterator> {
    /// Create a new Encoder ready to encode the datagram passed along
    ///
    /// # Arguments
    ///
    /// * `datagram` - the datagram to be encoded
    pub fn new(d: Datagram) -> Self {
        let mut datagram_iter = d.into_little_endian_iter();
        let last_value = datagram_iter.next();
        Encoder::<DatagramLittleEndianIterator> {
            datagram_iter,
            first_half_bit: true,
            last_value,
        }
    }
}

impl<I: Iterator<Item = bool>> Iterator for Encoder<I> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        match self.last_value {
            Some(bit) => {
                if self.first_half_bit {
                    self.first_half_bit = false;
                    Some(!bit)
                } else {
                    self.first_half_bit = true;
                    self.last_value = self.datagram_iter.next();
                    Some(bit)
                }
            }
            None => None,
        }
    }
}

//
pub enum InactivityLevel {
    High,
    Low,
}

pub enum FirstBitExpectation {
    Zero,
    One,
}

/// Decode a Manchester encoded stream of periodically taken samples into
/// a datagram.
pub struct Decoder {
    // Config data
    high_inactivity: bool,
    first_bit_is_one: bool,
    bit_order: BitOrder,
    // Collected output data
    datagram: Datagram,
    // Internal processing control data
    previous_sample: bool,
    edge_distance: u8,
    recording_distance: u8,
    receiving_started: bool,
    record_marker_reached: bool,
}

const SAMPLES_PER_HALFBIT_PERIOD: u8 = 3;
const TOLERANCE: u8 = 1;

//   ___---___------   e - first edge
//   xxx012345678901   x - exit criteria no bits are send anymore
//     f----tttt--xxx  t - tolerance range an edge is expected

const LOWER_BARRIER: u8 = 2 * SAMPLES_PER_HALFBIT_PERIOD - TOLERANCE;
const UPPER_BARRIER: u8 = 2 * SAMPLES_PER_HALFBIT_PERIOD + TOLERANCE;
const NO_EDGE_EXIT_LIMIT: u8 = 3 * SAMPLES_PER_HALFBIT_PERIOD;

impl Decoder {
    /// Create an instance of a new manchester encoder
    ///
    /// # Arguments
    ///
    /// * `high_inactivity` - A *true* value expects the input pin state high
    ///                       when nothing is received
    /// * `first_bit_is_one` - A *true value enforces the bit recording to start
    ///                        with a "1". This is important to know upfront
    ///                        where a new bit starts
    /// * `bit_order` - Either BigEndian (MSP is received first) or
    ///                 LittleEndian (LSB is received first)
    pub const fn new(
        inactivity_level: InactivityLevel,
        first_bit_expectation: FirstBitExpectation,
        bit_order: BitOrder,
    ) -> Self {
        let high_inactivity = match inactivity_level {
            InactivityLevel::High => true,
            InactivityLevel::Low => false,
        };
        let first_bit_is_one = match first_bit_expectation {
            FirstBitExpectation::One => true,
            FirstBitExpectation::Zero => false,
        };
        Decoder {
            datagram: Datagram {
                buffer: 0,
                length_in_bit: 0,
            },
            previous_sample: high_inactivity,
            edge_distance: NO_EDGE_EXIT_LIMIT,
            recording_distance: NO_EDGE_EXIT_LIMIT,
            receiving_started: false,
            high_inactivity,
            first_bit_is_one,
            record_marker_reached: false,
            bit_order,
        }
    }

    /// Sample a manchester modulated signal periodically and extract datagrams
    ///
    /// To cover some jitter the sampling rate is three times the half bit frequency
    /// i.e. an Infrared manchester decoded bit lasts  2x  889 us),
    /// Thus sampling period should be 296 us.
    ///
    /// Note: three times the bit frequency is good enough to consider the Nyquist
    /// criterion and some potential jitter in sending frequency.
    ///
    /// # Arguments
    ///
    ///  * `sample` - the level of the pin true equals high, false equals low
    ///
    /// # Returns
    ///
    ///  Option of an infrared datagram
    ///
    ///  * None - if no complete datagram is received
    ///  * Some(datagram) - a completely received datagram
    ///
    pub fn next(&mut self, sample: bool) -> Option<Datagram> {
        // To understand the algorithm record marker are introduced.
        //
        // Record marker are the sample taken directly after the edge
        // in the middle of the transmission of a bit
        // As of the manchester protocol definition, there must be always
        // an edge in the middle of transmission of the bit.
        //
        // Example: Start with "1" (high inactivity)
        //
        //        |bit_1|bit_0|bit_0     - The bits
        // -------...------...---...     - The signal
        //           ^     ^     ^       - The record marker
        //
        // Example: Start with "0" (high inactivity)
        //
        //        |bit_0|bit_1|bit_1     - The bits
        // ----------......---...---     - The signal
        //           ^     ^     ^       - The record marker
        //
        // At each record marker the bit value is determined and recorded
        let mut return_value: Option<Datagram> = None;

        if sample != self.previous_sample {
            if !self.receiving_started {
                // cover the start of the telegram
                if self.first_bit_is_one {
                    // start with a "1"
                    // by protocol design it is guaranteed that there is a second edge
                    // within half-bit time aka within SAMPLES_PER_HALFBIT_PERIOD
                    if self.edge_distance <= SAMPLES_PER_HALFBIT_PERIOD + TOLERANCE {
                        // second edge at the record marker
                        self.record_marker_reached = true;
                        self.receiving_started = true;
                    } else {
                        // very first edge -> do nothing on purpose
                    }
                } else {
                    // start with a "0"
                    // first edge is the record marker
                    self.record_marker_reached = true;
                    self.receiving_started = true;
                }
            }
            if self.recording_distance >= LOWER_BARRIER && self.recording_distance <= UPPER_BARRIER
            {
                self.record_marker_reached = true;
            }
            if self.record_marker_reached {
                // In the middle of a bit transmission the value is derived from the new sample
                self.datagram
                    .add_bit(sample ^ !self.high_inactivity, self.bit_order)
                    .unwrap();
                // reset internal data for the next record_marker
                self.recording_distance = 1;
                self.record_marker_reached = false;
            }
            self.previous_sample = sample;
            self.edge_distance = 1;
        } else {
            self.edge_distance += 1;
            self.recording_distance += 1;
        }

        if self.edge_distance > NO_EDGE_EXIT_LIMIT {
            // end of datagram condition no edge anymore
            if !self.datagram.is_empty() && sample ^ !self.high_inactivity {
                return_value = Some(self.datagram);
                self.receiving_started = false;
            }
            self.datagram = Datagram::default();
            self.edge_distance -= 1; // prevent number overflow
        }
        if self.recording_distance > NO_EDGE_EXIT_LIMIT {
            self.recording_distance -= 1; // prevent number overflow
        }
        return_value
    }
}

/// Control sending of datagrams, manage infrared radiation pollution
///
/// The InfraredEmitter behaves socially by enforcing a pause time between
/// subsequent
/// Required resources:
///
/// * A configured PWM - typically at a frequency of 36..38 kHz (RC5 protocol)
/// * A facility that periodically runs half bit sending, e.g. a timer ISR
///   typically at a period of 889 µs (half bit time, RC5 protocol)
///
/// # Example
///
/// TODO
#[derive(Debug)]
pub struct InfraredEmitter<P, C, I> {
    encoder: Option<Encoder<I>>,
    max_pause_cycles: u8,
    current_pause_cycles: u8,
    pwm: P,
    channel: C,
}

impl<P, C, D, I> InfraredEmitter<P, C, I>
where
    P: Pwm + Pwm<Channel = C> + Pwm<Duty = D>,
    C: Copy,
    D: core::ops::Mul<Output = D> + core::ops::Div<Output = D>,
    I: Iterator<Item = bool>,
{
    /// Create a new infrared Emitter
    ///
    /// # Arguments
    ///
    /// * `pause_cycles` - configures the time between subsequent datagram
    ///                    emissions. The total duration is half-bit-time (889 µs)
    ///                    times number of pause bit cycles. In the pause time
    ///                    no infrared radiation is emitted and other
    ///                    participants can occupy the radiation space.
    /// * `pwm` - the PWM to be used for ir pulse emission
    /// * `channel` - the channel to be used by the PWM
    pub fn new(pause_cycles: u8, pwm: P, channel: C) -> Self {
        InfraredEmitter {
            encoder: None,
            max_pause_cycles: pause_cycles,
            current_pause_cycles: 0,
            pwm,
            channel,
        }
    }

    /// Progress on sending a datagram by emitting a half bit
    ///
    /// This function needs to be called every half-bit period, i.e. each 889 µs.
    /// The periodically required call is most likely delegated to a timer ISR.
    ///
    /// half-bit emitting happens by enabling/disabling a a properly configured
    /// PWM.
    pub fn send_half_bit(&mut self) {
        match &mut self.encoder {
            Some(encoder) => match encoder.next() {
                Some(half_bit) => {
                    if half_bit {
                        self.pwm.enable(self.channel);
                    } else {
                        self.pwm.disable(self.channel);
                    }
                }
                None => {
                    self.pwm.disable(self.channel);
                    self.encoder = None;
                    self.current_pause_cycles = 0;
                }
            },
            None => {
                // the pwm is already disabled -> manage pause period
                self.current_pause_cycles += 1;
            }
        }
    }
}

impl<P, C, D> InfraredEmitter<P, C, DatagramBigEndianIterator>
where
    P: Pwm + Pwm<Channel = C> + Pwm<Duty = D>,
    C: Copy,
    D: core::ops::Mul<Output = D> + core::ops::Div<Output = D>,
{
    /// Immediately start sending a datagram if possible
    ///
    /// Sending is possible iff there is no sending procedure in progress.
    /// A call to this function is not blocking
    ///
    /// # Arguments
    ///
    /// * `datagram` - The datagram to be send
    /// * `sending_power` - The duty cycle of the pwm in percent
    ///                     should be less than or equal 25 (percent)
    ///                     Is reduced to 25 if a higher value is given.
    ///                     Lower sending power is appropriate for pairing datagrams.
    ///
    /// # Returns
    ///
    /// * *true* - if sending was initiated
    /// * *false* - if sending was not possible to initiate
    pub fn send_if_possible(&mut self, datagram: Datagram, sending_power: D) -> bool {
        if self.current_pause_cycles < self.max_pause_cycles {
            false
        } else {
            // let mut sending_power: D = if sending_power > 25 { 25 } else { sending_power };
            // let duty = self.pwm.get_max_duty() * sending_power / 100;
            // self.pwm.set_duty(self.channel, duty);
            self.pwm.set_duty(self.channel, sending_power);
            self.encoder = Some(Encoder::<DatagramBigEndianIterator>::new(datagram));
            true
        }
    }
}

impl<P, C, D> InfraredEmitter<P, C, DatagramLittleEndianIterator>
where
    P: Pwm + Pwm<Channel = C> + Pwm<Duty = D>,
    C: Copy,
    D: core::ops::Mul<Output = D> + core::ops::Div<Output = D>,
{
    /// Immediately start sending a datagram if possible
    ///
    /// Sending is possible iff there is no sending procedure in progress.
    /// A call to this function is not blocking
    ///
    /// # Arguments
    ///
    /// * `datagram` - The datagram to be send
    /// * `sending_power` - The duty cycle of the pwm in percent
    ///                     should be less than or equal 25 (percent)
    ///                     Is reduced to 25 if a higher value is given.
    ///                     Lower sending power is appropriate for pairing datagrams.
    ///
    /// # Returns
    ///
    /// * *true* - if sending was initiated
    /// * *false* - if sending was not possible to initiate
    pub fn send_if_possible(&mut self, datagram: Datagram, sending_power: D) -> bool {
        if self.current_pause_cycles < self.max_pause_cycles {
            false
        } else {
            // let mut sending_power: D = if sending_power > 25 { 25 } else { sending_power };
            // let duty = self.pwm.get_max_duty() * sending_power / 100;
            // self.pwm.set_duty(self.channel, duty);
            self.pwm.set_duty(self.channel, sending_power);
            self.encoder = Some(Encoder::<DatagramLittleEndianIterator>::new(datagram));
            true
        }
    }
}
#[cfg(test)]
mod tests;
