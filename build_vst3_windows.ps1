cargo build

New-Item -Force -ItemType Directory target/debug/soyboy-sp.vst3
New-Item -Force -ItemType Directory target/debug/soyboy-sp.vst3/Contents
New-Item -Force -ItemType Directory target/debug/soyboy-sp.vst3/Contents/Resources
New-Item -Force -ItemType Directory target/debug/soyboy-sp.vst3/Contents/x86_64-win


Copy-Item target/debug/soyboy_sp.dll target/debug/soyboy-sp.vst3/Contents/x86_64-win/soyboy-sp.vst3
