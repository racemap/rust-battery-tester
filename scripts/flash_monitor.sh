#!/bin/bash
esptool.py --chip auto --port /dev/ttyUSB1 --baud 115200 --before default_reset --after hard_reset write_flash -z --flash_mode dio --flash_freq 40m 0x10000 ./target/xtensa-esp32-espidf/release/rust-sport-tracker.bin

espflash monitor
