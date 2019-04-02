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
use crate::hal::serial::{config::Config, Serial};
use stm32f4xx_hal::gpio::Analog;
use stm32f4xx_hal::gpio::gpiob::PB0;
use stm32f4xx_hal::stm32::ADC1;

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
    static mut ITM: ITM                      = ();
    static mut ADC: Adc<ADC1>                = ();
    static mut PB0: PB0<Analog>              = ();
    // static mut SPI: Spi                    = ();
    // static mut CS:  PB12<Output<PushPull>> = ();
    // static mut CD:  PA6<Output<PushPull>>  = ();
    // static mut AF:  PA8<Output<PushPull>>  = ();
    // static mut TX:  Tx<hal::stm32::USART2> = ();
    // static mut RX:  Rx<hal::stm32::USART2> = ();

    #[init(schedule = [temp])]
    fn init() {
        let stim = &mut core.ITM.stim[0];
        iprintln!(stim, "PCB start");
        let rcc = device.RCC.constrain();

        // 16 MHz (default, all clocks)
        let clocks = rcc.cfgr.freeze();

        let gpioa = device.GPIOA.split();
        let gpiob = device.GPIOB.split();
        let gpioc = device.GPIOC.split();
        //---------------------------------------------------------
        let pb0_a = gpiob.pb0.into_analog();
        let pushB = gpiob.pb1.into_pull_down_input();
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
        
        cs.set_high();
        schedule.temp(Instant::now() + (16_000_000).cycles()).unwrap();
        
        // RX = rx;
        // TX = tx;
        // CS = cs;
        // CD = cd;
        // AF = af;
        // SPI = spi;
        
        ADC = adc;
        PB0 = pb0_a;
        ITM = core.ITM;
    }    



    #[idle]
    fn idle() -> ! {
        loop {
            asm::wfi();
        }
    }

    #[task(priority = 4, schedule = [temp], resources = [ITM, ADC, PB0], spawn = [temp])]
    fn temp(){
        let stim = &mut resources.ITM.stim[0];
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
        iprintln!(stim, "temperature : {:?}", temp_rounded);
        schedule.temp(Instant::now() + (32_000_000).cycles()).unwrap();
    }

    #[task(priority = 1, capacity = 3, resources = [ITM])]
    fn trace_data(byte: u8) {
        let stim = &mut resources.ITM.stim[0];
        iprintln!(stim, "data {}", byte);
        // for _ in 0..10000 {
        //     asm::nop();
        // }
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

    #[interrupt(priority = 3, resources = [RX], spawn = [trace_error, echo, interpreter])]
    fn USART2() {
        let rx = resources.RX;

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



    // Set of interrupt vectors, free to use for RTFM tasks
    // 1 per priority level suffices
    extern "C" {
        fn EXTI0();
        fn EXTI1();
        fn EXTI2();
    }

};



