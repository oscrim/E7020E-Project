// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_halt;

extern crate stm32f4;
//extern crate stm32f4xx_hal as hal;

// use crate::hal::stm32::Interrupt::EXTI0;
use rtfm::app;
// use hal::stm32::Interrupt::EXTI0;

#[app(device = stm32f4::stm32f413)]

const APP: () = {
    // init runs in an interrupt free section
    #[init]
    fn init() {}

    #[interrupt]
    fn EXTI0() {}

    #[interrupt]
    fn USART2() {}
};
