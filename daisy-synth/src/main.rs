#![no_main]
#![no_std]
use core::convert::TryInto;
use core::{mem, slice};
use log::info;

use stm32h7xx_hal::{pac, prelude::*};
use stm32h7xx_hal::stm32;
use stm32h7xx_hal::timer::Timer;

use libdaisy::audio;
use libdaisy::gpio::*;
use libdaisy::hid;
use libdaisy::logger;
use libdaisy::prelude::*;
use libdaisy::system;

use libdsp::oscillators::{Oscillator, OscillatorMode};

// const LOOP_BUFFFER_SIZE: usize = 64 * 1024 * 1024 / 2 / mem::size_of::<u32>();
const LOOP_BUFFFER_SIZE: usize = libdaisy::sdram::Sdram::bytes() / 2 / mem::size_of::<f32>();

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
        switch1: hid::Switch<Daisy28<Input<PullUp>>>,
        osc: Oscillator,
        timer2: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);
        let buffer = [(0.0, 0.0); audio::BLOCK_SIZE_MAX];
        system.timer2.set_freq(1.ms());

        let loop_buffer_1: &mut [f32; LOOP_BUFFFER_SIZE] = unsafe {
            slice::from_raw_parts_mut(&mut system.sdram[0], LOOP_BUFFFER_SIZE)
                .try_into()
                .unwrap()
        };
        let loop_buffer_2: &mut [f32; LOOP_BUFFFER_SIZE] = unsafe {
            slice::from_raw_parts_mut(&mut system.sdram[LOOP_BUFFFER_SIZE], LOOP_BUFFFER_SIZE)
                .try_into()
                .unwrap()
        };

        let mut seed_led = hid::Led::new(system.gpio.led, false, 1000);
        seed_led.set_brightness(0.0);

        let daisy28 = system
            .gpio
            .daisy28
            .take()
            .expect("Failed to get pin daisy28!")
            .into_pull_up_input();

        let mut switch1 = hid::Switch::new(daisy28, hid::SwitchType::PullUp);
        switch1.set_double_thresh(Some(500));
        switch1.set_held_thresh(Some(1500));

        let mut sclk = system
            .gpio
            .daisy8
            .take()
            .expect("Failed to get sclk pin")
            .into_alternate_af5();

        let mut miso = system
            .gpio
            .daisy9
            .take()
            .expect("Failed to get miso pin")
            .into_alternate_af5();

        let mut mosi = system
            .gpio
            .daisy10
            .take()
            .expect("Failed to get mosi pin")
            .into_alternate_af5();
        
        pac::Peripherals::into();
        let mut spi = ctx.device.SPI1::spi();

        info!("Startup done!");

        // TODO: Check sample rate
        let osc = Oscillator::new(OscillatorMode::Saw, 440.0, 48000.0);

        init::LateResources {
            audio: system.audio,
            buffer,
            seed_led,
            switch1,
            osc,
            timer2: system.timer2,
        }
    }

    // Interrupt handler for audio
    #[task( binds = DMA1_STR1, resources = [audio, buffer, osc], priority = 8 )]
    fn audio_handler(ctx: audio_handler::Context) {
        let audio = ctx.resources.audio;
        let buffer = ctx.resources.buffer;
        let osc = ctx.resources.osc;

        if audio.get_stereo(buffer) {
            for (left, right) in buffer {
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

    #[task( binds = TIM2, resources = [timer2, seed_led, switch1, osc] )]
    fn interface_handler(mut ctx: interface_handler::Context) {
        ctx.resources.timer2.clear_irq();
        let switch1 = ctx.resources.switch1;
        let seed_led = ctx.resources.seed_led;
        switch1.update();
        seed_led.update();
    }
};
