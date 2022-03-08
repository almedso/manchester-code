# Manchester Encoding and Decoding

[![crates.io](https://img.shields.io/crates/v/manchester-code?style=flat-square&logo=rust)](https://crates.io/crates/manchester-code)
[![docs.rs](https://img.shields.io/badge/docs.rs-manchester--code-blue?style=flat-square)](https://docs.rs/manchester-code)
[![license](https://img.shields.io/badge/license-MIT-blue?style=flat-square-blue)](#license)
[![rustc](https://img.shields.io/badge/rustc-1.52+-blue?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![CI status](https://github.com/almedso/manchester-code/actions/workflows/ci.yml/badge.svg)](https://github.com/almedso/manchester-code/actions/workflows/ci.yml)


A `no-std` library to allow Manchester encoding and decoding of datagrams.
It requires certain deep embedded resources like timers, PWM and ISR's.


## Features

* Decode monotonically sampled data stream that is Manchester modulated
  like it is used in RC5
  * High/low IN-Activitity configuration
  * Zero or one first bit configuration
  * Big endian/ little endian configuration
  * Automatic start and end of datagram detection
  * Requires a periodic timer
* Encode
  * Big endian/ little endian configuration
  * Requires a timer ISR and a PWM (single channel)


## Example

* Check the [documentation](https://docs.rs/manchester-code)

## License

This project is licensed under

- MIT License ([`LICENSE.md`](LICENSE.md) or
  [online](https://opensource.org/licenses/MIT))

## Contributing

Your PRs and suggestions are always welcome.


### Future Work

* defmt optional
* fmt optional
* async as stream
* ci + readme reporting
* publish
