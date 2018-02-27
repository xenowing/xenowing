    .section .text

    .global _start
_start:
    li x1, 0x30000000
    li x2, 5
    sb x2, 0(x1)
    li x3, 0x20000000

loop:
        li x1, 0x30000000
        lb x2, 0(x1)
        addi x2, x2, 1
        sb x2, 0(x1)
        sb x2, 0(x3)

        li x4, 0x00080000
wait:
            addi x4, x4, -1
        bne x4, zero, wait

    j loop
