# TODO: Consider using cargo-binutils or similar
TARGET_PREFIX=riscv64-unknown-elf-

RM=rm
RM_FLAGS=-rf

.PHONY: all
all: bootloader program rtl generated-rtl sim

.PHONY: clean
clean: bootloader-clean program-clean rtl-clean generated-rtl-clean sim-clean test-clean

# Boot ROM

BOOT_ROM_RUST_DIR=sw/bootloader
BOOT_ROM_TARGET_DIR=$(BOOT_ROM_RUST_DIR)/target
BOOT_ROM_BIN=$(BOOT_ROM_TARGET_DIR)/bootloader.bin
BOOT_ROM_ELF=$(BOOT_ROM_TARGET_DIR)/riscv32i-unknown-none-elf/release/bootloader

.PHONY: bootloader
bootloader: $(BOOT_ROM_BIN) bootloader-rust

.PHONY: bootloader-clean
bootloader-clean: bootloader-rust-clean
	$(RM) $(RM_FLAGS) $(BOOT_ROM_BIN)

$(BOOT_ROM_BIN): bootloader-rust
	$(TARGET_PREFIX)objcopy -O binary $(BOOT_ROM_ELF) $@

.PHONY: bootloader-rust
bootloader-rust:
	cd $(BOOT_ROM_RUST_DIR) && cargo build --release

.PHONY: bootloader-rust-clean
bootloader-rust-clean:
	cd $(BOOT_ROM_RUST_DIR) && cargo clean

# Program

PROGRAM_RUST_DIR=sw/program
PROGRAM_TARGET_DIR=$(PROGRAM_RUST_DIR)/target
PROGRAM_BIN=$(PROGRAM_TARGET_DIR)/program.bin
PROGRAM_ELF=$(PROGRAM_TARGET_DIR)/riscv32i-unknown-none-elf/release/program

.PHONY: program
program: $(PROGRAM_BIN) program-rust

.PHONY: program-clean
program-clean: program-rust-clean
	$(RM) $(RM_FLAGS) $(PROGRAM_BIN)

$(PROGRAM_BIN): program-rust
	$(TARGET_PREFIX)objcopy -O binary $(PROGRAM_ELF) $@

.PHONY: program-rust
program-rust:
	cd $(PROGRAM_RUST_DIR) && cargo build --release

.PHONY: program-rust-clean
program-rust-clean:
	cd $(PROGRAM_RUST_DIR) && cargo clean

# RTL

RTL_DIR=rtl

.PHONY: rtl
rtl: bootloader
	cd $(RTL_DIR) && cargo build --release

.PHONY: rtl-clean
rtl-clean:
	cd $(RTL_DIR) && cargo clean

# Generated RTL

GENERATED_RTL_NAME=_generated.v
GENERATED_RTL=$(RTL_DIR)/$(GENERATED_RTL_NAME)

.PHONY: generated-rtl
generated-rtl: $(GENERATED_RTL)

$(GENERATED_RTL): rtl
	cd $(RTL_DIR) && cargo run --release > $(GENERATED_RTL_NAME)

.PHONY: generated-rtl-clean
generated-rtl-clean:
	$(RM) $(RM_FLAGS) $(GENERATED_RTL)

# Sim

SIM_DIR=sim
APPROX_RECIPROCAL_DIR=$(SIM_DIR)/approx-reciprocal
BUSTER_DIR=$(SIM_DIR)/buster
BUSTER_MIG_UI_BRIDGE_DIR=$(SIM_DIR)/buster-mig-ui-bridge
FIFO_DIR=$(SIM_DIR)/fifo
FLOW_CONTROLLED_PIPE_DIR=$(SIM_DIR)/flow-controlled-pipe
MARV_DIR=$(SIM_DIR)/marv
PEEK_BUFFER_DIR=$(SIM_DIR)/peek-buffer
READ_CACHE_DIR=$(SIM_DIR)/read-cache

.PHONY: sim
sim: approx-reciprocal buster buster-mig-ui-bridge fifo flow-controlled-pipe marv peek-buffer read-cache

.PHONY: approx-reciprocal
approx-reciprocal:
	cd $(APPROX_RECIPROCAL_DIR) && cargo build --release

