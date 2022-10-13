use std::ops::{Add, Mul};

use serde::{Deserialize, Serialize};

use crate::common::f64_utils;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct i4(u8);

impl i4 {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 2u8.pow(4) - 1;
    pub const LEVELS: u8 = 2u8.pow(4);
    pub const SIGNED_MIN: i8 = i4::LEVELS as i8 / 2 * -1;
    pub const SIGNED_MAX: i8 = i4::LEVELS as i8 / 2 - 1;
    pub const ZERO: i8 = i4::LEVELS as i8 / 4;
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
        let f = f64_utils::normalize(f);
        let other = f64_utils::normalize(other);
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
        let v = v - i4::SIGNED_MIN;
        i4(v as u8)
    }
}

impl From<i4> for i8 {
    fn from(v: i4) -> Self {
        let v = v.0 as i8;
        v + i4::SIGNED_MIN
    }
}

impl From<i4> for f64 {
    fn from(v: i4) -> Self {
        let i4v: i8 = v.into();

        if i4v == 0 {
            0.0
        } else if i4v < 0 {
            let levels = i4::SIGNED_MIN as f64 * -1.0;
            i4v as f64 / levels
        } else {
            let levels = i4::SIGNED_MAX as f64;
            i4v as f64 / levels
        }
    }
}

impl From<f64> for i4 {
    fn from(v: f64) -> Self {
        if v.is_subnormal() {
            i4::from(0i8)
        } else if v < 0.0 {
            let v = v.clamp(-1.0, 0.0) + 1.0;
            let v = v * i4::SIGNED_MIN.abs() as f64;
            i4(v as u8)
        } else {
            let v = v.clamp(0.0, 1.0);
            let v = (v * i4::SIGNED_MAX as f64) as u8 + i4::SIGNED_MIN.abs() as u8;
            i4(v)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::i4;

    #[test]
    fn test_i4_from_i8() {
        assert_eq!(i4::from(-8i8), i4(0));
        assert_eq!(i4::from(0i8), i4(8));
        assert_eq!(i4::from(7i8), i4(15));

        assert_eq!(i4::from(10i8), i4(15));
        assert_eq!(i4::from(-10i8), i4(0));

        assert_eq!(i4::from(i4::SIGNED_MIN), i4(0));
        assert_eq!(i4::from(i4::SIGNED_MAX), i4(15));
    }

    #[test]
    fn test_i8_from_i4() {
        assert_eq!(-8i8, i4(0).into());
        assert_eq!(0i8, i4(8).into());
        assert_eq!(7i8, i4(15).into());
    }

    #[test]
    fn test_i4_from_f64() {
        assert_eq!(i4(0), i4::from(-1.0));
        assert_eq!(i4(8), i4::from(0.0));
        assert_eq!(i4(15), i4::from(1.0));

        assert_eq!(i4(0), i4::from(-10.0));
        assert_eq!(i4(15), i4::from(10.0));
    }

    #[test]
    fn test_f64_from_i4() {
        assert_eq!(-1.0, i4(0).into());
        assert_eq!(0.0, i4(8).into());
        assert_eq!(1.0, i4(15).into());
    }

    #[test]
    fn test_mul_i4_and_f64() {
        assert_eq!(i4::from(0.0), i4::from(1.0) * 0.0);
        assert_eq!(i4::from(0.5), i4::from(1.0) * 0.5);
        assert_eq!(i4::from(1.0), i4::from(1.0) * 1.0);

        assert_eq!(i4::from(-0.5), i4::from(1.0) * -0.5);
        assert_eq!(i4::from(-1.0), i4::from(1.0) * -1.0);
    }
}
