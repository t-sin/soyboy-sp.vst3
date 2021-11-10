#!/bin/bash

cargo build

mkdir -p target/debug/soyboy-sp.vst3
mkdir -p target/debug/soyboy-sp.vst3/Contents
mkdir -p target/debug/soyboy-sp.vst3/Contents/Resources
mkdir -p target/debug/soyboy-sp.vst3/Contents/x86_64-linux

cp target/debug/libsoyboy_sp.so target/debug/soyboy-sp.vst3/Contents/x86_64-linux/soyboy-sp.so
