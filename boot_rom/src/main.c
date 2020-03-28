#include <xw/xw.h>

#define PROGRAM_RAM ((volatile uint8_t *)0x01000000)

typedef void (*program_ram_entry)();

int main()
{
    xw_puts("xw online");

    // TODO: Proper command
    xw_uart_write(0x01);
    // TODO: Proper filename
    const char *filename = "../../program/program.bin";
    int filename_index = 0;
    while (true)
    {
        char c = filename[filename_index++];
        xw_uart_write(c);
        if (!c)
            break;
    }
    uint32_t len = 0;
    len |= ((uint32_t)xw_uart_read() << 0);
    len |= ((uint32_t)xw_uart_read() << 8);
    len |= ((uint32_t)xw_uart_read() << 16);
    len |= ((uint32_t)xw_uart_read() << 24);

    for (uint32_t i = 0; i < len; i++)
    {
        uint8_t byte = xw_uart_read();
        xw_set_leds(byte);
        PROGRAM_RAM[i] = byte;
    }

    xw_puts("program RAM read successful");

    ((program_ram_entry)PROGRAM_RAM)();

    return 0;
}
