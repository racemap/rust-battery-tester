// Wokwi Custom UART Chip Example
//
// This chip implements a simple ROT13 letter substitution cipher.
// It receives a string over UART, and returns the same string with
// each alphabetic character replaced with its ROT13 substitution.
//
// For information and examples see:
// https://link.wokwi.com/custom-chips-alpha
//
// SPDX-License-Identifier: MIT
// Copyright (C) 2022 Uri Shaked / wokwi.com

#include "wokwi-api.h"
#include <stdio.h>
#include <stdlib.h>

typedef struct {
  uart_dev_t uart0;
} chip_state_t;

static void on_uart_rx_data(void *user_data, uint8_t byte);
static void on_uart_write_done(void *user_data);

void chip_init(void) {
  chip_state_t *chip = malloc(sizeof(chip_state_t));

  const uart_config_t uart_config = {
    .tx = pin_init("TX", INPUT_PULLUP),
    .rx = pin_init("RX", INPUT),
    .baud_rate = 115200,
    .rx_data = on_uart_rx_data,
    .write_done = on_uart_write_done,
    .user_data = chip,
  };
  chip->uart0 = uart_init(&uart_config);

  printf("UART Chip initialized!\n");
}

static uint8_t rot13(uint8_t value) {
  const uint8_t ROT = 13;
  if(value >= 'A' && value <='Z') {
    return (value + ROT) <= 'Z' ? value + ROT : value - ROT;
  }
  if(value >= 'a' && value <= 'z') {
    return (value + ROT) <= 'z' ? value + ROT : value - ROT;
  }
  return value;
}

 static void on_uart_rx_data(void *user_data, uint8_t byte) {
  chip_state_t *chip = (chip_state_t*)user_data;
  printf("Incoming UART data: %d\n", byte);
  uint8_t data_out = rot13(byte);
  uart_write(chip->uart0, &data_out, sizeof(data_out));
}

static void on_uart_write_done(void *user_data) {
  chip_state_t *chip = (chip_state_t*)user_data;
  printf("UART done\n");
}
