//! The RTFM framework
//!
//! What it covers:
//! - Priority based scheduling
//! - Message passing

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::{asm, iprintln};

extern crate stm32f4xx_hal as hal;
use crate::hal::prelude::*;
use crate::hal::serial::{self, config::Config, Rx, Serial, Tx};
use hal::stm32::ITM;

use nb::block;

use rtfm::{app, Instant};

// Our error type
#[derive(Debug)]
pub enum Error {
    RingBufferOverflow,
    UsartSendOverflow,
    UsartReceiveOverflow,
}

#[derive(Debug)]
pub enum Event {
    Timout,
    ChannelA,
    ChannelB,
}

#[derive(Debug, Copy, Clone)]
pub struct Data {
    state: State,
    a: bool,
    b: bool,
    event_counter: u32,
    out: bool,
}

#[derive(Debug, Copy, Clone)]
pub enum State {
    S8000,
    S8001,
    S8004,
    S8014,
    S8005,
    C001,
    C002,
    C003,
}

const TIMEOUT: u32 = 16_000_000;

#[app(device = hal::stm32)]
const APP: () = {
    // Late resources
    static mut TX: Tx<hal::stm32::USART2> = ();
    static mut RX: Rx<hal::stm32::USART2> = ();
    static mut ITM: ITM = ();

    // app resources
    static mut DATA: Data = Data {
        state: State::S8001,
        a: false,
        b: false,
        event_counter: 0,
        out: false,
    };

    // init runs in an interrupt free section>
    #[init(resources = [DATA])]
    fn init() {
        let stim = &mut core.ITM.stim[0];
        iprintln!(stim, "Start {:?}", resources.DATA);

        let rcc = device.RCC.constrain();

        // 16 MHz (default, all clocks)
        let clocks = rcc.cfgr.freeze();

        let gpioa = device.GPIOA.split();

        let tx = gpioa.pa2.into_alternate_af7();
        let rx = gpioa.pa3.into_alternate_af7(); // try comment out

        let mut serial = Serial::usart2(
            device.USART2,
            (tx, rx),
            Config::default().baudrate(115_200.bps()),
            clocks,
        )
        .unwrap();

        // generate interrupt on Rxne
        serial.listen(serial::Event::Rxne);
        // Separate out the sender and receiver of the serial port
        let (tx, rx) = serial.split();

        // Our split serial
        TX = tx;
        RX = rx;

        // For debugging
        ITM = core.ITM;
    }

    // idle may be interrupted by other interrupt/tasks in the system
    // #[idle(resources = [RX, TX, ITM])]
    #[idle]
    fn idle() -> ! {
        loop {
            asm::wfi();
        }
    }

    #[task(priority = 1, resources = [ITM])]
    fn trace_data(byte: u8) {
        let stim = &mut resources.ITM.stim[0];
        iprintln!(stim, "data {}", byte);
    }

    #[task(priority = 1, resources = [ITM])]
    fn trace_error(error: Error) {
        let stim = &mut resources.ITM.stim[0];
        iprintln!(stim, "{:?}", error);
    }

    #[interrupt(priority = 3, resources = [RX], spawn = [trace_data, trace_error, echo])]
    fn USART2() {
        match resources.RX.read() {
            Ok(byte) => {
                spawn.echo(byte).unwrap();
                if spawn.trace_data(byte).is_err() {
                    spawn.trace_error(Error::RingBufferOverflow).unwrap();
                }
            }
            Err(_err) => spawn.trace_error(Error::UsartReceiveOverflow).unwrap(),
        }
    }

    #[task(priority = 2, resources = [TX], spawn = [trace_error, event_a, event_b])]
    fn echo(byte: u8) {
        let tx = resources.TX;

        if block!(tx.write(byte)).is_err() {
            spawn.trace_error(Error::UsartSendOverflow).unwrap();
        }

        match byte {
            b'a' => spawn.event_a(false).unwrap(),
            b'b' => spawn.event_b(false).unwrap(),
            b'A' => spawn.event_a(true).unwrap(),
            b'B' => spawn.event_b(true).unwrap(),
            _ => (),
        }
    }

    #[task(priority = 1, resources = [ITM, DATA], schedule = [discrepency])]
    fn event_a(val: bool) {
        let stim = &mut resources.ITM.stim[0];
        iprintln!(stim, "event a {}", val);

        let data = &mut resources.DATA;
        data.a = val;
        iprintln!(stim, "Start {:?}", data);
        // let data = resources.DATA;

        match data.state {
            State::S8000 => {
                if data.a ^ data.b {
                    schedule
                        .discrepency(scheduled + TIMEOUT.cycles(), data.event_counter, data.state)
                        .unwrap();
                    data.out = false;
                    data.state = State::S8005;
                } else if !data.a & !data.b {
                    data.state = State::S8001
                } else {
                    return;
                }
            }
            State::S8001 => {
                if data.a & !data.b {
                    schedule
                        .discrepency(scheduled + TIMEOUT.cycles(), data.event_counter, data.state)
                        .unwrap();
                    data.out = false;
                    data.state = State::S8004;
                } else if !data.a & data.b {
                    schedule
                        .discrepency(scheduled + TIMEOUT.cycles(), data.event_counter, data.state)
                        .unwrap();
                    data.out = false;
                    data.state = State::S8014;
                } else if data.a & data.b {
                    data.out = true;
                    data.state = State::S8000;
                }
            }
            State::S8004 => {
                if !data.a {
                    data.event_counter += 1;
                    data.out = false;
                    data.state = State::S8001;
                } else if data.b {
                    data.event_counter += 1;
                    data.out = true;
                    data.state = State::S8000;
                } else {
                    data.out = false;
                }
            }
            State::S8014 => {
                if !data.b {
                    data.event_counter += 1;
                    data.out = false;
                    data.state = State::S8001;
                } else if data.a {
                    data.event_counter += 1;
                    data.out = true;
                    data.state = State::S8000;
                } else {
                    data.out = false;
                }
            }
            State::S8005 => {
                if !data.a & !data.b {
                    data.event_counter += 1;
                    data.out = false;
                    data.state = State::S8001;
                } else {
                    data.out = false;
                }
            }

            _ => {
                if !data.a & !data.b {
                    data.out = false;
                    data.state = State::S8001;
                } else {
                    data.out = false;
                }
            }
        }
        iprintln!(stim, "Start {:?}", resources.DATA);
    }

    #[task(priority = 1, resources = [ITM, DATA],  schedule = [discrepency])]
    fn event_b(val: bool) {
        let stim = &mut resources.ITM.stim[0];
        iprintln!(stim, "event b {}", val);

        let data = &mut resources.DATA;
        data.b = val;
        iprintln!(stim, "Start {:?}", data);

        match data.state {
            State::S8000 => {
                if data.a ^ data.b {
                    schedule
                        .discrepency(scheduled + TIMEOUT.cycles(), data.event_counter, data.state)
                        .unwrap();
                    data.out = false;
                    data.state = State::S8005;
                } else if !data.a & !data.b {
                    data.state = State::S8001
                } else {
                    return;
                }
            }
            State::S8001 => {
                if data.a & !data.b {
                    schedule
                        .discrepency(scheduled + TIMEOUT.cycles(), data.event_counter, data.state)
                        .unwrap();
                    data.out = false;
                    data.state = State::S8004;
                } else if !data.a & data.b {
                    schedule
                        .discrepency(scheduled + TIMEOUT.cycles(), data.event_counter, data.state)
                        .unwrap();
                    data.out = false;
                    data.state = State::S8014;
                } else if data.a & data.b {
                    data.out = true;
                    data.state = State::S8000;
                }
            }
            State::S8004 => {
                if !data.a {
                    data.event_counter += 1;
                    data.out = false;
                    data.state = State::S8001;
                } else if data.b {
                    data.event_counter += 1;
                    data.out = true;
                    data.state = State::S8000;
                } else {
                    data.out = false;
                }
            }
            State::S8014 => {
                if !data.b {
                    data.event_counter += 1;
                    data.out = false;
                    data.state = State::S8001;
                } else if data.a {
                    data.event_counter += 1;
                    data.out = true;
                    data.state = State::S8000;
                } else {
                    data.out = false;
                }
            }
            State::S8005 => {
                if !data.a & !data.b {
                    data.event_counter += 1;
                    data.out = false;
                    data.state = State::S8001;
                } else {
                    data.out = false;
                }
            }

            _ => {
                if !data.a & !data.b {
                    data.out = false;
                    data.state = State::S8001;
                } else {
                    data.out = false;
                }
            }
        }
    }

    #[task(priority = 1, resources = [ITM])]
    fn discrepency(counter: u32, state: State) {
        let stim = &mut resources.ITM.stim[0];
        iprintln!(stim, "counter {} state {:?}", counter, state);
        // if data.event_counter == resources.DATA.event_counter {
        //     iprintln!(stim, "timeout");
        //     // data.force_reinit =
        // }
    }

    extern "C" {
        fn EXTI0();
        fn EXTI1();
        fn EXTI2();
        fn EXTI3();
        fn EXTI4();
    }
};

// fn evaluate_equivalence(scheduled: Instant, data: &mut Data,  d: event_a::Spawn) {
//     data.EventCounter += 1;
//     if data.A ^ data.B {

//         // spawn.discrepency(scheduled + TIMEOUT.cycles(), data.EventCounter, data.A, data.B);

//     }
// }
//
//
//
//
//  invariants
//  out = true -> A & B
//
//  event A false -> A = false
//  event A true  -> A = true
//  event B false -> B = false
//  event B true  -> B = true
//
//  A ^ B -- TIMEOUT --> !(A & B) -> out = false
