    .section .text.xw_cycles, "ax"
    .global _cycles
_cycles:
    /* Read loop to avoid overflow */
        rdcycleh a1
        rdcycle a0
        rdcycleh t0
    bne a1, t0, _cycles

    ret
