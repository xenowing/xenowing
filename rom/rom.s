    .section .text

    .global _start
_start:
    li x1, 0x20000000

    li x2, 7
    sb x2, 0(x1)

loop:
        lh x3, 2(x1)
        li x4, 0x1
        sub x3, x3, x4
        sh x3, 2(x1)
    j loop
