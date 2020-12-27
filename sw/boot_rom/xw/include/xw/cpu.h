#ifndef XW_CPU_H
#define XW_CPU_H

#include "inttypes.h"

extern uint64_t xw_cycles();

void xw_sleep_cycles(uint64_t cycles);

#endif
