OUTPUT_ARCH("riscv")
ENTRY(_start)

PROVIDE(_max_program_size = 4k);

MEMORY
{
    BOOT_ROM    (rx) : ORIGIN = 0x00000000, LENGTH = 16k
    PROGRAM_RAM (!r) : ORIGIN = 0x10000000, LENGTH = _max_program_size
    RAM         (!r) : ORIGIN = 0x10000000 + _max_program_size, LENGTH = 16k - _max_program_size
}

REGION_ALIAS("REGION_TEXT", BOOT_ROM);
REGION_ALIAS("REGION_RODATA", BOOT_ROM);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);

PROVIDE(_stext = ORIGIN(REGION_TEXT));
PROVIDE(_stack_start = ORIGIN(REGION_STACK) + LENGTH(REGION_STACK));
PROVIDE(_stack_size = 2k);
PROVIDE(_heap_end = ORIGIN(REGION_HEAP) + LENGTH(REGION_HEAP) - _stack_size);

PROVIDE(_sprogram = ORIGIN(PROGRAM_RAM));
PROVIDE(_eprogram = ORIGIN(PROGRAM_RAM) + LENGTH(PROGRAM_RAM));

SECTIONS
{
    .text _stext :
    {
        KEEP(*(.init));
        *(.text .text.*);
    } > REGION_TEXT

    /* Pack .rodata in boot ROM as well as .text as it's read-only */
    .rodata : ALIGN(4)
    {
        *(.srodata .srodata.*);
        *(.rodata .rodata.*);
        . = ALIGN(4);
    } > REGION_RODATA

    .data : ALIGN(4)
    {
        _sidata = LOADADDR(.data);
        _sdata = .;
        /* TODO: What is this exactly? */
        /* Must be called __global_pointer$ for linker relaxations to work */
        PROVIDE(__global_pointer$ = . + 0x800);
        *(.sdata .sdata.* .sdata2 .sdata2.*);
        *(.data .data.*);
        . = ALIGN(4);
        _edata = .;
    } > REGION_DATA AT > REGION_RODATA

    .bss (NOLOAD) :
    {
        _sbss = .;
        *(.sbss .sbss.* .bss .bss.*);
        . = ALIGN(4);
        _ebss = .;
    } > REGION_BSS

    .heap (NOLOAD) :
    {
        _sheap = .;
        . = ABSOLUTE(_heap_end);
        _eheap = .;
    } > REGION_HEAP

    .stack (NOLOAD) :
    {
        _estack = .;
        . = ABSOLUTE(_stack_start);
        _sstack = .;
    } > REGION_STACK

    /* TODO: What is this exactly? */
    .eh_frame (INFO) : { KEEP(*(.eh_frame)) }
}
