#!/bin/bash

cargo build

mkdir -p target/debug/gbi.vst3
mkdir -p target/debug/gbi.vst3/Contents
mkdir -p target/debug/gbi.vst3/Contents/Resources
mkdir -p target/debug/gbi.vst3/Contents/x86_64-linux

cp target/debug/libgbi.so target/debug/gbi.vst3/Contents/x86_64-linux/gbi.so
