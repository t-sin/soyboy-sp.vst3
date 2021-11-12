# SoyBoy SP - A GameBoy instrument

**(NICE PENGUIN LOGO HERE)**

I want to do some chiptunes on GNU/Linux.

## TODOs

- [ ] VST3 related
    - [x] plugin do nothing
    - [x] signal processing
    - [x] controllable parameters
    - [ ] Original GUI?
- [ ] plugin features
    - [ ] square wave osillator
        - [x] oscillate with fixed duty ratio
        - [x] duty ratio parameter
        - [ ] frequency sweep unit
    - [x] envelope generator
        - [x] generate ADSR envelope
        - [x] make output 4-bit value
    - [ ] noise oscillator
        - [ ] oscillate with fixed period
        - [ ] periods for random number generation
    - [ ] wave table oscillator
        - ???
    - [ ] pitch bend
    - [x] note velocity
    - [ ] note stutter (note delay)
    - [ ] DAC simulation

## References

- [Gameboy sound hardware - GbdevWiki](https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware)
- [vst3-sys examples](https://github.com/RustAudio/vst3-sys/tree/master/examples)
- [How to create VST3 plugins (Japanese)](https://vstcpp.wpblog.jp/?page_id=1316)

## Author

- Shinichi Tanaka (<shinichi.tanaka45@gmail.com>)

## License

???
