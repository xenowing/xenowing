#include <xw/xw.h>

void put_u64(uint64_t value)
{
    xw_puts_nn("0x");
    char buf[] = "0000000000000000";
    for (int i = 0; i < 16; i++)
    {
        uint32_t digit = (uint32_t)(value >> i * 4) & 0xf;
        buf[15 - i] = digit < 10 ? '0' + digit : 'a' + (digit - 10);
    }
    xw_puts_nn(buf);
}

int main()
{
    xw_puts("hello from program RAM!!!");

    //xw_display_init();

    xw_puts("get ready for some 0xfadebabe");
    const uint32_t phase_bits = 8;
    const uint32_t phase_min = 0;
    const uint32_t phase_max = 1 << phase_bits;
    const uint32_t phase_mask = 0xff;
    uint32_t phase = phase_min;
    uint32_t duty_cycle = phase_min;
    bool duty_cycle_rising = true;
    xw_set_leds(0);

    uint32_t framebuffer_x = 0;
    uint32_t framebuffer_y = 0;
    bool framebuffer_x_rising = true;
    bool framebuffer_y_rising = true;
    uint16_t color = (31 << 11) | (63 << 5) | 31;
    uint32_t box_size = 100;

    bool first_box = true;

    while (true)
    {
        phase++;
        xw_set_leds((phase & phase_mask) < duty_cycle ? 1 : 0);

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

        uint64_t frame_start_time = 0;
        if (first_box)
            frame_start_time = xw_cycles();
        uint16_t *back_buffer = xw_get_back_buffer();
        uint64_t clear_start_time = 0;
        if (first_box)
            clear_start_time = xw_cycles();
        for (int y = 0; y < XW_FRAMEBUFFER_HEIGHT; y++)
        {
            for (int x = 0; x < XW_FRAMEBUFFER_WIDTH; x++)
            {
                back_buffer[y * XW_FRAMEBUFFER_WIDTH + x] = 0;
            }
        }
        uint64_t clear_end_time = 0;
        if (first_box)
            clear_end_time = xw_cycles();
        if (framebuffer_x_rising)
        {
            framebuffer_x++;
            if (framebuffer_x == XW_FRAMEBUFFER_WIDTH - box_size)
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
            if (framebuffer_y == XW_FRAMEBUFFER_HEIGHT - box_size)
                framebuffer_y_rising = false;
        }
        else
        {
            framebuffer_y--;
            if (framebuffer_y == 0)
                framebuffer_y_rising = true;
        }
        uint64_t box_start_time = 0;
        if (first_box)
            box_start_time = xw_cycles();
        for (int y = 0; y < box_size; y++)
        {
            for (int x = 0; x < box_size; x++)
            {
                back_buffer[(framebuffer_y + y) * XW_FRAMEBUFFER_WIDTH + framebuffer_x + x] = color;
            }
        }
        if (first_box)
        {
            uint64_t box_end_time = xw_cycles();
            uint64_t frame_end_time = xw_cycles();

            xw_puts_nn("Total frame cycles: ");
            put_u64(frame_end_time - frame_start_time);
            xw_puts("");

            xw_puts_nn("Clear cycles: ");
            put_u64(clear_end_time - clear_start_time);
            xw_puts("");

            xw_puts_nn("Box cycles: ");
            put_u64(box_end_time - box_start_time);
            xw_puts("");

            first_box = false;
        }

        xw_swap_buffers(true);
    }

    return 0;
}
