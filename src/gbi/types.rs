#[allow(non_camel_case_types)]
pub struct u4(u8);

impl u4 {
    pub const MIN_U8: u8 = 0x0;
    pub const MAX_U8: u8 = 0xf;
    pub const ZERO_U8: u8 = 0x8;
    pub const MIN: u4 = u4(u4::MIN_U8);
    pub const MAX: u4 = u4(u4::MAX_U8);
    pub const ZERO: u4 = u4(u4::ZERO_U8);

    pub fn new(v: u8) -> u4 {
        u4(v)
    }

    pub fn to_f64(&self) -> f64 {
        let v = self.0 as f64 / u4::MAX.0 as f64 * 2.0 - 1.0;
        v
    }
}

impl From<u8> for u4 {
    fn from(v: u8) -> u4 {
        if v > u4::MAX_U8 {
            u4::MAX
        } else {
            u4::new(v)
        }
    }
}
