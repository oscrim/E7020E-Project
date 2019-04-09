#![no_main]
#![no_std]
extern crate cortex_m;
extern crate panic_halt;
extern crate stm32f4xx_hal as hal;
use crate::hal::prelude::*;
use cortex_m::{asm, iprintln, peripheral::Peripherals, peripheral::itm::Stim};

use hal::stm32::{GPIOA, GPIOB, GPIOC, EXTI, ITM, ADC1, SPI2};
use crate::hal::spi::{Spi, Mode, Phase, Polarity, NoMiso};
use crate::hal::serial::{config::Config, Serial};
use stm32f4xx_hal::gpio::Analog;
use stm32f4xx_hal::gpio::gpiob::PB0;
use stm32f4xx_hal::gpio::PushPull;
use stm32f4xx_hal::gpio::Output;
use stm32f4xx_hal::gpio::gpioa::PA8;
use stm32f4xx_hal::gpio::gpioa::PA6;
use stm32f4xx_hal::gpio::gpiob::PB12;
use stm32f4xx_hal::gpio::AF5;
use stm32f4xx_hal::gpio::Alternate;
use stm32f4xx_hal::gpio::gpiob::PB15;
use stm32f4xx_hal::gpio::gpiob::PB13;
use stm32f4xx_hal::serial::Tx;
use stm32f4xx_hal::serial::Rx;
use stm32f4xx_hal::interrupt;

use nb::block;
use rtfm::{app, Instant};
use stm32f4xx_hal::{ 
  gpio::gpiob,
  adc::{
    Adc,
    config::AdcConfig,
    config::SampleTime,
  },
};

// Our error type
#[derive(Debug)]
pub enum Error {
    RingBufferOverflow,
    UsartSendOverflow,
    UsartReceiveOverflow,
}

