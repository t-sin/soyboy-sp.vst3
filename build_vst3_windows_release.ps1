$Env:RUSTFLAGS = "-Ctarget-cpu=x86-64"
cargo build --release

New-Item -Force -ItemType Directory target/release/soyboy-sp.vst3
New-Item -Force -ItemType Directory target/release/soyboy-sp.vst3/Contents
New-Item -Force -ItemType Directory target/release/soyboy-sp.vst3/Contents/Resources
New-Item -Force -ItemType Directory target/release/soyboy-sp.vst3/Contents/x86_64-win


Copy-Item target/release/soyboy_sp.dll target/release/soyboy-sp.vst3/Contents/x86_64-win/soyboy-sp.vst3
