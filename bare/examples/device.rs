//! Using a device crate
//!
//! Crates generated using [`svd2rust`] are referred to as device crates. These crates provide an
//! API to access the peripherals of a device.
//!
//! [`svd2rust`]: https://crates.io/crates/svd2rust
//!
//! Device crates also provide an `interrupt!` macro (behind the "rt" feature) to register interrupt
//! handlers.
//!
//! This example depends on the [`stm32f103xx`] crate so you'll have to add it to your Cargo.toml.
//!
//! [`stm32f103xx`]: https://crates.io/crates/stm32f103xx
//!
//! ---

#![no_main]
#![no_std]

#[allow(unused_extern_crates)]
extern crate panic_halt;

use cortex_m::{iprint, peripheral::syst::SystClkSource};
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use stm32f4::stm32f413::{interrupt, Interrupt, ITM, NVIC};

#[entry]
fn main() -> ! {
    let p = cortex_m::Peripherals::take().unwrap();

    let mut syst = p.SYST;
    let mut nvic = p.NVIC;

    nvic.enable(Interrupt::EXTI0);

    // configure the system timer to wrap around every second
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(16_000_000); // 1s
    syst.enable_counter();

    loop {
        // busy wait until the timer wraps around
        while !syst.has_wrapped() {}

        // trigger the `EXTI0` interrupt

        NVIC::pend(Interrupt::EXTI0);
    }
}

// try commenting out this line: you'll end in `default_handler` instead of in `exti0`
#[interrupt]
fn EXTI0() {
    hprintln!(".").unwrap();
    let itm = unsafe { &mut *ITM::ptr() };
    let stim = &mut itm.stim[0];
    iprint!(stim, ".");
}
