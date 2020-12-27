#include <xw/inttypes.h>
#include <xw/bool.h>
#include <xw/cpu.h>
#include <xw/display.h>

#define DISPLAY_STATUS ((volatile uint32_t *)0x22000000)

#define DISPLAY_STATUS_VBLANK_BIT 0
#define DISPLAY_STATUS_VBLANK_MASK (1 << DISPLAY_STATUS_VBLANK_BIT)

#define DISPLAY_FRAMEBUFFER_ADDR ((volatile uint32_t *)0x22000004)

static uint16_t framebuffer0[XW_FRAMEBUFFER_WIDTH * XW_FRAMEBUFFER_HEIGHT] __attribute__((aligned(8)));
static uint16_t framebuffer1[XW_FRAMEBUFFER_WIDTH * XW_FRAMEBUFFER_HEIGHT] __attribute__((aligned(8)));
static uint16_t *back_buffer;
static uint16_t *front_buffer;

void xw_display_init()
{
    back_buffer = framebuffer0;
    front_buffer = framebuffer1;
    //*DISPLAY_FRAMEBUFFER_ADDR = (uint32_t)front_buffer;
}

uint16_t *xw_get_back_buffer()
{
    return back_buffer;
}

void xw_swap_buffers(bool vsync)
{
    uint16_t *temp = back_buffer;
    back_buffer = front_buffer;
    front_buffer = temp;
    /*while (vsync && !(*DISPLAY_STATUS & DISPLAY_STATUS_VBLANK_MASK))
        ;
    *DISPLAY_FRAMEBUFFER_ADDR = (uint32_t)front_buffer;*/
}
