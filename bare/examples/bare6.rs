//! bare6.rs
//!
//! Clocking
//!
//! What it covers:
//! - using svd2rust generated API
//! - setting the clock via script (again)
//! - routing the clock to a PIN for monitoring by an oscilloscope

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::{iprintln, peripheral::itm::Stim};
use cortex_m_rt::entry;

use stm32f4::stm32f413::{self, DWT, GPIOA, GPIOC, RCC};

#[entry]
fn main() -> ! {
    let p = stm32f413::Peripherals::take().unwrap();
    let mut c = stm32f413::CorePeripherals::take().unwrap();

    let stim = &mut c.ITM.stim[0];
    iprintln!(stim, "bare6");

    c.DWT.enable_cycle_counter();
    unsafe {
        c.DWT.cyccnt.write(0);
    }
    let t = DWT::get_cycle_count();
    iprintln!(stim, "{}", t);

    clock_out(&p.RCC, &p.GPIOC);
    idle(stim, p.RCC, p.GPIOA);

    loop {}
}

// user application
fn idle(stim: &mut Stim, rcc: RCC, gpioa: GPIOA) {
    iprintln!(stim, "idle");

    // power on GPIOA, RM0368 6.3.11
    rcc.ahb1enr.modify(|_, w| w.gpioaen().set_bit());

    // configure PA5 as output, RM0368 8.4.1
    gpioa.moder.modify(|_, w| w.moder5().bits(1));

    // at 16 Mhz, 8_000_000 cycles = period 0.5s
    // at 64 Mhz, 4*8_000_000 cycles = period 0.5s
    // let cycles = 8_000_000;
    let cycles = 4 * 8_000_000;

    loop {
        iprintln!(stim, "on {}", DWT::get_cycle_count());
        // set PA5 high, RM0368 8.4.7
        gpioa.bsrr.write(|w| w.bs5().set_bit());
        wait_cycles(cycles);

        iprintln!(stim, "off {}", DWT::get_cycle_count());
        // set PA5 low, RM0368 8.4.7
        gpioa.bsrr.write(|w| w.br5().set_bit());
        wait_cycles(cycles);
    }
}

// uses the DWT.CYCNT
// doc: ARM trm_100166_0001_00_en.pdf, chapter 9.2
// we use the `cortex-m` abstraction, as re-exported by the stm32f40x
fn wait_cycles(nr_cycles: u32) {
    let t = DWT::get_cycle_count().wrapping_add(nr_cycles);
    while (DWT::get_cycle_count().wrapping_sub(t) as i32) < 0 {}
}

// see the Reference Manual RM0368 (www.st.com/resource/en/reference_manual/dm00096844.pdf)
// rcc,     chapter 6
// gpio,    chapter 8
fn clock_out(rcc: &RCC, gpioc: &GPIOC) {
    // output MCO2 to pin PC9

    // mco2 	: SYSCLK = 0b00
    // mcopre 	: divide by 4 = 0b110
    rcc.cfgr
        .modify(|_, w| unsafe { w.mco2().sysclk().mco2pre().bits(0b110) });
   
    // power on GPIOC, RM0368 6.3.11
    rcc.ahb1enr.modify(|_, w| w.gpiocen().set_bit());

    // MCO_2 alternate function AF0, STM32F401xD STM32F401xE data sheet
    // table 9
    // AF0, gpioc reset value = AF0

    // configure PC9 as alternate function 0b10, RM0368 6.2.10
    gpioc.moder.modify(|_, w| w.moder9().bits(0b10));
    
    // otyper reset state push/pull, in reset state (don't need to change)

    // ospeedr 0b11 = very high speed
    gpioc.ospeedr.modify(|_, w| w.ospeedr9().bits(0b11));
}

// 0. Compile and run the example, in 16Mhz
//
//    > cargo build --example bare6 --features "pac"
//    (or use the vscode build task)
//
//    The "pac" feature enables the optional dependency "stm32f4"
//
//    Cargo.toml:
// 
//    [dependencies.stm32f4]
//    version = "0.5.0"
//    features = ["stm32f413", "rt"]
//    optional = true
// 
//    [features]
//    pac = ["stm32f4"]    
//
//    Notice, we use the stm32f413 API, since it covers stm32f401/f411/f413.
//
// 1. The processor SYSCLK defaults to HCI 16Mhz
//    (this is what you get after a `monitor reset halt`).
//
//    Confirm that your ITM dump traces the init, idle and led on/off.
//    Make sure your TPIU is set to a system clock at 16Mhz
//
//    What is the frequency of blinking?
//
//    about 2 seconds between toggles.
//
//    commit your answers (bare6_1)
//
// 2. Now connect an oscilloscope to PC9, which is set to
//    output the MCO2.
//
//    What is the frequency of MCO2 read by the oscilloscope?
//
//    4MHz
//
//    Compute the value of SYSCLK based on the oscilloscope reading
//
//    4 * 4 (divide by 4 so need to multiply now) = 16MHz
//
//    What is the peak to peak reading of the signal?
//
//    5.8V
//
//    Make a folder called "pictures" in your git project.
//    Make a screen dump of the oscilloscope output.
//    Save the the picture as "bare_6_16mhz_high_speed".
//
//    Commit your answers (bare6_2)
//
// 3. Now run the example in 64Mz
//    You can do that by issuing a `monitor reset init`
//    which reprograms SYSCLK to 4*HCI.
//
//
//    Confirm that your ITM dump traces the init, idle and led on/off
//    (make sure your TPIU is set to a system clock at 64Mhz)
//
//    Uncommnet: `let cycles = 4 * 8_000_000;
//`
//    What is the frequency of blinking?
//
//    ~0.5 sec between each toggle
//
//    Commit your answers (bare6_3)
//
// 4. Repeat experiment 2
//
//    What is the frequency of MCO2 read by the oscilloscope?
//
//    16MHz
//
//    Compute the value of SYSCLK based on the oscilloscope reading.
//
//    16 * 4 = 64MHz
//
//    What is the peak to peak reading of the signal?
//
//    6
//
//    Make a screen dump or photo of the oscilloscope output.
//    Save the the picture as "bare_6_64mhz_high_speed".
//
//    Commit your answers (bare6_4)
//
// 5. In the `clock_out` function, the setup of registers is done through
//    setting bitpattens manually, e.g. 
//     rcc.cfgr
//        .modify(|_, w| unsafe { w.mco2().bits(0b00).mco2pre().bits(0b110) });
//
//    However based on the vendor SVD file the svd2rust API provides  
//    a better abstraction, based on pattern enums and functions.
//
//    To view the API you can generate documentation for your crate:
//
//    > cargo doc --features "pac" --open
//
//    By searching for `mco2` you find the enumerations and functions.
//    So here 
//       `w.mco2().bits{0b00}` is equivalent to 
//       `w.mco2().sysclk()` and improves readabiity. 
//
//    Replace all bitpatterns used by the function name equivalents.
//
//    Test that the application still runs as before.
//
//    Commit your code (bare6_4)