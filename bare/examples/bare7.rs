//! bare7.rs
//! 
//! Serial echo
//!
//! What it covers:
//! - changing the clock using Rust code
//! - working with the svd2rust API
//! - working with the HAL (Hardware Abstraction Layer)
//! - USART polling (blocking wait)

#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::iprintln;
use cortex_m_rt::entry;
use nb::block;

extern crate stm32f4xx_hal as hal;
use crate::hal::prelude::*;
use crate::hal::serial::{config::Config, Serial};
use stm32f4xx_hal::stm32::{self, DWT, GPIOA, GPIOC, RCC};

#[entry]
fn main() -> ! {
    let mut c = hal::stm32::CorePeripherals::take().unwrap();
    let stim = &mut c.ITM.stim[0];
    iprintln!(stim, "bare7");

    let p = hal::stm32::Peripherals::take().unwrap();

    let rcc = p.RCC.constrain();

    // 16 MHz (default, all clocks)
    let clocks = rcc.cfgr.sysclk(84.mhz()).hclk(84.mhz()).pclk1(42.mhz()).pclk2(64.mhz()).freeze();

    let gpioa = p.GPIOA.split();

    let tx = gpioa.pa2.into_alternate_af7();
    let rx = gpioa.pa3.into_alternate_af7(); // try comment out
    // let rx = gpioa.pa3.into_alternate_af6(); // try uncomment

    let serial = Serial::usart2(
        p.USART2,
        (tx, rx),
        Config::default().baudrate(115_200.bps()),
        clocks,
    )
    .unwrap();

    // Separate out the sender and receiver of the serial port
    let (mut tx, mut rx) = serial.split();
    let gpioc = p.GPIOC.split();
    gpioc.pc9.into_alternate_af0().set_speed(hal::gpio::Speed::Low);

    loop {
        match block!(rx.read()) {
            Ok(byte) => {
                iprintln!(stim, "Ok {:?}", byte);
                tx.write(byte).unwrap(); 
            }
            Err(err) => {
                iprintln!(stim, "Error {:?}", err);
            }
        }
    }

}

