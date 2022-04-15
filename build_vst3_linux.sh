#!/bin/bash

if [ "$1" = "--release" ]; then
    echo "release build..."
    CARGO_BUILD_OPTS=--release
    BUILD_TARGET=release
else
    echo "debug build..."
    CARGO_BUILD_OPTS=
    BUILD_TARGET=debug
fi

cargo build $CARGO_BUILD_OPTS

mkdir -p target/$BUILD_TARGET/soyboy-sp.vst3
mkdir -p target/$BUILD_TARGET/soyboy-sp.vst3/Contents
mkdir -p target/$BUILD_TARGET/soyboy-sp.vst3/Contents/Resources
mkdir -p target/$BUILD_TARGET/soyboy-sp.vst3/Contents/x86_64-linux

cp target/$BUILD_TARGET/libsoyboy_sp.so target/$BUILD_TARGET/soyboy-sp.vst3/Contents/x86_64-linux/soyboy-sp.so
