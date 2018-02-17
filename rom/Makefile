.PHONY: all

all: rom.bin

rom.o: rom.s
	riscv-none-embed-as -o rom.o rom.s

rom.elf: rom.o link.ld
	riscv-none-embed-gcc -o rom.elf -nostartfiles rom.o -Tlink.ld

rom.bin: rom.elf
	riscv-none-embed-objcopy -O binary rom.elf rom.bin
