#ifndef XW_UART_H
#define XW_UART_H

#include "inttypes.h"

void xw_uart_write(uint8_t byte);

void xw_putc(const char c);
void xw_puts(const char *s);

#endif
