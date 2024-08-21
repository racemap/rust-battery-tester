#!/bin/bash

# Gitpod and VsCode Codespaces tasks do not source the user environment
if [ "${USER}" == "gitpod" ]; then
    which idf.py >/dev/null || {
        source ~/export-esp.sh > /dev/null 2>&1
    }
elif [ "${CODESPACE_NAME}" != "" ]; then
    which idf.py >/dev/null || {
        source ~/export-esp.sh > /dev/null 2>&1
    }
fi

BUILD_MODE=""
case "$1" in
    ""|"release")
        BUILD_MODE="release"
        cargo build --release
        ;;
    "debug")
        BUILD_MODE="debug"
        cargo build
        ;;
    *)
        echo "Wrong argument. Only \"debug\"/\"release\" arguments are supported"
        exit 1;;
esac

export ESP_ARCH=xtensa-esp32-espidf

esptool.py --chip esp32 elf2image --flash_size 4MB ./target/${ESP_ARCH}/${BUILD_MODE}/rust-sport-tracker -o ./target/${ESP_ARCH}/${BUILD_MODE}/rust-sport-tracker.bin

esptool.py --chip auto --port /dev/ttyUSB0 --baud 115200 --before default_reset --after hard_reset write_flash -z --flash_mode dio --flash_freq 40m 0x10000 ./target/${ESP_ARCH}/${BUILD_MODE}/rust-sport-tracker.bin

espflash monitor