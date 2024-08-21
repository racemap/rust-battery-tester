#!/usr/bin/env bash

set -e

BUILD_MODE=""
case "$1" in
    ""|"release")
        BUILD_MODE="release"
        ;;
    "debug")
        BUILD_MODE="debug"
        ;;
    *)
        echo "Wrong argument. Only \"debug\"/\"release\" arguments are supported"
        exit 1;;
esac

export ESP_ARCH=xtensa-esp32-espidf

esptool.py --chip esp32 elf2image --flash_size 4MB ./target/${ESP_ARCH}/${BUILD_MODE}/rust-sport-tracker -o ./target/${ESP_ARCH}/${BUILD_MODE}/rust-sport-tracker.bin

