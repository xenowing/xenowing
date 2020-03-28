#include <xw/cpu.h>

void xw_sleep_cycles(uint64_t cycles)
{
    uint64_t t = xw_cycles();
    while (xw_cycles() - t < cycles)
        ;
}
