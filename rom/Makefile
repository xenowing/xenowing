ifeq ($(OS),Windows_NT)
	GCC_PREFIX=riscv-none-embed-
	EXE_EXT=.exe
else
	GCC_PREFIX=riscv64-unknown-elf-
	EXE_EXT=
endif

MIFFED_DIR=../misc/miffed
MIFFED_SRC=$(wildcard $(MIFFED_DIR)/**/*.rs)
MIFFED=$(MIFFED_DIR)/target/release/miffed$(EXE_EXT)

.PHONY: all clean

all: rom.bin rom.mif

rom.o: rom.s
	$(GCC_PREFIX)as -march=rv32i -o rom.o rom.s

rom.elf: rom.o link.ld
	$(GCC_PREFIX)gcc -march=rv32i -o rom.elf -nostartfiles rom.o -Tlink.ld

rom.bin: rom.elf
	$(GCC_PREFIX)objcopy -O binary rom.elf rom.bin

rom.mif: rom.bin $(MIFFED)
	$(MIFFED) rom.bin rom.mif

$(MIFFED): $(MIFFED_SRC)
	cd $(MIFFED_DIR) && cargo build --release

clean:
	cd $(MIFFED_DIR) && cargo clean
