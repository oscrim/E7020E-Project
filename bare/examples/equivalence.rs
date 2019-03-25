//! Implementiation of the "Equivalence" safety function
//! According to the PLCOpen standard
//! 

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::{asm, iprintln};

extern crate stm32f4xx_hal as hal;
use crate::hal::prelude::*;
use crate::hal::serial::{self, config::Config, Rx, Serial, Tx};
use hal::{
    gpio::{gpioa, Output, PushPull},
    stm32::ITM,
};

use nb::block;

use rtfm::app;

// Our error type
#[derive(Debug)]
pub enum Error {
    RingBufferOverflow,
    UsartSendOverflow,
    UsartReceiveOverflow,
}

#[derive(Debug, Copy, Clone)]
pub struct Data {
    a: bool,
    b: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

use State::*;

const PERIOD: u32 = 16_000_000;
const DISCREPENCY: u32 = 10;
const T: bool = true;
const F: bool = false;

#[app(device = hal::stm32)]
const APP: () = {
    // Late resources
    static mut TX: Tx<hal::stm32::USART2> = ();
    static mut RX: Rx<hal::stm32::USART2> = ();
    static mut ITM: ITM = ();
    static mut LED: gpioa::PA5<Output<PushPull>> = ();

    static mut DATA: Data = Data { a: F, b: F };

    // init runs in an interrupt free section>
    #[init(resources = [DATA], schedule = [periodic])]
    fn init() {
        let stim = &mut core.ITM.stim[0];
        iprintln!(stim, "start: {:?}", resources.DATA);

        let rcc = device.RCC.constrain();

        // 16 MHz (default, all clocks)
        let clocks = rcc.cfgr.freeze();

        let gpioa = device.GPIOA.split();

        let tx = gpioa.pa2.into_alternate_af7();
        let rx = gpioa.pa3.into_alternate_af7();
        let led = gpioa.pa5.into_push_pull_output();

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
        // Start periodic task
        schedule.periodic(start).unwrap();

        // pass on late resources
        LED = led;

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

    #[interrupt(priority = 2, resources = [RX], spawn = [trace_data, trace_error, echo])]
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

    #[task(priority = 1, resources = [TX, DATA], spawn = [trace_error])]
    fn echo(byte: u8) {
        let tx = resources.TX;

        if block!(tx.write(byte)).is_err() {
            spawn.trace_error(Error::UsartSendOverflow).unwrap();
        }

        let mut data = resources.DATA;

        match byte {
            b'a' => data.a = false,
            b'b' => data.b = false,
            b'A' => data.a = true,
            b'B' => data.b = true,
            _ => (),
        }
    }

    #[task (priority = 1, resources = [LED, ITM, DATA], schedule = [periodic])]
    fn periodic() {
        // we start directly in the init state S80001
        static mut STATE: State = State::S8001;
        static mut TIMEOUT: u32 = 0;

        let stim = &mut resources.ITM.stim[0];
        let data = resources.DATA;

        iprintln!(stim, "Old State {:?}", STATE);
        iprintln!(stim, "Timeout {:?}", TIMEOUT);
        iprintln!(stim, "{:?}", data);

        *STATE = match STATE {
            S8000 => match (data.a, data.b) {
                (F, F) => S8001,
                (F, T) | (T, F) => {
                    *TIMEOUT = DISCREPENCY;
                    S8005
                }
                (T, T) => S8000,
            },
            S8001 => match (data.a, data.b) {
                (F, F) => S8001,
                (F, T) => {
                    *TIMEOUT = DISCREPENCY;
                    S8014
                }
                (T, F) => {
                    *TIMEOUT = DISCREPENCY;
                    S8004
                }
                (T, T) => S8000,
            },
            S8004 => {
                *TIMEOUT -= 1;
                match *TIMEOUT {
                    0 => C001,
                    _ => match (data.a, data.b) {
                        (F, _) => S8001,
                        (_, T) => S8000,
                        _ => S8004,
                    },
                }
            }
            S8014 => {
                *TIMEOUT -= 1;
                match *TIMEOUT {
                    0 => C002,
                    _ => match (data.a, data.b) {
                        (_, F) => S8001,
                        (T, _) => S8000,
                        _ => S8014,
                    },
                }
            }
            S8005 => {
                *TIMEOUT -= 1;
                match *TIMEOUT {
                    0 => C003,
                    _ => match (data.a, data.b) {
                        (F, F) => S8001,
                        _ => S8005,
                    },
                }
            }
            C001 | C002 => match (data.a, data.b) {
                (F, F) => S8001,
                _ => C002,
            },
            C003 => match (data.a, data.b) {
                (F, F) => S8001,
                _ => C003,
            },
        };
        iprintln!(stim, "New State {:?}\n", STATE);

        if *STATE == S8000 {
            resources.LED.set_high();
        } else {
            resources.LED.set_low();
        }

        schedule.periodic(scheduled + PERIOD.cycles()).unwrap();
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
