#include <xw/xw.h>

#define FRAMEBUFFER_WIDTH 320
#define FRAMEBUFFER_HEIGHT 240
static uint16_t framebuffer[FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT] __attribute__((aligned(8)));

int main()
{
    xw_puts("xw online");

    xw_display_init();

    xw_puts("time to try a random framebuffer addr");
    xw_display_set_framebuffer_addr(framebuffer);

    xw_puts("get ready for some 0xfadebabe");
    const uint32_t phase_bits = 8;
    const uint32_t phase_min = 0;
    const uint32_t phase_max = 1 << phase_bits;
    const uint32_t phase_mask = 0xff;
    uint32_t phase = phase_min;
    uint32_t duty_cycle = phase_min;
    bool duty_cycle_rising = true;
    xw_set_leds(0);

    const uint32_t ticks_max = 2000;//500;
    uint32_t ticks = 0;

    uint32_t framebuffer_x = FRAMEBUFFER_WIDTH / 2;
    uint32_t framebuffer_y = FRAMEBUFFER_HEIGHT / 2;
    bool framebuffer_x_rising = true;
    bool framebuffer_y_rising = true;
    uint16_t color = (31 << 11) | (63 << 5) | 31;

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

            for (int y = 0; y < FRAMEBUFFER_HEIGHT; y++)
            {
                for (int x = 0; x < FRAMEBUFFER_WIDTH; x++)
                {
                    framebuffer[y * FRAMEBUFFER_WIDTH + x] = 0;
                }
            }
            /*if (framebuffer_x_rising)
            {
                framebuffer_x++;
                if (framebuffer_x == FRAMEBUFFER_WIDTH - 1)
                    framebuffer_x_rising = false;
            }
            else
            {
                framebuffer_x--;
                if (framebuffer_x == 0)
                    framebuffer_x_rising = true;
            }
            if (framebuffer_y_rising)
            {
                framebuffer_y++;
                if (framebuffer_y == FRAMEBUFFER_HEIGHT - 1)
                    framebuffer_y_rising = false;
            }
            else
            {
                framebuffer_y--;
                if (framebuffer_y == 0)
                    framebuffer_y_rising = true;
            }*/
            framebuffer[(framebuffer_y * FRAMEBUFFER_WIDTH) + framebuffer_x] = color;
        }
    }

    return 0;
}
