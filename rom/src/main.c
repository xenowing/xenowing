#include <xw/xw.h>

int main()
{
    xw_puts("xw online");

    xw_puts("get ready for some 0xfadebabe");
    const uint32_t phase_bits = 8;
    const uint32_t phase_min = 0;
    const uint32_t phase_max = 1 << phase_bits;
    const uint32_t phase_mask = 0xff;
    uint32_t phase = phase_min;
    uint32_t duty_cycle = phase_min;
    bool duty_cycle_rising = true;
    xw_set_leds(0);

    const uint32_t ticks_max = 500;
    uint32_t ticks = 0;

    while (true)
    {
        uint64_t t = xw_cycles();
        while (xw_cycles() - t < 1000)
            ;

        phase++;
        xw_set_leds((phase & phase_mask) < duty_cycle ? 1 : 0);

        ticks++;
        if (ticks >= ticks_max)
        {
            ticks = 0;
            if (duty_cycle_rising)
            {
                duty_cycle++;
                if (duty_cycle == phase_max)
                    duty_cycle_rising = false;
            }
            else
            {
                duty_cycle--;
                if (duty_cycle == 0)
                    duty_cycle_rising = true;
            }
        }
    }

    return 0;
}
