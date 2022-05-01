pub trait AudioProcessor<T> {
    fn process(&mut self, sample_rate: f64) -> T;
    fn set_freq(&mut self, freq: f64);
}
