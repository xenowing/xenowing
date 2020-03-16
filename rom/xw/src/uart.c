#include <xw/bool.h>
#include <xw/uart.h>

#define XW_UART_BASE (0x02000000)

#define XW_UART_TX_STATUS ((volatile uint8_t *)(XW_UART_BASE + 0x00000000))
#define XW_UART_TX_WRITE ((volatile uint8_t *)(XW_UART_BASE + 0x00000010))

#define XW_UART_RX_STATUS ((volatile uint8_t *)(XW_UART_BASE + 0x00000020))
#define XW_UART_RX_READ ((volatile uint8_t *)(XW_UART_BASE + 0x00000030))

#define XW_UART_COMMAND_PUTC (0x00)

void xw_uart_write(uint8_t byte)
{
    while (!(*XW_UART_TX_STATUS & 1))
        ;

    *XW_UART_TX_WRITE = byte;
}

uint8_t xw_uart_read()
{
    while (!(*XW_UART_RX_STATUS & 1))
        ;

    return *XW_UART_RX_READ;
}

void xw_putc(const char c)
{
    xw_uart_write(XW_UART_COMMAND_PUTC);
    xw_uart_write((uint8_t)c);
}

void xw_puts(const char *s)
{
    xw_puts_nn(s);

    xw_putc('\n');
}

void xw_puts_nn(const char *s)
{
    while (true)
    {
        char c = *s++;
        if (!c)
            break;
        xw_putc(c);
    }
}

char xw_getc()
{
    return xw_uart_read();
}
