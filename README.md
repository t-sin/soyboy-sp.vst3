# SoyBoy SP - A GameBoy instrument

I want to do some chiptunes on GNU/Linux.

![SoyBoy SP](edamame-logo.gif)

[*Edamame* is the Humboldt penguin living in Tobu Zoo](https://twitter.com/tobuzoo7/status/982488509725327361).

## Overview

*SoyBoy SP* is a VST3 instrument plugin to generate some GameBoy-like sounds. It has some basic features below:

- Three oscillator modes (square wave, noise, wavetable)
- A 32-samples 4bit wavetable oscillator
- A linear envelope generator
- A frequency sweeping to bend the pitch automatically
- A stutter, it's like a note delay

Additionally, *SoyBoy SP* has these features:

- Can be polyphonic; you can choose a number of voices (1 ~ 6)
- Pitch bending with MPE (MIDI Polyphonic Expression)

See [the website](https://t-sin.github.io/soyboy-sp.vst3/) ([Japanese ver. here](https://t-sin.github.io/soyboy-sp.vst3/index.ja.html)) to know how to use.
o
## Requirements

- GNU/Linux: install some libraries (see *Install*)
- Windows: none
- mac OS: currently not supported (because I don't have mac)

## Tested on

- `o`: works
- `x`: tested but does not works
- `-`: not tested

| /              | GNU/Linux | Winwods | mac OS |
| ---            | ---       | ---     | ---    |
| Bitwig 4.x     | o         | o       | -      |
| FL Studio 20.x | -         | o       | -      |
| Zrythm         | -         | o       | -      |
| Reaper         | -         | o       | -      |

## Install

### GNU/Linux

#### Build

There is no binary distribution. You can build *SoyBoy SP* by the instructions below:

1. Install [Rust](https://www.rust-lang.org) toolschain
2. Install depentent libraries
    - `apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`
3. Clone this repository
4. Run `./build_vst3_linux.sh --release`

#### Install

After doing that instruction, the directory `soyboy-sp.vst3` will be created in the path `target/release` in this repository. The directory is the VST3 plugin. On GNU/Linux, VST3 plugin is a directory.

### Windows

For the official binary distribution, see [the BOOTH page]().

To build it, install [Rust programming language](https://www.rust-lang.org) and run `build_vst3_windows_release.ps1`. A directory `soyboy-sp.vst3` is created in `target/release` and this is a VST3 plugin.

## References

- [Gameboy sound hardware - GbdevWiki](https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware)
- [vst3-sys examples](https://github.com/RustAudio/vst3-sys/tree/master/examples)
- [How to create VST3 plugins (Japanese)](https://vstcpp.wpblog.jp/?page_id=1316)

## Author

- t-sin (<shinichi.tanaka45@gmail.com>)

## License

This project is lisenced under the GPLv3 because of [Steinberg's licensing poricy](https://developer.steinberg.help/display/VST/VST+3+Licensing).
