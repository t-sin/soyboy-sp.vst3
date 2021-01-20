# Design memo

## Block diagram

![](module-archtecture.png)

## Controlable parameters

### General parameters

- oscillator select
  - 0: square wave
  - 1: noise
  - 2: wavetable

### Square wave channel parameters

- frequency sweep on/off
  - 0: off
  - 1: on

- frequency sweep negate
  - 0: false
  - 1: true

- frequency sweep intensity
  - 0 ~ 8

- frequency sweep period
  - 0 ~ 8

- sweep LFO (triangle wave) on/off
  - 0: off
  - 1: on

### Noise channel parameters

### Wavetable channel parameters
