//! bare8.rs
//!
//! The RTFM framework
//!
//! What it covers:
//! - utilisizing the RTFM framework for serial communicaton
//! - singletons (enteties with a singe instance)
//! - owned resources
//! - peripheral access in RTFM
//! - polling in `idle`

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::{asm, iprintln};
use nb::block;

extern crate stm32f4xx_hal as hal;
use crate::hal::prelude::*;
use crate::hal::serial::{config::Config, Rx, Serial, Tx};
use hal::stm32::{ITM, USART2};

use rtfm::app;

#[app(device = hal::stm32)]
const APP: () = {
    // Late resources
    static mut TX: Tx<USART2> = ();
    static mut RX: Rx<USART2> = ();
    static mut ITM: ITM = ();

    // init runs in an interrupt free section
    #[init]
    fn init() {
        let stim = &mut core.ITM.stim[0];
        iprintln!(stim, "bare8");

        let rcc = device.RCC.constrain();

        // 16 MHz (default, all clocks)
        let clocks = rcc.cfgr.freeze();

        let gpioa = device.GPIOA.split();

        let tx = gpioa.pa2.into_alternate_af7();
        let rx = gpioa.pa3.into_alternate_af7(); 

        asm::bkpt();

        let serial = Serial::usart2(
            device.USART2,
            (tx, rx),
            Config::default().baudrate(115_200.bps()),
            clocks,
        )
        .unwrap();

        // Separate out the sender and receiver of the serial port
        let (tx, rx) = serial.split();

        // Late resources
        TX = tx;
        RX = rx;
        ITM = core.ITM;
    }

    // idle may be interrupted by other interrupt/tasks in the system
    #[idle(resources = [RX, TX, ITM])]
    fn idle() -> ! {
        let rx = resources.RX;
        let tx = resources.TX;
        let stim = &mut resources.ITM.stim[0];
        let mut errors = 0;
        let mut received = 0;


        loop {
            match block!(rx.read()) {
                Ok(byte) => {
                    received += 1;
                    iprintln!(stim, "bytes received {:?}", received);
                    iprintln!(stim, "Ok {:?}", byte);
                    tx.write(byte).unwrap(); 
                }
                Err(err) => {
                    iprintln!(stim, "Error {:?}", err);
                    errors += 1;
                    iprintln!(stim, "errors {:?}", errors);
                }
            }
        }
    }
};


// 0. Compile and run the example. Notice, we use the default 16MHz clock.
//
//    > cargo build --example bare7 --features "hal rtfm"
//    (or use the vscode build task)
//
//    The "hal" feature enables the optional dependencies stm32f4xx-hal and rtfm".
//
//    Cargo.toml:
// 
//    [dependencies.cortex-m-rtfm]
//    version = "0.4.0"
//    optional = true
// 
//    [dependencies.stm32f4xx-hal]
//    git = "https://github.com/stm32-rs/stm32f4xx-hal.git"
//    version = "0.2.8"
//    features = ["stm32f413", "rt"]
//    optional = true
//
//    [features]
//    pac = ["stm32f4"]
//    hal = ["stm32f4xx-hal"]
//    rtfm = ["cortex-m-rtfm"]
//
// 1. Our CPU now runs slower, did it effect the behavior?
//
//    When sending the abcd string it sends ababababab then gets stuck just sending 'b'. ITM tracing prints out 'error overrun.
//
//    Commit your answer (bare8_1)
//
// 2. Add a local variable `received` that counts the number of bytes received.
//    Add a local variable `errors` that counts the number of errors.
//
//    Adjust the ITM trace to include the additional information.
//
//    Commit your development (bare8_2)
//
// 3. The added tracing, how did that effect the performance,
//    (are you know loosing more data)?
//
//    It seems as though when you have the textra ITM tracing it will display an overrun error 1 byte earlier.
//    If I remove the tracing I can send though 2 bytes before error, when I put the tracing in I can only send thourgh 1 byte.
// 
//    Commit your answer (bare8_3)
//
// 4. *Optional
//    Compile and run the program in release mode.
//    If using vscode, look at the `.vscode` folder `task.json` and `launch.json`,
//    and add a new "profile" (a bit of copy paste).
//
//    How did the optimized build compare to the debug build (performance/lost bytes)
//
//    ** your answer here **
// 
//    Commit your answer (bare8_4)

