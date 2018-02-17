    .section .text

    .global _start
_start:
    li x1, 0x20000000
    li x2, 0b101
    sw x2, 0(x1)

    j _start
