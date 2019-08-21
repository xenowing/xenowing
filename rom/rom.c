#include <stdint.h>
#include <stdbool.h>

#define XW_LEDS ((volatile uint8_t *)0x20000000)

#define XW_UART_TX_STATUS ((volatile uint8_t *)0x21000000)
#define XW_UART_TX_WRITE ((volatile uint8_t *)0x21000004)

void xw_uart_write(uint8_t byte)
{
    while (!(*XW_UART_TX_STATUS & 1))
        ;

    *XW_UART_TX_WRITE = byte;
}

void xw_putc(const char c)
{
    xw_uart_write((uint8_t)c);
}

void xw_puts(const char *s)
{
    while (true)
    {
        char c = *s++;
        if (!c)
            break;
        xw_putc(c);
    }

    xw_putc('\r');
    xw_putc('\n');
}

extern uint64_t xw_cycles();

int main()
{
    /*const uint16_t start_state = 0xace1;
    uint16_t lfsr = start_state;

    while (true)
    {
        uint16_t bit = (lfsr >> 0) ^ (lfsr >> 2) ^ (lfsr >> 3) ^ (lfsr >> 5);
        lfsr = (lfsr >> 1) | (bit << 15);
        xw_uart_write((uint8_t)lfsr);
    }*/

    xw_puts("We're in main! Let's try a rather long string as it looks like we might be having some issues and it'd be nice to know it's not timing-related at least, or something something something darkside aaaah");

    xw_puts("Setting initial LED value");
    uint8_t leds = 5;

    xw_puts("Main loop time!");
    while (true)
    {
        *XW_LEDS = leds;
        leds -= 1;

        uint64_t t = xw_cycles();
        while (xw_cycles() - t < 20000000)
            ;
    }

    return 0;
}
