TARGET_PREFIX=riscv64-unknown-elf-
ifeq ($(OS),Windows_NT)
	EXE_EXT=.exe
else
	EXE_EXT=
endif

ARCH=rv32i
ABI=ilp32
INCLUDE_DIRS=xw/include
CC=$(TARGET_PREFIX)gcc
CC_FLAGS=-march=$(ARCH) -mabi=$(ABI) -mstrict-align -fno-builtin -nostdinc -nodefaultlibs -Wall -O2 $(foreach d,$(INCLUDE_DIRS),-I$d)
CC_SOURCES=$(wildcard src/*.c) $(wildcard xw/src/*.c)
CC_OBJS=$(CC_SOURCES:.c=.o)
AS=$(TARGET_PREFIX)as
AS_FLAGS=-march=$(ARCH)
AS_SOURCES=$(wildcard src/*.s)
AS_OBJS=$(AS_SOURCES:.s=.o)
OBJS=$(CC_OBJS) $(AS_OBJS)
LD_SCRIPT=link.ld
LD_FLAGS=-march=$(ARCH) -nostartfiles -T$(LD_SCRIPT)
PROGRAM_ELF=program.elf
PROGRAM_BIN=program.bin

RM=rm
RM_FLAGS=-f

.PHONY: all
all: $(PROGRAM_BIN)

.s.o:
	$(AS) $(AS_FLAGS) -o $@ $<

.c.o:
	$(CC) $(CC_FLAGS) -o $@ -c $<

$(PROGRAM_ELF): $(OBJS) $(LD_SCRIPT)
	$(CC) $(LD_FLAGS) -o $@ $(OBJS)

$(PROGRAM_BIN): $(PROGRAM_ELF)
	$(TARGET_PREFIX)objcopy -O binary $(PROGRAM_ELF) $@

.PHONY: clean
clean:
	$(RM) $(RM_FLAGS) $(PROGRAM_ELF) $(PROGRAM_BIN) $(OBJS)
