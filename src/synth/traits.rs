use super::SampleType;

trait MonoGenerator {
    fn tick(frames: &[SampleType]);
}

trait StereoGenerator {
    fn tick(frames: &[(SampleType, SampleType)]);
}