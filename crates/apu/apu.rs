use crate::register::Register;

#[derive(Debug)]
pub struct APU {
    pub registers: Register,
    // some internal states
}

impl APU {
    pub fn init() -> APU {
        APU {
            registers: Register::init(),
        }
    }
}
