use super::SampleType;

pub trait MonoGenerator {
    fn tick(&mut self) -> SampleType;
}

pub trait StereoGenerator {
    fn tick(&mut self) -> (SampleType, SampleType);
}