#[app(device = hal::stm32)]
const APP: () = {
    static mut EXTI:   EXTI                                                            = ();
    static mut ITM:    ITM                                                             = ();
    static mut ADC:    Adc<ADC1>                                                       = ();
    static mut PB0:    PB0<Analog>                                                     = ();
    static mut SPI:    Spi<SPI2, (PB13<Alternate<AF5>>, NoMiso, PB15<Alternate<AF5>>)> = ();
    static mut CS:     PB12<Output<PushPull>>                                          = ();
    static mut CD:     PA6<Output<PushPull>>                                           = ();
    static mut AF:     PA8<Output<PushPull>>                                           = ();
    static mut TX:     Tx<hal::stm32::USART2>                                          = ();
    static mut RX:     Rx<hal::stm32::USART2>                                          = ();

    #[init(schedule = [temp, screen])]
    fn init() {
        let stim = &mut core.ITM.stim[0];
        iprintln!(stim, "PCB start");

        let gpioa = device.GPIOA.split();
        let gpiob = device.GPIOB.split();
        let gpioc = device.GPIOC.split();
        //---------------------------------------------------------
        let pb0_a = gpiob.pb0.into_analog();
        let pushB = gpiob.pb1.into_pull_down_input();

        let rcc = device.RCC;
            rcc.ahb1enr.modify(|_, w| w.gpioben().set_bit());
            rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());

        let syscfg =  device.SYSCFG;
            syscfg.exticr1.modify(|_, w| unsafe {w.exti1().bits(0b001) });

        let exti = device.EXTI;
            exti.imr.modify(|_, w| w.mr1().set_bit());
            exti.rtsr.modify(|_, w| w.tr1().set_bit());
            exti.ftsr.modify(|_, w| w.tr1().clear_bit());

        let rcc = rcc.constrain();

        // 16 MHz (default, all clocks)
        let clocks = rcc.cfgr.freeze();
            
        let mut relay = gpioc.pc12.into_push_pull_output();
        let adc = Adc::adc1(device.ADC1, true, AdcConfig::default());
        //---------------------------------------------------------
        //let sck = gpioc.pc10.into_alternate_af6();
        //let mosi2 = gpioc.pc12.into_alternate_af6();
        ////let mut mosi = gpioc.pc12.into_push_pull_output();
        let sck = gpiob.pb13.into_alternate_af5();
        let mosi = gpiob.pb15.into_alternate_af5();
        //let mut cd = gpioc.pc11.into_push_pull_output();
        //let mut cs = gpioa.pa15.into_push_pull_output();
        let mut cd = gpioa.pa6.into_push_pull_output();
        let mut cs = gpiob.pb12.into_push_pull_output();
        let mut af = gpioa.pa8.into_push_pull_output();
        let tx = gpioa.pa2.into_alternate_af7();
        let rx = gpioa.pa3.into_alternate_af7();
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
        
        let serial = Serial::usart2(
            device.USART2,
            (tx, rx),
            Config::default().baudrate(115_200.bps()),
            clocks,
        ).unwrap();

        let (mut tx, mut rx) = serial.split();


        cs.set_low();
        //let data = 
        spi.write(&[0x40, 0xA1, 0xC0, 0xA4, 0xA6, 0xA2, 0x2F, 0x27, 0x81, 0x10, 0xFA, 0x90, 0xAF]);
        /*match data {
            Ok(v) => iprintln!(stim, "working with version: {:?}", v),
            Err(e) => iprintln!(stim, "error parsing header: {:?}", e),
        }*/

        for page in 0..8 {
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
        //schedule.screen(Instant::now() + (32_000_000).cycles()).unwrap();

        RX = rx;
        TX = tx;
        CS = cs;
        CD = cd;
        AF = af;
        SPI = spi;
        
        ADC = adc;
        PB0 = pb0_a;
        EXTI = exti;
        ITM = core.ITM;
        
    }    

    #[idle]
    fn idle() -> ! {
        loop {
            asm::wfi();
        }
    }

    #[task(priority = 4, schedule = [screen], resources = [CS, CD, SPI, AF], spawn = [screen])]
    fn screen(){
        let mut spi = resources.SPI;
        let mut cs = resources.CS;
        let mut cd = resources.CD;
        let mut af = resources.AF;
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
        schedule.screen(Instant::now() + (8_000_000).cycles()).unwrap();
    }

    #[task(priority = 1, resources = [ITM, ADC, PB0])]
    fn temp(){
        let adc = resources.ADC;
        let pb0_a = resources.PB0;

        let sample = adc.convert(pb0_a, SampleTime::Cycles_480);
        let millivolts = adc.sample_to_millivolts(sample);
        //iprintln!(stim, "millivolts: {:?}", millivolts);
        let temp = -0.1805*(millivolts as f64) + 186.88;
        //iprintln!(stim, "temp before rounding: {:?}", temp);
        let temp_rounded: i32;
        if temp > 0.0 {
            temp_rounded = (temp + 0.5) as i32;
        }
        else {
            temp_rounded = (temp - 0.5) as i32;
        }
        resources.ITM.lock(|itm| {
            let stim = &mut itm.stim[0];
            iprintln!(stim, "temperature : {:?}", temp_rounded);
        });
        
    }

    #[task(priority = 1, resources = [ITM], capacity = 3)]
    fn trace_data(byte: u8) {
        let stim = &mut resources.ITM.stim[0];
        iprintln!(stim, "data {}", byte);
    }

    #[task(priority = 1, resources = [ITM])]
    fn trace_error(error: Error) {
        let stim = &mut resources.ITM.stim[0];
        iprintln!(stim, "{:?}", error);
    }

    #[task(priority = 2, resources = [TX], spawn = [trace_error])]
    fn echo(byte: u8) {
        let tx = resources.TX;

        if block!(tx.write(byte)).is_err() {
            let _ = spawn.trace_error(Error::UsartSendOverflow);
        }
    }

    #[task(priority = 1, capacity = 10, resources = [ITM], spawn = [temp])]
    fn interpreter(byte: u8){
        static mut B: [u8; 10] = [0u8; 10];
        static mut i: usize = 0;
        let stim = &mut resources.ITM.stim[0];
        
        B[*i] = byte;
        *i = *i + 1;

        //tmp: 116 109 112
        if (B[0] == 116) && (B[1] == 109) && (B[2] == 112) && (B[3] == 13){
            spawn.temp().unwrap();
            iprintln!(stim, "USB Temp");
            B.iter_mut().for_each(|x| *x = 0);
            *i = 0;
        }
        
        else if (B.contains(&13) && B.contains(&10)){
            //Resets B!
            //iprintln!(stim, "Reset");
            B.iter_mut().for_each(|x| *x = 0);
            *i = 0;
        }
    }

    #[interrupt(priority = 1, resources = [RX, ITM], spawn = [trace_error, echo, interpreter])] 
    fn USART2() {
        let rx = resources.RX;
        resources.ITM.lock(|itm| {
            let stim = &mut itm.stim[0];
            iprintln!(stim, "USART");
        });

        match rx.read() {
            Ok(byte) => {
                let _ = spawn.echo(byte);
                if spawn.interpreter(byte).is_err() {
                    let _ = spawn.trace_error(Error::RingBufferOverflow);
                }
            }
            Err(_err) => {
                let _ = spawn.trace_error(Error::UsartReceiveOverflow);
            }
        }
    }

    #[interrupt(priority = 2, resources = [EXTI], spawn = [temp])]
    fn EXTI1(){
        spawn.temp();
        resources.EXTI.pr.modify(|_, w| w.pr1().set_bit());                                                      
    }



    // Set of interrupt vectors, free to use for RTFM tasks
    // 1 per priority level suffices
    extern "C" {
        fn EXTI0();
        fn EXTI2();
        fn EXTI3();
        fn EXTI4();
    }

};



