#[allow(non_camel_case_types)]
pub struct i4(i8);

impl i4 {
    pub const MIN_I8: i8 = -0x08;
    pub const MAX_I8: i8 = 0x07;
    pub const ZERO_I8: i8 = 0x00;
    pub const MIN: i4 = i4(i4::MIN_I8);
    pub const MAX: i4 = i4(i4::MAX_I8);
    pub const ZERO: i4 = i4(i4::ZERO_I8);

    pub fn new(v: i8) -> i4 {
        i4(v)
    }

    pub fn to_f64(&self) -> f64 {
        let v = self.0 as f64 / i4::MAX.0 as f64 * 2.0 - 1.0;
        v
    }
}

impl From<i8> for i4 {
    fn from(v: i8) -> i4 {
        if v < i4::MIN_I8 {
            i4::MIN
        } else if v > i4::MAX_I8 {
            i4::MAX
        } else {
            i4::new(v)
        }
    }
}

pub trait AudioProcessor<T> {
    fn process(&mut self, sample_rate: f64) -> T;
}
