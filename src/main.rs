#![no_main]
#![no_std]

use panic_semihosting as _;
use cortex_m_rt::entry;
use synth::traits::MonoGenerator;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

use daisy_bsp as daisy;
use daisy::hal;
use hal::prelude::*;

use daisy::led::Led;
use daisy::audio;

use daisy::pac;
use pac::interrupt;

use crate::synth::oscillators::{Oscillator, OscillatorMode};

mod synth;


// - static global state ------------------------------------------------------

static AUDIO_INTERFACE: Mutex<RefCell<Option<audio::Interface>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // - board setup ----------------------------------------------------------

    let board = daisy::Board::take().unwrap();
    let dp = daisy::pac::Peripherals::take().unwrap();

    let ccdr = board.freeze_clocks(dp.PWR.constrain(),
                                   dp.RCC.constrain(),
                                   &dp.SYSCFG);

    let pins = board.split_gpios(dp.GPIOA.split(ccdr.peripheral.GPIOA),
                                 dp.GPIOB.split(ccdr.peripheral.GPIOB),
                                 dp.GPIOC.split(ccdr.peripheral.GPIOC),
                                 dp.GPIOD.split(ccdr.peripheral.GPIOD),
                                 dp.GPIOE.split(ccdr.peripheral.GPIOE),
                                 dp.GPIOF.split(ccdr.peripheral.GPIOF),
                                 dp.GPIOG.split(ccdr.peripheral.GPIOG));

    let mut led_user = daisy::led::LedUser::new(pins.LED_USER);

    let pins = (pins.AK4556.PDN.into_push_pull_output(),
                pins.AK4556.MCLK_A.into_alternate_af6(),
                pins.AK4556.SCK_A.into_alternate_af6(),
                pins.AK4556.FS_A.into_alternate_af6(),
                pins.AK4556.SD_A.into_alternate_af6(),
                pins.AK4556.SD_B.into_alternate_af6());

    let sai1_prec = ccdr.peripheral.SAI1.kernel_clk_mux(hal::rcc::rec::Sai1ClkSel::PLL3_P);

    let audio_interface = audio::Interface::init(&ccdr.clocks,
                                                 sai1_prec,
                                                 pins,
                                                 ccdr.peripheral.DMA1).unwrap();
    // - audio callback -------------------------------------------------------

    let audio_interface = {
        let mut OSC_1: Oscillator = Oscillator::new(OscillatorMode::Square, 440.0, 48000.0);
        let mut OSC_2: Oscillator = Oscillator::new(OscillatorMode::Saw, 220.0, 48000.0);

        audio_interface.start(move |fs, block| {
            for frame in block {
                *frame = (OSC_1.tick_poly_blep(), OSC_2.tick_poly_blep());
            }
        })
    };

    let audio_interface = match audio_interface {
        Ok(audio_interface) => audio_interface,
        Err(e) => {
            loop {}
        }
    };

    cortex_m::interrupt::free(|cs| {
        AUDIO_INTERFACE.borrow(cs).replace(Some(audio_interface));
    });

    // - main loop ------------------------------------------------------------

    let one_second = ccdr.clocks.sys_ck().0;
    let mut counter = 0;

    loop {
        counter += 1;

        led_user.on();
        cortex_m::asm::delay(one_second);
        led_user.off();
        cortex_m::asm::delay(one_second);
    }
}

// - interrupts ---------------------------------------------------------------

/// interrupt handler for: dma1, stream1
#[interrupt]
fn DMA1_STR1() {
    cortex_m::interrupt::free(|cs| {
        if let Some(audio_interface) = AUDIO_INTERFACE.borrow(cs).borrow_mut().as_mut() {
            match audio_interface.handle_interrupt_dma1_str1() {
                Ok(()) => (),
                Err(e) => {
                }
            };
        }
    });
}