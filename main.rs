use gbi_apu::apu::APU;

fn main() {
    let apu = APU::init();
    println!("apu = {:?}", apu);
}
