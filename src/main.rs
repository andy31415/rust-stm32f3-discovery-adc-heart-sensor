#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;

use stm32f3xx_hal::pac;
use stm32f3xx_hal::prelude::*;
use stm32f3xx_hal::rcc::Clocks;

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
    rprintln!("Hello, world!");

    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let core_periphs = cortex_m::Peripherals::take().unwrap();

    cortex_m::interrupt::free(|cs| {
        TICK_COUNTER
            .borrow(cs)
            .borrow_mut()
            .replace(TickCounter::new(core_periphs.SYST, &clocks))
    });

    let mut counter = 0;
    loop {
        sleepms(1000);
        rprintln!("{}: Still running: {}", millis(), counter);
        counter += 1;
    }
}

#[cortex_m_rt::exception]
fn SysTick() {
    cortex_m::interrupt::free(|cs| {
        if let Some(counter) = &mut *TICK_COUNTER.borrow(cs).borrow_mut() {
            counter.tick();
        }
    })
}
