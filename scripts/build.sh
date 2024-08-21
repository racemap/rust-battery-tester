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
export ESP_ARCH=xtensa-esp32s3-espidf
BUILD_MODE=""
case "$1" in
    ""|"release")
        BUILD_MODE="release"
        cargo build --release
        esptool.py --chip ESP32S3 elf2image --output target/${ESP_ARCH}/${BUILD_MODE}/my-app.bin target/${ESP_ARCH}/${BUILD_MODE}/rust-sport-tracker
        rm target/${ESP_ARCH}/${BUILD_MODE}/rust-sport-tracker.bin
        mv  target/${ESP_ARCH}/${BUILD_MODE}/my-app.bin target/${ESP_ARCH}/${BUILD_MODE}/rust-sport-tracker.bin
        ;;
    "debug")
        BUILD_MODE="debug"
        cargo build
        esptool.py --chip ESP32S3 elf2image --output target/${ESP_ARCH}/${BUILD_MODE}/my-app.bin target/${ESP_ARCH}/${BUILD_MODE}/rust-sport-tracker
        rm target/${ESP_ARCH}/${BUILD_MODE}/rust-sport-tracker.bin
        mv  target/${ESP_ARCH}/${BUILD_MODE}/my-app.bin target/${ESP_ARCH}/${BUILD_MODE}/rust-sport-tracker.bin
        ;;
    *)
        echo "Wrong argument. Only \"debug\"/\"release\" arguments are supported"
        exit 1;;
esac



esptool.py --chip esp32s3 elf2image --flash_size 4MB ./target/${ESP_ARCH}/${BUILD_MODE}/rust-sport-tracker -o ./target/${ESP_ARCH}/${BUILD_MODE}/rust-sport-tracker.bin
