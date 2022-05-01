use std::ops::{Add, Mul};

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub struct i4(u8);

impl i4 {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 2u8.pow(4);
    pub const SIGNED_MIN: i8 = i4::MAX as i8 / 2 * -1;
    pub const SIGNED_MAX: i8 = i4::MAX as i8 / 2 - 1;
}

impl PartialEq for i4 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    fn ne(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for i4 {}

impl Add<i4> for i4 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        let v = self.0 + other.0;
        i4(v.clamp(i4::MIN, i4::MAX))
    }
}

impl Mul<f64> for i4 {
    type Output = i4;
    fn mul(self, other: f64) -> Self::Output {
        let f: f64 = self.into();
        println!("f = {}", f);
        i4::from(f * other)
    }
}

impl From<u8> for i4 {
    fn from(v: u8) -> Self {
        i4(v.clamp(i4::MIN, i4::MAX))
    }
}

impl From<i8> for i4 {
    fn from(v: i8) -> Self {
        let v = v.clamp(i4::SIGNED_MIN, i4::SIGNED_MAX);
        i4(v as u8)
    }
}

impl From<i4> for i8 {
    fn from(v: i4) -> Self {
        let v = v.0 as i8;
        v - i4::SIGNED_MIN
    }
}

impl From<i4> for f64 {
    fn from(v: i4) -> Self {
        let i4v: i8 = v.into();

        if i4v == 0 {
            0.0
        } else if i4v < 0 {
            let levels = i4::SIGNED_MIN.abs() as f64;
            i4v as f64 / levels
        } else {
            let levels = i4::SIGNED_MAX as f64;
            i4v as f64 / levels
        }
    }
}

impl From<f64> for i4 {
    fn from(v: f64) -> Self {
        let v = v.clamp(-1.0, 1.0) + 1.0;
        let v = (v * i4::MAX as f64) as u8;
        i4(v)
    }
}
