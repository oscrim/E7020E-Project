#![no_main]
#![no_std]
extern crate cortex_m;
extern crate panic_halt;
extern crate stm32f4xx_hal as hal;
use crate::hal::prelude::*;
use hal::stm32::ITM; //
use cortex_m::{asm, iprintln};
use hal::stm32::{GPIOA, GPIOB, GPIOC};
use crate::hal::spi::{Spi, Mode, Phase, Polarity, NoMiso};

use nb::block;
use rtfm::app;
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
    static mut ITM: ITM = ();

    #[init]
    fn init() {
        let stim = &mut core.ITM.stim[0];
        iprintln!(stim, "bigboy");
        let rcc = device.RCC.constrain();

        // 16 MHz (default, all clocks)
        let clocks = rcc.cfgr.freeze();

        let gpiob = device.GPIOB.split();
        let gpioc = device.GPIOC.split();
        //---------------------------------------------------------
        let pb0_a = gpiob.pb0.into_analog();
        let pushB = gpiob.pb1.into_pull_down_input();
        let mut relay = gpioc.pc12.into_push_pull_output();
        let mut adc = Adc::adc1(device.ADC1, true, AdcConfig::default());
        //---------------------------------------------------------
        //let sck = gpioc.pc10.into_alternate_af6();
        //let mosi2 = gpioc.pc12.into_alternate_af6();
        ////let mut mosi = gpioc.pc12.into_push_pull_output();
        let sck = gpiob.pb13.into_alternate_af5();
        let mosi = gpiob.pb15.into_alternate_af5();
        let gpioa = device.GPIOA.split();
        //let mut cd = gpioc.pc11.into_push_pull_output();
        //let mut cs = gpioa.pa15.into_push_pull_output();
        let mut cd = gpioa.pa6.into_push_pull_output();
        let mut cs = gpiob.pb12.into_push_pull_output();
        let mut af = gpioa.pa8.into_push_pull_output();
        af.set_high();
        //mosi.set_high();
        cd.set_low();
        cs.set_low();
        cs.set_high();
        //mosi2.set_high();
        let mode = Mode {
            polarity: Polarity::IdleHigh,
            phase: Phase::CaptureOnFirstTransition,
        };
        let mut spi = Spi::spi2(
            device.SPI2,
            (sck, NoMiso, mosi),
            mode,
            10_000_000.hz(),
            clocks
        );
        
        cs.set_low();
        //let data = 
        spi.write(&[0x40, 0xA1, 0xC0, 0xA4, 0xA6, 0xA2, 0x2F, 0x27, 0x81, 0x10, 0xFA, 0x90, 0xAF]);
        /*match data {
            Ok(v) => iprintln!(stim, "working with version: {:?}", v),
            Err(e) => iprintln!(stim, "error parsing header: {:?}", e),
        }*/
        
        cs.set_high();
        loop {
            if unsafe { (*GPIOB::ptr()).idr.read().idr1().bit_is_set() } == false{
                relay.set_high();
                for page in 0..8 {
                    //asm::bkpt();
                    cs.set_low();
                    cd.set_low();
                    let msb_adress = 0x10 + (0>>4);
                    let lsb_adress = 0x00 + (0&0x0F);
                    let adress_page = 0xB0 + (page&0x0F);
                    spi.write(&[msb_adress, lsb_adress, adress_page]);
                    cd.set_high();
                    for _ in 0..102 {
                        spi.write(&[0xff]);
                        af.set_high();
                        for _ in 0..400{
                            asm::nop;
                        }
                        af.set_low();
                    } 
                }         
                for page in 0..8 {
                    //asm::bkpt();
                    cs.set_low();
                    cd.set_low();
                    
                    let msb_adress = 0x10 + (0>>4);
                    let lsb_adress = 0x00 + (0&0x0F);
                    let adress_page = 0xB0 + (page&0x0F);
                    spi.write(&[msb_adress, lsb_adress, adress_page]);
                    cd.set_high();
                    for _ in 0..102 {
                        spi.write(&[0x00]);
                        for _ in 0..400{
                            af.set_high();
                            asm::nop;
                            af.set_low();
                        }
                    } 
                } 
                relay.set_low();
            }
            else{
                iprintln!(stim, "click!");
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
            }
        }
        ITM = core.ITM;
    }    
    #[idle]
    fn idle() -> ! {
        loop {
            asm::wfi();
        }
    }

};
/*fn samplingF(adc: Adc, stim: &mut ITM, pb0_a: GPIOB) -> !{
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
    }
*/



