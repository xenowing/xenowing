#ifndef XW_UART_H
#define XW_UART_H

#include "inttypes.h"

#define XW_UART_TX_STATUS ((volatile uint8_t *)0x21000000)
#define XW_UART_TX_WRITE ((volatile uint8_t *)0x21000004)

void xw_uart_write(uint8_t byte);

void xw_putc(const char c);
void xw_puts(const char *s);

#endif