.PHONY: buster
buster:
	cd $(BUSTER_DIR) && cargo build --release

.PHONY: buster-mig-ui-bridge
buster-mig-ui-bridge:
	cd $(BUSTER_MIG_UI_BRIDGE_DIR) && cargo build --release

.PHONY: fifo
fifo:
	cd $(FIFO_DIR) && cargo build --release

.PHONY: flow-controlled-pipe
flow-controlled-pipe:
	cd $(FLOW_CONTROLLED_PIPE_DIR) && cargo build --release

.PHONY: marv
marv:
	cd $(MARV_DIR) && cargo build --release

.PHONY: peek-buffer
peek-buffer:
	cd $(PEEK_BUFFER_DIR) && cargo build --release

.PHONY: read-cache
read-cache:
	cd $(READ_CACHE_DIR) && cargo build --release

.PHONY: sim-clean
sim-clean: approx-reciprocal-clean buster-clean buster-mig-ui-bridge-clean fifo-clean flow-controlled-pipe-clean marv-clean peek-buffer-clean read-cache-clean

.PHONY: approx-reciprocal-clean
approx-reciprocal-clean:
	cd $(APPROX_RECIPROCAL_DIR) && cargo clean

.PHONY: buster-clean
buster-clean:
	cd $(BUSTER_DIR) && cargo clean

.PHONY: buster-mig-ui-bridge-clean
buster-mig-ui-bridge-clean:
	cd $(BUSTER_MIG_UI_BRIDGE_DIR) && cargo clean

.PHONY: fifo-clean
fifo-clean:
	cd $(FIFO_DIR) && cargo clean

.PHONY: flow-controlled-pipe-clean
flow-controlled-pipe-clean:
	cd $(FLOW_CONTROLLED_PIPE_DIR) && cargo clean

.PHONY: marv-clean
marv-clean:
	cd $(MARV_DIR) && cargo clean

.PHONY: peek-buffer-clean
peek-buffer-clean:
	cd $(PEEK_BUFFER_DIR) && cargo clean

.PHONY: read-cache-clean
read-cache-clean:
	cd $(READ_CACHE_DIR) && cargo clean

# Test

TEST_DIR=test

.PHONY: test
test: approx-reciprocal-test buster-test buster-mig-ui-bridge-test riscv-arch-test fifo-test flow-controlled-pipe-test peek-buffer-test read-cache-test rtl-test

.PHONY: approx-reciprocal-test
approx-reciprocal-test: approx-reciprocal
	cd $(APPROX_RECIPROCAL_DIR) && cargo test --release

.PHONY: buster-test
buster-test: buster
	cd $(BUSTER_DIR) && cargo test --release

.PHONY: buster-mig-ui-bridge-test
buster-mig-ui-bridge-test: buster-mig-ui-bridge
	cd $(BUSTER_MIG_UI_BRIDGE_DIR) && cargo test --release && cargo run --release -- 10 1000

.PHONY: riscv-arch-test
riscv-arch-test: marv
	make -C $(TEST_DIR)/riscv-arch-test

.PHONY: fifo-test
fifo-test: fifo
	cd $(FIFO_DIR) && cargo test --release && cargo run --release -- 10 10000000

.PHONY: flow-controlled-pipe-test
flow-controlled-pipe-test: flow-controlled-pipe
	cd $(FLOW_CONTROLLED_PIPE_DIR) && cargo test --release && cargo run --release -- 10 1000

.PHONY: peek-buffer-test
peek-buffer-test: peek-buffer
	cd $(PEEK_BUFFER_DIR) && cargo test --release && cargo run --release -- 10 1000

.PHONY: read-cache-test
read-cache-test: read-cache
	cd $(READ_CACHE_DIR) && cargo test --release && cargo run --release -- 10 2000

.PHONY: rtl-test
rtl-test: rtl
	cd $(RTL_DIR) && cargo test --release

.PHONY: test-clean
test-clean: riscv-arch-test-clean

.PHONY: riscv-arch-test-clean
riscv-arch-test-clean:
	make clean -C $(TEST_DIR)/riscv-arch-test
