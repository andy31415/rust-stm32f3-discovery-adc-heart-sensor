#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;

use core::cell::RefCell;

use cortex_m::{
    interrupt::Mutex,
    peripheral::{syst::SystClkSource, SYST},
};

use stm32f3xx_hal::{
    adc::{Adc, ClockMode},
    pac,
    prelude::*,
    rcc::Clocks,
};

struct TickCounter {
    #[allow(dead_code)]
    /// keep syst as a resource
    syst: SYST,
    millisconds: u64,
}

impl TickCounter {
    pub fn new(mut syst: SYST, clocks: &Clocks) -> TickCounter {
        syst.set_clock_source(SystClkSource::Core);

        // per ms counter
        syst.set_reload(clocks.hclk().0 / 1_000 - 1);
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();

        TickCounter {
            syst,
            millisconds: 0,
        }
    }

    pub fn tick(&mut self) {
        self.millisconds += 1;
    }

    pub fn get(&self) -> u64 {
        self.millisconds
    }
}

static TICK_COUNTER: Mutex<RefCell<Option<TickCounter>>> = Mutex::new(RefCell::new(None));

fn millis() -> u64 {
    cortex_m::interrupt::free(|cs| {
        if let Some(counter) = &*TICK_COUNTER.borrow(cs).borrow_mut() {
            counter.get()
        } else {
            0
        }
    })
}

fn sleepms(ms: u64) {
    let deadline = millis() + ms;
    while millis() < deadline {
        cortex_m::asm::nop();
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let core_periphs = cortex_m::Peripherals::take().unwrap();

    cortex_m::interrupt::free(|cs| {
        TICK_COUNTER
            .borrow(cs)
            .borrow_mut()
            .replace(TickCounter::new(core_periphs.SYST, &clocks))
    });

    let mut adc1 = Adc::adc1(
        dp.ADC1,
        &mut dp.ADC1_2,
        &mut rcc.ahb,
        ClockMode::SyncDiv4,
        clocks,
    );

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut pa2 = gpioa.pa2.into_analog(&mut gpioa.moder, &mut gpioa.pupdr);
    adc1.setup_oneshot();

    loop {
        sleepms(10);
        let value: u16 = adc1.read(&mut pa2).unwrap();
        rprintln!("{} {}", millis(), value);
    }
}

#[cortex_m_rt::exception]
#[allow(non_snake_case)]
fn SysTick() {
    cortex_m::interrupt::free(|cs| {
        if let Some(counter) = &mut *TICK_COUNTER.borrow(cs).borrow_mut() {
            counter.tick();
        }
    })
}
