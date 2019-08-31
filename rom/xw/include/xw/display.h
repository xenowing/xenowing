#ifndef XW_DISPLAY_H
#define XW_DISPLAY_H

#include "inttypes.h"
#include "bool.h"

#define XW_FRAMEBUFFER_WIDTH 320
#define XW_FRAMEBUFFER_HEIGHT 240

void xw_display_init();

uint16_t *xw_get_back_buffer();
void xw_swap_buffers(bool vsync);

#endif
