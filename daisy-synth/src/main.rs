#![no_main]
#![no_std]

use log::info;

use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::stm32;
use stm32h7xx_hal::timer::Timer;

use libdaisy::audio;
use libdaisy::gpio::*;
use libdaisy::hid;
use libdaisy::logger;

use libdsp::oscillators::{Oscillator, OscillatorMode};

mod gpio;
mod system;


#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        audio: audio::Audio,
        buffer: audio::AudioBuffer,
        seed_led: hid::Led<SeedLed>,
        osc: Oscillator,
        timer2: Timer<stm32::TIM2>,
        lcd: system::LCD
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);
        let buffer = [(0.0, 0.0); audio::BLOCK_SIZE_MAX];
        system.timer2.set_freq(1.ms());

        let mut seed_led = hid::Led::new(system.gpio.led, false, 1000);
        seed_led.set_brightness(0.0);

        info!("Startup done!");

        // TODO: Check sample rate
        let osc = Oscillator::new(OscillatorMode::Saw, 440.0, 48000.0);

        init::LateResources {
            audio: system.audio,
            buffer,
            seed_led,
            osc,
            timer2: system.timer2,
            lcd: system.ili9341
        }
    }

    // Interrupt handler for audio
    #[task( binds = DMA1_STR1, resources = [audio, buffer, osc], priority = 8 )]
    fn audio_handler(ctx: audio_handler::Context) {
        let audio = ctx.resources.audio;
        let buffer = ctx.resources.buffer;
        let osc = ctx.resources.osc;

        if audio.get_stereo(buffer) {
            for (left, _right) in buffer {
                let right = osc.tick_poly_blep();
                audio.push_stereo((*left, right)).unwrap();
            }
        } else {
            info!("Error reading data!");
        }
    }

    // Non-default idle ensures chip doesn't go to sleep which causes issues for
    // probe.rs currently
    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task( binds = TIM2, resources = [timer2, seed_led, osc] )]
    fn interface_handler(ctx: interface_handler::Context) {
        ctx.resources.timer2.clear_irq();
        let seed_led = ctx.resources.seed_led;
        seed_led.update();
    }
};
