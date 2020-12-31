use gbi_apu::register::Register;
use gbi_apu::APU;

fn main() {
    let apu = APU {
        registers: Register::init(),
    };
    println!("apu = {:?}", apu);
}
