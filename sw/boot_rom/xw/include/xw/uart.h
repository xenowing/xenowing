#ifndef XW_UART_H
#define XW_UART_H

#include "inttypes.h"

void xw_uart_write(uint8_t byte);
uint8_t xw_uart_read();

void xw_putc(const char c);
void xw_puts(const char *s);
void xw_puts_nn(const char *s);

char xw_getc();

#endif
