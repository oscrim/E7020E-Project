#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::{asm, iprintln};


extern crate stm32f4xx_hal as hal;
use crate::hal::prelude::*;
use hal::stm32::ITM;

use nb::block;
use rtfm::{app};
use stm32f4xx_hal::{
  gpio::gpiob,
  adc::{
    Adc,
    config::AdcConfig,
    config::SampleTime,
  },
};

#[app(device = hal::stm32)]
const APP: () = {
    

    #[init]
    fn init() {

        let stim = &mut core.ITM.stim[0];
        iprintln!(stim, "temp test");

        let rcc = device.RCC.constrain();

        // 16 MHz (default, all clocks)
        let clocks = rcc.cfgr.freeze();

        let gpiob = device.GPIOB.split();
        let pb0_a = gpiob.pb0.into_analog();
        let mut adc = Adc::adc1(device.ADC1, true, AdcConfig::default());

        loop {
          let sample = adc.convert(&pb0_a, SampleTime::Cycles_480);
          let millivolts = adc.sample_to_millivolts(sample);

          iprintln!(stim, "millivolts: {:?}", millivolts);

          let temp = -0.1805*(millivolts as f64) + 186.88;

          iprintln!(stim, "temp before rounding: {:?}", temp);

          let temp_rounded: i32;

          if temp > 0.0 {
            temp_rounded = (temp + 0.5) as i32;
          }
          else {
            temp_rounded = (temp - 0.5) as i32;
          }

          iprintln!(stim, "temp after rounding: {:?}", temp_rounded);

          for _ in 0..10_000 {
          cortex_m::asm::nop(); // no operation (cannot be optimized out)
          }
        }
    }
};