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
use crate::hal::serial::{config::Config, Event, Rx, Serial, Tx};
use hal::stm32::ITM;

use nb::block;
use rtfm::app;

// Our error type
#[derive(Debug)]
pub enum Error {
    RingBufferOverflow,
    UsartSendOverflow,
    UsartReceiveOverflow,
}

#[app(device = hal::stm32)]
const APP: () = {
    // Late resources
    static mut TX: Tx<hal::stm32::USART2> = ();
    static mut RX: Rx<hal::stm32::USART2> = ();
    static mut ITM: ITM = ();

    // init runs in an interrupt free section>
    #[init]
    fn init() {
        let stim = &mut core.ITM.stim[0];
        iprintln!(stim, "bare10");

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
        serial.listen(Event::Rxne);
        // Separate out the sender and receiver of the serial port
        let (tx, rx) = serial.split();

        // Our split serial
        TX = tx;
        RX = rx;

        // For debugging
        ITM = core.ITM;
    }

    // idle may be interrupted by other interrupt/tasks in the system
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
        // for _ in 0..10000 {
        //     asm::nop();
        // }
    }

    #[task(priority = 1, resources = [ITM])]
    fn trace_error(error: Error) {
        let stim = &mut resources.ITM.stim[0];
        iprintln!(stim, "{:?}", error);
    }

    #[task(priority = 2, resources = [TX], spawn = [trace_error])]
    fn echo(byte: u8) {
        let tx = resources.TX;

        if block!(tx.write(byte)).is_err() {
            let _ = spawn.trace_error(Error::UsartSendOverflow);
        }

    }

    #[interrupt(priority = 3, resources = [RX], spawn = [trace_data, trace_error, echo])]
    fn USART2() {
        let rx = resources.RX;

        match rx.read() {
            Ok(byte) => {
                let _ = spawn.echo(byte);
                if spawn.trace_data(byte).is_err() {
                    let _ = spawn.trace_error(Error::RingBufferOverflow);
                }
            }
            Err(_err) => {
                let _ = spawn.trace_error(Error::UsartReceiveOverflow);
            }
        }
    }

    // Set of interrupt vectors, free to use for RTFM tasks
    // 1 per priority level suffices
    extern "C" {
        fn EXTI0();
        fn EXTI1();
    }
};

// 0. Compile and run the project at 16MHz in release mode
//    make sure its running (not paused).
//
//    > cargo build --example bare9 --features "hal rtfm" --release
//    (or use the vscode build task)
//
//    Connect a terminal program.
//    Verify that it works as bare9.
// 
// 1. Now, comment out the loop in `trace_data`.
//    The loop is just there to simulate some workload...
//  
//    Try now to send a sequence `abcd`
//
//    Did you loose any data (was the data correctly echoed)?
//
//    ** your answer here **
//
//    Was the data correctly traced over the ITM?
//
//    ** your answer here **
//
//    Why did you loose trace information?
//
//    ** your asnwer here **
//
//    Commit your answers (bare10_1)
//
// 2. Read the RTFM manual (book).
//    Figure out a way to accomdate for 4 outstanding messages to the `trace_data` task.
//
//    Verify that you can now correctly trace sequences of 4 characters sent.
//
//    Can you think of how to determine a safe bound on the message buffer size?
//    (Safe meaning, that message would never be lost due to the buffer being full.)
//
//    What information would you need?
//
//    ** your answer here **
//    
//    Commit your answers (bare10_2)
//
// 3. Implement a command line interpreter as a new task.
//    It should:  
//    - have priority 1.
//    - take a byte as an argument (passed from the USART2 interrupt).
//    - have a local buffer B of 10 characters
//    - have sufficent capacity to recieve 10 charactes sent in a sequence
//    - analyse the input buffer B checking the commands
//      set <int> <RETURN>       // to set blinking frequency
//      on <RETURN>              // to enable the led blinking
//      of <RETURN>              // to disable the led blinking
// 
//      <int> should be decoded to an integer value T, and <RETURN> accept either <CR> or <LF>.
//
//    The set value should blink the LED in according the set value in Herz, 
//    (so `set 1 <RETURN>` should blink with 1Hz)
//    
//    Tips:
//    Create two tasks, (`on', that turns the led on, and a task `off` that turns the led off).
//    `on` calls `off` with a timer offset (check the RTFM manual).
//    `off` calls `on` with a timer offset.
//
//    The timing offset can implemented as a shared resource T between the command line interpreter and
//    the  'on/off ' tasks. From `init` you can give an initial timing offset T, and send an
//    initial message to `on` triggering the periodic behavior. 
//
//    The 'on/off ' tasks can have a high priority 4, and use locking (in the low prio task)
//    parsing the input. This way, the led will have a very low jitter.
//
//    (You can even use an atomic data structure, which allows for lock free access.)
//    
//  
//    The on/off is easiest impemented by having another shared variable used as a condition
//    for the `on` task to set the GPIO. Other solutions could be to stop the sequence (i.e.)
//    contiditionally call the `off` task instead. Then the command interpreted would
//    trigger a new sequence when an "on" command is detected. (Should you allow multiple, overlapping)
//    sequences? Why not, could be cool ;)
//    The main advantage of the stopping approach is that the system will be truly idle
//    if not blinking, and thus more power efficient.
//
//    You can reuse the code for setting up and controlling the GPIO/led.
//
//    You can come up with various extensions to this application, setting the
//    the duty cycle (on/off ratio in %), etc.
//
//    Commit your solution (bare10_3)