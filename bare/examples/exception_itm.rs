//! Overriding an exception handler
//!
//! You can override an exception handler using the [`#[exception]`][1] attribute.
//!
//! [1]: https://rust-embedded.github.io/cortex-m-rt/0.6.1/cortex_m_rt_macros/fn.exception.html
//!
//! Notice, steal will panic! in debug mode, due to a `debug_assert` (ignored in release).
//! ---

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::{iprint, iprintln, Peripherals};
use cortex_m_rt::{entry, exception};

#[entry]
fn main() -> ! {
    let mut p = Peripherals::take().unwrap();
    let mut syst = p.SYST;
    let stim = &mut p.ITM.stim[0];
    iprintln!(stim, "exception_itm");

    // configures the system timer to trigger a SysTick exception every second
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(16_000_000); // period = 1s
    syst.enable_counter();
    syst.enable_interrupt();

    loop {}
}

#[exception]
fn SysTick() {
    // Here we steal all the peripherals.
    //
    // This is unsafe, as some other task/tasks may access the peripherals
    // simultaneously, causing a conflict/race.
    //
    // The operation is checked in `debug` mode but not in release
    // using a `debug_assert`. As the periherals is already taken
    // by main, we need to compile in `--release` mode, to avoid a panic.
    let mut p = unsafe { cortex_m::Peripherals::steal() };
    let stim = &mut p.ITM.stim[0];
    iprint!(stim, ".");
}
