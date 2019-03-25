//! Overriding an exception handler
//!
//! You can override an exception handler using the [`#[exception]`][1] attribute.
//!
//! [1]: https://rust-embedded.github.io/cortex-m-rt/0.6.1/cortex_m_rt_macros/fn.exception.html
//!

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::peripheral::{syst::SystClkSource, ITM};
use cortex_m::{iprint, iprintln, Peripherals};
use cortex_m_rt::{entry, exception};

#[entry]
fn main() -> ! {
    let mut p = Peripherals::take().unwrap();
    let mut syst = p.SYST;
    let stim = &mut p.ITM.stim[0];
    iprintln!(stim, "exception_itm_raw");

    // configures the system timer to trigger a SysTick exception every second
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(16_000_000); // period = 1s
    syst.enable_counter();
    syst.enable_interrupt();

    loop {}
}

#[exception]
fn SysTick() {
    // here we access `ITM` using a *raw* pointer
    // this is unsafe, as there may be other tasks accessing the peripheral
    // simultaneously (and that might cause a conflict/race)
    let itm = unsafe { &mut *ITM::ptr() };
    let stim = &mut itm.stim[0];
    iprint!(stim, ".");
}
