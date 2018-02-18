MIFFED_DIR=../misc/miffed
MIFFED_SRC=$(wildcard $(MIFFED_DIR)/**/*.rs)
MIFFED=$(MIFFED_DIR)/target/release/miffed.exe

.PHONY: all clean

all: rom.bin rom.mif

rom.o: rom.s
	riscv-none-embed-as -o rom.o rom.s

rom.elf: rom.o link.ld
	riscv-none-embed-gcc -o rom.elf -nostartfiles rom.o -Tlink.ld

rom.bin: rom.elf
	riscv-none-embed-objcopy -O binary rom.elf rom.bin

rom.mif: rom.bin $(MIFFED)
	$(MIFFED) rom.bin rom.mif

$(MIFFED): $(MIFFED_SRC)
	cd $(MIFFED_DIR) && cargo build --release

clean:
	cd $(MIFFED_DIR) && cargo clean
