#include <xw/leds.h>

#define XW_LEDS ((volatile uint8_t *)0x20000000)

void xw_set_leds(uint8_t leds)
{
    *XW_LEDS = leds;
}
