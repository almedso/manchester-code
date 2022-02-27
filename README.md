# manchester-code

## Manchester Decoder

Features:

* Decode monotonically sampled data stream that is Manchester modulated
  like it is used in RC5
  * High/low IN-Activitity configuration
  * Zero or one first bit configuration
  * Big endian/ little endian configuration
  * Automatic start and end of datagram detection
  * Sampling needs to be 3 times the length of half a bit. (i.e. only a
    single periodic timer is needed), for a infrared receiver
    889 µs halfbit period => the periodic timer should run all 297 µs.
* Encode
  * Big endian/ little endian configuration
    
## Manchester Modulation

https://en.wikipedia.org/wiki/Manchester_code

* The first bit (after inactivity is always recognized as zero)
* A datagram is finished if their is no edge anymore longer than a bit
  period and all subsequent samples are at inactive level
* all other situations are treated as errors and are rejected

## Example

The lib runs in no_std environments

```rust
#![deny(warnings)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use nucleo_stm32g071rb as board;

use board::hal::prelude::*;
use board::hal::stm32;
use board::hal::nb::block;

use manchester_code::Decode;

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let mut rcc = dp.RCC.constrain();

    let gpioa = dp.GPIOA.split(&mut rcc);
    let infrared = gpioa.pa8.into_pull_up_input();

    let mut timer = dp.TIM17.timer(&mut rcc);
    timer.start(297.us());
    let mut receiver = Decoder::new(true);
    loop {
        match receiver.next(infrared.is_high().unwrap()) {
            None => (),
            Some(t) => if t.length() > 2 {
                defmt::println!("Datagram: {:?}",  t ); },
        };
        block!(timer.wait()).unwrap();
    }
}
```

# Todo

* defmt optional
* fmt optional
* async as stream
* ci + readme reporting
* publish
