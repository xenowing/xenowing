    .section .init, "ax"
    .global _entry
_entry:
    /* Clear bss section */
    lui t0, %hi(_sbss)
    addi t0, t0, %lo(_sbss)
    lui t1, %hi(_ebss)
    addi t1, t1, %lo(_ebss)
    j _clear_bss_loop_end
_clear_bss_loop:
        sw zero, 0(t0)
        addi t0, t0, 4
_clear_bss_loop_end:
    blt t0, t1, _clear_bss_loop

    /* Copy data sections from ROM into RAM, as they need to be writable */
    lui t0, %hi(_sdata)
    addi t0, t0, %lo(_sdata)
    lui t1, %hi(_edata)
    addi t1, t1, %lo(_edata)
    lui t2, %hi(_sidata)
    addi t2, t2, %lo(_sidata)
    j _copy_data_loop_end
_copy_data_loop:
        lw t3, 0(t2)
        sw t3, 0(t0)
        addi t0, t0, 4
        addi t2, t2, 4
_copy_data_loop_end:
    blt t0, t1, _copy_data_loop

    /* Set up env registers */
    .option push
    .option norelax
    lui gp, %hi(__global_pointer$)
    addi gp, gp, %lo(__global_pointer$)
    .option pop

    lui sp, %hi(_stack_start)
    addi sp, sp, %lo(_stack_start)

    add s0, sp, zero

    /* Let's gooooo!! */
    lui t0, %hi(_rust_entry)
    addi t0, t0, %lo(_rust_entry)
    jalr zero, 0(t0)