// 0. Background reading:
//    STM32F401xD STM32F401xE, section 3.11
//    We have two AMBA High-performance Bus (AHB)
//    APB1 low speed bus (max freq 42 MHz)
//    APB2 high speed bus (max frex 84 MHz)
//
//    RM0368 Section 6.2
//    Some important/useful clock acronymes and their use:
//
//    SYSCLK - the clock that drives the `core`
//    HCLK   - the clock that drives the AMBA bus(es), memory, DMA, trace unit, etc.
//
//    Typically we set HCLK = SYSCLK / 1 (no prescale) for our applications
//
//    FCLK   - Free running clock runing at HCLK
//
//    CST    - CoreSystemTimer drives the SysTick counter, HCLK/(1 or 8)
//    PCLK1  - The clock driving the APB1 (<= 42 MHz)
//             Timers on the APB1 bus will be triggered at PCLK1 * 2
//    PCLK2  - The clock driving the APB2 (<= 84 MHz)
//             Timers on the APB2 bus will be triggered at PCLK2
//
//    Compliation:
//    > cargo build --example bare7 --features "hal"
//    (or use the vscode build task)
//
//    The "hal" feature enables the optional dependency stm32f4xx-hal".
//
//    Cargo.toml:
// 
//    [dependencies.stm32f4]
//    version = "0.5.0"
//    features = ["stm32f413", "rt"]
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
//
//    Notice, stm32f4xx-hal internally enables the dependency to stm32f4, 
//    so we don't need to explicitly enable it.
//
//    The HAL provides a generic abstraction over the whole stm32f4 family,
//    as previously we use the stm32f413, since it also covers stm32f401/stm32f411.
//
// 1. The rcc.cfgr.x.freeze() sets the clock according to the configuration x given.
//
//    rcc.cfgr.freeze(); sets a default configuration.
//    sysclk = hclk = pclk1 = pclk2 = 16MHz
//
//    What is wrong with the following configurations?
//
//    rcc.cfgr.sysclk(64.mhz()).pclk1(64.mhz()).pclk2(64.mhz()).freeze();
//
//    The max frequency for pclk1 is 42MHz so it won't work with specified 64.
//    HCLK should be set to 64MHz alsongside the sysclk. Above config will set it to default 16MHz which is wrong.
//
//    rcc.cfgr.sysclk(84.mhz()).pclk1(42.mhz()).pclk2(64.mhz()).freeze();
//
//    hclk needs to be set to 84MHz.
//
//    Commit your answers (bare7_1)
//
//    Tip: You may use `stm32cubemx` to get a graphical view for experimentation.
//    This is a good tool when designing your PCB (as pinouts are given).
//
// 2. Now give the system with a valid clock, sysclk of 84 MHz.
//
//    Include the code for outputting the clock to MCO2.
//
//    Repeat the expermient bare6_2.
//
//    What is the frequency of MCO2 read by the oscilloscope.
//
//    84MHz
//
//    Compute the value of SYSCLK based on the oscilloscope reading.
//
//    84MHz as specified in the code
//
//    What is the peak to peak reading of the signal.
//
//    10.1V
//
//    Make a screen dump or photo of the oscilloscope output.
//    Save the the picture as "bare_6_84mhz_high_speed"
//
//    Commit your answers (bare7_2)
//
// 3. Now reprogram the PC9 to be "Low Speed", and re-run at 84Mz.
//
//    Did the frequency change in comparison to assignment 5?
//
//    Unreadable it changed a lot. Only says "no edges"
//
//    What is the peak to peak reading of the signal (and why did it change)?
//
//    peak is 70mV since the signal is so low the peak to peak reading will be almost non-existant as well.
//
//    Make a screen dump or photo of the oscilloscope output.
//    Save the the picture as "bare_6_84mhz_low_speed".
//
//    Commit your answers (bare7_3)
//
// 4. Revisit the `README.md` regarding serial communication.
//    start a terminal program, e.g., `moserial`.
//    Connect to the port
//
//    Device       /dev/ttyACM0
//    Baude Rate   115200
//    Data Bits    8
//    Stop Bits    1
//    Parity       None
//    
//    This setting is typically abbreviated as 115200 8N1.
//
//    Run the example, make sure your ITM is set to 84MHz.
//
//    Send a single character (byte), (set the option No end in moserial).
//    Verify that sent bytes are echoed back, and that ITM tracing is working.
// 
//    If not go back check your ITM setting, clocks etc.
//
//    Try sending: "abcd" as a single sequence, don't send the quation marks, just abcd.
//
//    What did you receive, and what was the output of the ITM trace.
//
//    I recieve a neverending stream of abcdabcdabcdabcd etc..
//    The ITM trace outputs ok 97, ok 98, ok 99, ok 100 in a loop as well.
//
//    Explain why the buffer overflows.
//
//    It does not overflow, if it were to overflow it would be because it is trying to write bytes before the buffer could reset.
//
//    commit your answers (bare7_4)
//
//    Discussion:
//    Common to all MCUs is that they have multiple clocking options.
//    Understanding the possibilities and limitations of clocking is fundamental
//    to designing both the embedded hardware and software. Tools like
//    `stm32cubemx` can be helpful to give you the big picture.
//    You will likely finding it useful when designing your board (e.g., checking
//    what functionality can be associated to each pin etc.)
//
//    The `stm32f4xx-hal` gives you an abstraction for programming,
//    setting up clocks, assigning pins, etc.
//
//    The hal is under development, and already covers some basic functionality
//    like serial communication. Still, in order to fully understand what is
//    going on under the hood you need to check the documentation (data sheets,
//    user manuals etc.)
//
//    Your crate can be autmatically documentedthread:
//
//    $ cargo doc --open --features "hal"
//
//    This will document both your crate and its dependencies besides the `core` library.
//
//    You can open the `core` library documentation by
//
//    $ rustup doc
//
//    or just show the path to the doc (to open it manually)
//
//    $ rustup doc --path
