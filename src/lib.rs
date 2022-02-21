//! # Manchester Decoder
//!
//! Features:
//!
//! * Decode monotonically sampled data stream that is Manchester modulated
//!   like it is used in RC5
//! * High/low IN-Activitity configuration
//! * Automatic start and end of datagram detection
//! * Sampling needs to be 3 times the length of half a bit. (i.e. only a
//!   single periodic timer is needed), for a infrared receiver
//!   889 µs halfbit period => the periodic timer should run all 297 µs.
//!
//! # Manchester Modulation
//!
//! https://en.wikipedia.org/wiki/Manchester_code
//!
//! * The first bit (after inactivity is always recognized as zero)
//! * A datagram is finished if their is no edge anymore longer than a bit
//!   period and all subsequent samples are at inactive level
//! * all other situations are treated as errors and are rejected
//!
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

use defmt::Format;

use core::iter::{IntoIterator, Iterator};

#[derive(Default, Copy, Clone, Debug)]
pub struct Datagram {
    length_in_bit: u8,
    buffer: u128,
}

const SAMPLES_PER_HALFBIT_PERIOD: u8 = 3;
const TOLERANCE: u8 = 1;

//   ___---___------   e - first edge
//   xxx012345678901   x - exit criteria no bits are send anymore
//     f----tttt--xxx  t - tolerance range an edge is expected

const LOWER_BARRIER: u8 = 2 * SAMPLES_PER_HALFBIT_PERIOD - TOLERANCE;
const UPPER_BARRIER: u8 = 2 * SAMPLES_PER_HALFBIT_PERIOD + TOLERANCE;
const NO_EDGE_EXIT_LIMIT: u8 = 3 * SAMPLES_PER_HALFBIT_PERIOD;

#[derive(Debug)]
struct Error;

impl Datagram {
    fn add_bit(&mut self, bit: bool) -> Result<(), Error> {
        if self.length_in_bit == 127 {
            Err(Error)
        } else {
            self.length_in_bit += 1;
            self.buffer <<= 1;
            if bit {
                self.buffer += 1;
            };
            Ok(())
        }
    }
    pub fn length(&self) -> u8 {
        self.length_in_bit
    }
    pub fn new(bit_repr: &str) -> Self {
        let mut datagram = Datagram::default();
        for bit in bit_repr.bytes() {
            match bit {
                b'0' => datagram.add_bit(false).unwrap(),
                b'1' => datagram.add_bit(true).unwrap(),
                _ => (),
            }
        }
        datagram
    }
    /// Determine of the n-th bit of the datagram is one
    ///
    /// Indexing sequence is analog to vectors, it starts from zero the bit added at the first add_bit call
    /// to n-1 associated with the bit added at the n-th add_bit call.
    /// The length of the datagram quals n
    pub fn is_one(&self, index: u8) -> bool {
        if index >= self.length_in_bit {
            panic!("Wrong Index")
        }
        let mask: u128 = 1 << (self.length_in_bit - 1 - index);
        !matches!(mask & self.buffer, 0)
    }
}

impl PartialEq for Datagram {
    fn eq(&self, other: &Self) -> bool {
        self.buffer == other.buffer && self.length_in_bit == other.length_in_bit
    }
}

pub struct DatagramIterator {
    datagram: Datagram,
    index: u8,
}

impl IntoIterator for Datagram {
    type Item = bool;
    type IntoIter = DatagramIterator;

    fn into_iter(self) -> Self::IntoIter {
        DatagramIterator {
            datagram: self,
            index: self.length(),
        }
    }
}

impl Iterator for DatagramIterator {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if 0 < self.index {
            self.index -= 1;
            Some(
                self.datagram
                    .is_one(self.datagram.length() - 1 - self.index),
            )
        } else {
            None
        }
    }
}

impl Eq for Datagram {}

impl Format for Datagram {
    fn format(&self, f: defmt::Formatter) {
        for index in 0..self.length_in_bit {
            if self.is_one(self.length_in_bit - 1 - index) {
                defmt::write!(f, "1");
            } else {
                defmt::write!(f, "0");
            }
        }
    }
}

pub struct Encoder {
    datagram_iter: DatagramIterator,
    first_half_bit: bool,
    last_value: Option<bool>,
}

impl Encoder {
    pub fn new(d: Datagram) -> Self {
        let mut datagram_iter = d.into_iter();
        let last_value = datagram_iter.next();
        Encoder {
            datagram_iter,
            first_half_bit: true,
            last_value,
        }
    }
}

impl Iterator for Encoder {
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

pub struct Decoder {
    datagram: Datagram,
    previous_sample: bool,
    edge_distance: u8,
    recording_distance: u8,
    high_inactivity: bool,
    receiving_started: bool,
}

impl Decoder {
    pub fn new(high_inactivity: bool) -> Self {
        Decoder {
            datagram: Datagram::default(),
            previous_sample: high_inactivity,
            edge_distance: 0,
            recording_distance: 0,
            receiving_started: false,
            high_inactivity,
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
    ///  None - if no complete datagram is received
    ///  Some(datagram) - a completely received datagram
    ///
    pub fn next(&mut self, sample: bool) -> Option<Datagram> {
        let mut return_value: Option<Datagram> = None;

        if sample != self.previous_sample {
            if !self.receiving_started {
                // cover the start of the telegram - first edge - the middle of "0"
                self.datagram
                    .add_bit(sample ^ !self.high_inactivity)
                    .unwrap();
                self.receiving_started = true;
                self.recording_distance = 1;
            }
            if self.recording_distance >= LOWER_BARRIER && self.recording_distance <= UPPER_BARRIER
            {
                // full bit length - if a valid bit it must be a edge just before
                self.datagram
                    .add_bit(sample ^ !self.high_inactivity)
                    .unwrap();
                self.recording_distance = 1;
            }
            self.previous_sample = sample;
            self.edge_distance = 1;
        } else {
            self.edge_distance += 1;
            self.recording_distance += 1;
        }

        if self.edge_distance > NO_EDGE_EXIT_LIMIT {
            // end of datagram condition no edge anymore
            if self.datagram.length() > 0 && sample ^ !self.high_inactivity {
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

#[cfg(test)]
mod tests;
