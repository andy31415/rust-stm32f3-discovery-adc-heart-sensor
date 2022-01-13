#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f3xx_hal::pac;
use stm32f3xx_hal::prelude::*;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello, world!");

    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let _clocks = rcc.cfgr.freeze(&mut flash.acr);

    loop {}
}
