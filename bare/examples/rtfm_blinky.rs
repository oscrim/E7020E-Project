#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::peripheral::syst::SystClkSource;
use stm32f4::stm32f413::GPIOA;

use rtfm::app;

#[app(device = stm32f4::stm32f413)]
const APP: () = {
    // late resorce binding
    static mut GPIOA: GPIOA = ();

    // init runs in an interrupt free section
    #[init]
    fn init() {
        // configures the system timer to trigger a SysTick exception every second
        core.SYST.set_clock_source(SystClkSource::Core);
        core.SYST.set_reload(16_000_000); // period = 1s
        core.SYST.enable_counter();
        core.SYST.enable_interrupt();

        // power on GPIOA, RM0368 6.3.11
        device.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        // configure PA5 as output, RM0368 8.4.1
        device.GPIOA.moder.modify(|_, w| w.moder5().bits(1));

        // pass on late resources
        GPIOA = device.GPIOA;
    }

    #[exception (resources = [GPIOA])]
    fn SysTick() {
        static mut TOGGLE: bool = false;

        if *TOGGLE {
            resources.GPIOA.bsrr.write(|w| w.bs5().set_bit());
        } else {
            resources.GPIOA.bsrr.write(|w| w.br5().set_bit());
        }

        *TOGGLE = !*TOGGLE;
    }
};
