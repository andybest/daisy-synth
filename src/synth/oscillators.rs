use super::{SampleType, PI, TWO_PI};
use super::traits::{MonoGenerator};

use micromath::F32Ext;

pub struct DSPSettings {
    sample_rate: SampleType
}

#[derive(PartialEq, Clone, Copy)]
pub enum OscillatorMode {
    Sine,
    Saw,
    Square,
    Triangle,
}

pub struct Oscillator {
    sample_rate: SampleType,
    mode: OscillatorMode,
    frequency: SampleType,
    phase: SampleType,
    phase_increment: SampleType,
    last_output: SampleType,
}

impl Oscillator {
    pub fn new(mode: OscillatorMode, frequency: SampleType, sample_rate: SampleType) -> Oscillator {
        let mut osc = Oscillator {
            sample_rate,
            mode,
            frequency,
            phase: 0.0,
            phase_increment: 0.0,
            last_output: 0.0,
        };
        osc.update_phase_increment();

        osc
    }

    fn update_phase_increment(&mut self) {
        self.phase_increment = self.frequency * 2.0 * PI / self.sample_rate;
    }

    pub fn set_frequency(&mut self, frequency: SampleType) {
        self.frequency = frequency;
        self.update_phase_increment();
    }

    pub fn set_sample_rate(&mut self, sample_rate: SampleType) {
        self.sample_rate = sample_rate;
        self.update_phase_increment();
    }

    fn poly_blep(&self, t: SampleType) -> SampleType {
        let dt = self.phase_increment / TWO_PI;
        if t < dt {
            let x = t / dt;
            x + x - x * x - 1.0
        } else if t > 1.0 - dt {
            let x = (t - 1.0) / dt;
            x * x + x + x + 1.0
        } else {
            0.0
        }
    }

    fn naive_waveform(&self, mode: OscillatorMode) -> SampleType {
        match mode {
            OscillatorMode::Sine => self.phase.sin(),
            OscillatorMode::Saw => (2.0 * self.phase / TWO_PI) - 1.0,
            OscillatorMode::Square => {
                if self.phase < PI {
                    1.0
                } else {
                    -1.0
                }
            }
            OscillatorMode::Triangle => {
                let x = -1.0 + (2.0 * self.phase / TWO_PI);
                2.0 * (x.abs() - 0.5)
            }
        }
    }

    pub fn tick_naive(&mut self) -> SampleType {
        let x = self.naive_waveform(self.mode);
        
        self.phase += self.phase_increment;
        while self.phase >= TWO_PI {
            self.phase -= TWO_PI
        }

        x
    }

    pub fn tick_poly_blep(&mut self) -> SampleType {
        let t = self.phase / TWO_PI;

        let samp: SampleType = match self.mode {
            OscillatorMode::Sine => self.naive_waveform(OscillatorMode::Sine),
            OscillatorMode::Saw => self.naive_waveform(OscillatorMode::Saw) - self.poly_blep(t),
            _ => {
                let mut x = self.naive_waveform(OscillatorMode::Square);
                x += self.poly_blep(t);
                x -= self.poly_blep((t + 0.5).fract());

                if self.mode == OscillatorMode::Triangle {
                    x = self.phase_increment * x + (1.0 - self.phase_increment) * self.last_output;
                    self.last_output = x;
                }

                x
            }
        };

        self.phase += self.phase_increment;
        while self.phase >= TWO_PI {
            self.phase -= TWO_PI
        }

        samp
    }
}

impl MonoGenerator for Oscillator {
    fn tick(&mut self) -> SampleType {
        self.tick_poly_blep()
    }
}