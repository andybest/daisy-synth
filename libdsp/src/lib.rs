#![no_std]
#[cfg(feature = "sample_f32")] type SampleType = f32;
#[cfg(not(feature = "sample_f32"))] type SampleType = f64;

const PI: SampleType = 3.141592653589793;
const TWO_PI: SampleType = PI * 2.0;

pub mod traits;
pub mod oscillators;