//! Serial echo
//!
//! Connect using e.g., `moserial` to `/dev/ttyACM0`
//! 115200 8N1
//!
//! The MCU will echo incoming data and send a trace over ITM.
//! Notice, as the hardware has a single byte buffer only, the input
//! buffer may overflow.

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m_rt::entry;
use nb::block;
extern crate stm32f4xx_hal as hal;

use crate::hal::prelude::*;
use crate::hal::serial::{config::Config, Serial};
use cortex_m::iprintln;

#[entry]
fn main() -> ! {
    let mut c = hal::stm32::CorePeripherals::take().unwrap();
    let stim = &mut c.ITM.stim[0];

    let p = hal::stm32::Peripherals::take().unwrap();

    // let mut flash = p.FLASH.constrain();
    let rcc = p.RCC.constrain();

    let gpioa = p.GPIOA.split();

    // let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let clocks = rcc.cfgr.freeze();
    // let _clocks = rcc.cfgr.freeze();

    let tx = gpioa.pa2.into_alternate_af7();
    let rx = gpioa.pa3.into_alternate_af7(); // try comment out
                                             // let rx = gpioa.pa3.into_alternate_af0(); // try uncomment

    let serial = Serial::usart2(
        p.USART2,
        (tx, rx),
        Config::default().baudrate(115_200.bps()),
        clocks,
    )
    .unwrap();

    // Separate out the sender and receiver of the serial port
    let (mut tx, mut rx) = serial.split();

    loop {
        match block!(rx.read()) {
            Ok(byte) => {
                iprintln!(stim, "Ok {:?}", byte);
                let _ = tx.write(byte);
            }
            Err(err) => {
                iprintln!(stim, "Error {:?}", err);
            }
        }
    }
}
