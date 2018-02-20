    .section .text

    .global _start
_start:
    li x1, 0x20000000

again:
        rdcycleh x3
        rdcycle x2
        rdcycleh x4
    bne x3, x4, again
    srli x2, x2, 2

    sb x2, 7(x1)
    sb x2, 6(x1)
    sb x2, 5(x1)
    sb x2, 4(x1)
    sb x2, 3(x1)
    sb x2, 2(x1)
    sb x2, 1(x1)
    sb x2, 0(x1)

    j _start
