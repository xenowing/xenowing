    .section .entry
    .align 1

    .global _entry
_entry:
    li x1, 0xdeadbeef
    li x2, 0x30000000
    sw x1, 0(x2)
    lw x3, 0(x2)

wait:
    j wait

    j stub

    .section .text.stub
    .align 1

    .global stub

stub:
    /* Copy data sections from ROM into RAM, as they need to be writable */
    lui t0, %hi(__data_start)
    addi t0, t0, %lo(__data_start)
    lui t1, %hi(__data_end)
    addi t1, t1, %lo(__data_end)
    lui t2, %hi(__data_va)
    addi t2, t2, %lo(__data_va)
    j copy_data_loop_end
copy_data_loop:
        lw t3, 0(t0)
        sw t3, 0(t2)
        addi t0, t0, 4
        addi t2, t2, 4
copy_data_loop_end:
    blt t0, t1, copy_data_loop

    /* Clear bss sections */
    lui t0, %hi(__bss_start)
    addi t0, t0, %lo(__bss_start)
    lui t1, %hi(__bss_end)
    addi t1, t1, %lo(__bss_end)
    j clear_bss_loop_end
clear_bss_loop:
        sw zero, 0(t0)
        addi t0, t0, 4
clear_bss_loop_end:
    blt t0, t1, clear_bss_loop

    /* Set up env registers */
    lui sp, %hi(__stack_va)
    addi sp, sp, %lo(__stack_va)
    lui tp, %hi(__tp)
    addi tp, tp, %lo(__tp)
    lui gp, %hi(__gp)
    addi gp, gp, %lo(__gp)

    /* Let's gooooo!! */
    lui t0, %hi(main)
    addi t0, t0, %lo(main)
    jalr zero, 0(t0)

    /* If main exits, just start over */
    j stub

    .section .text.xw_cycles
    .align 1

    .global xw_cycles

xw_cycles:
    /* Read loop to avoid overflow */
        rdcycleh a1
        rdcycle a0
        rdcycleh t0
    bne a1, t0, xw_cycles

    ret
