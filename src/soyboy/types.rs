use std::ops::{Add, Mul};

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub struct i4(f64);

impl i4 {
    pub fn zero() -> f64 {
        0.0
    }

    pub fn min() -> f64 {
        1.0 - i4::max()
    }

    pub fn max() -> f64 {
        2.0_f64.powf(4.0)
    }

    pub fn range() -> f64 {
        i4::min().abs() + i4::max().abs()
    }
}

impl From<f64> for i4 {
    fn from(v: f64) -> Self {
        let min = i4::min();
        let max = i4::max();

        let v = if v < min {
            min
        } else if v > max {
            max
        } else {
            v.trunc()
        };

        i4(v)
    }
}

impl From<i4> for f64 {
    fn from(src: i4) -> f64 {
        src.0
    }
}

impl Add<i4> for i4 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        i4(self.0 + other.0)
    }
}

impl Mul<i4> for i4 {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        i4(self.0 * other.0)
    }
}

impl Mul<f64> for i4 {
    type Output = Self;
    fn mul(self, other: f64) -> Self::Output {
        i4(self.0 * other)
    }
}

pub trait AudioProcessor<T> {
    fn process(&mut self, sample_rate: f64) -> T;
}
