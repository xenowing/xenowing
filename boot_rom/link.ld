OUTPUT_ARCH("riscv")
ENTRY(_entry)

MEMORY
{
    boot_rom (rx) : ORIGIN = 0x00000000, LENGTH = 8k
    ram (!r) : ORIGIN = 0x10000000, LENGTH = 128k
}

__text_va = ORIGIN(boot_rom);
__data_va = ORIGIN(ram);
__stack_va = ORIGIN(ram) + LENGTH(ram);

SECTIONS
{
    .text __text_va :
    {
        PROVIDE (__tp = .);
        *(.entry*) /* Ensure the entry point is at the beginning of boot ROM */
        *(.text*)
    } >boot_rom = 0xff

    /* Pack .rodata in boot ROM as well as .text as it's read-only */
    .rodata :
    {
        *(.rodata*)
    } >boot_rom = 0xff

    __data_start = .;
    .data __data_va : AT(__data_start)
    {
        *(.data*)
    } >ram = 0xff

    .sdata :
    {
        PROVIDE (__gp = . + 0x800);
        *(.sdata*)
    } >ram = 0xff
    /* Since the .data and .sdata sections are in RAM, not boot ROM, using . is actually incorrect here,
       as that would refer to the position after these sections in RAM, not in boot ROM, so we must
       calculate the boot ROM position manually here. */
    __data_end = __data_start + SIZEOF(.data) + SIZEOF(.sdata);

    __bss_start = .;
    .sbss (NOLOAD):
    {
        *(.sbss*)
        *(.scommon*)
    } >ram = 0xff

    .bss (NOLOAD):
    {
        *(.bss*)
        *(COMMON)
    } >ram = 0xff
    __bss_end = .;
}
