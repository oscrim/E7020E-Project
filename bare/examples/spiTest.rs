#![no_main]
#![no_std]
extern crate cortex_m;
extern crate panic_halt;
extern crate stm32f4xx_hal as hal;
use crate::hal::prelude::*;
use cortex_m::{asm, iprintln};
use crate::hal::spi::{Spi, Mode, Phase, Polarity, NoMiso};
use rtfm::app;


#[app(device = hal::stm32)]
const APP: () = {

    #[init]
    fn init() {
        let stim = &mut core.ITM.stim[0];
        iprintln!(stim, "lcd");
        let rcc = device.RCC.constrain();

        // 16 MHz (default, all clocks)
        let clocks = rcc.cfgr.freeze();

        let gpioc = device.GPIOC.split();
        let sck = gpioc.pc10.into_alternate_af6();
        let mosi = gpioc.pc12.into_alternate_af6();
        //let mut mosi = gpioc.pc12.into_push_pull_output();
        let gpioa = device.GPIOA.split();
        let mut cd = gpioc.pc11.into_push_pull_output();
        let mut cs = gpioa.pa15.into_push_pull_output();
        //mosi.set_high();
        cd.set_low();
        cs.set_low();
        cs.set_high();
        let mode = Mode {
            polarity: Polarity::IdleHigh,
            phase: Phase::CaptureOnFirstTransition,
        };
        let mut spi = Spi::spi3(
            device.SPI3,
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
                spi.write(&[0xff]);
                for _ in 0..1000{
                    asm::nop;
                }

            } 
        }
}
#[idle]
fn idle() -> ! {
    loop {
        asm::wfi();
    }
}
};

