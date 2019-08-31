ifeq ($(OS),Windows_NT)
	TARGET_PREFIX=riscv-none-embed-
	EXE_EXT=.exe
else
	TARGET_PREFIX=riscv64-unknown-elf-
	EXE_EXT=
endif

ARCH=rv32i
ABI=ilp32
INCLUDE_DIRS=xw/include
CC=$(TARGET_PREFIX)gcc
CC_FLAGS=-march=$(ARCH) -mabi=$(ABI) -fno-builtin -nostdinc -nodefaultlibs -Wall -O3 $(foreach d,$(INCLUDE_DIRS),-I$d)
CC_SOURCES=$(wildcard src/*.c) $(wildcard xw/src/*.c)
CC_OBJS=$(CC_SOURCES:.c=.o)
AS=$(TARGET_PREFIX)as
AS_FLAGS=-march=$(ARCH)
AS_SOURCES=$(wildcard src/*.s)
AS_OBJS=$(AS_SOURCES:.s=.o)
OBJS=$(CC_OBJS) $(AS_OBJS)
LD_SCRIPT=link.ld
LD_FLAGS=-march=$(ARCH) -nostartfiles -T$(LD_SCRIPT)
ROM_ELF=rom.elf
ROM_BIN=rom.bin
ROM_MIF=rom.mif

MIFFED_DIR=../misc/miffed
MIFFED_SRC=$(wildcard $(MIFFED_DIR)/**/*.rs)
MIFFED=$(MIFFED_DIR)/target/debug/miffed$(EXE_EXT)

RM=rm
RM_FLAGS=-f

.PHONY: all
all: $(ROM_MIF)

.s.o:
	$(AS) $(AS_FLAGS) -o $@ $<

.c.o:
	$(CC) $(CC_FLAGS) -o $@ -c $<

$(ROM_ELF): $(OBJS) $(LD_SCRIPT)
	$(CC) $(LD_FLAGS) -o $@ $(OBJS)

$(ROM_BIN): $(ROM_ELF)
	$(TARGET_PREFIX)objcopy -O binary $(ROM_ELF) $@

$(ROM_MIF): $(ROM_BIN) $(MIFFED)
	$(MIFFED) $(ROM_BIN) $@

$(MIFFED): $(MIFFED_SRC)
	cd $(MIFFED_DIR) && cargo build

.PHONY: clean
clean:
	$(RM) $(RM_FLAGS) $(ROM_ELF) $(ROM_BIN) $(ROM_MIF) $(OBJS)
	cd $(MIFFED_DIR) && cargo clean
