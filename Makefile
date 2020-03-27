RM=rm
RM_FLAGS=-rf

.PHONY: all
all: generated-rtl generated-rtl-old rtl sim

.PHONY: clean
clean: generated-rtl-clean generated-rtl-old-clean rtl-clean sim-clean test-clean

RTL_DIR=rtl

.PHONY: rtl
rtl:
	cd $(RTL_DIR) && cargo build --release

.PHONY: rtl-clean
rtl-clean:
	cd $(RTL_DIR) && cargo clean

GENERATED_RTL_NAME=_generated.v
GENERATED_RTL=$(RTL_DIR)/$(GENERATED_RTL_NAME)

.PHONY: generated-rtl
generated-rtl: $(GENERATED_RTL)

$(GENERATED_RTL): rtl
	cd $(RTL_DIR) && cargo run --release > $(GENERATED_RTL_NAME)

.PHONY: generated-rtl-clean
generated-rtl-clean:
	$(RM) $(RM_FLAGS) $(GENERATED_RTL)

RTL_OLD_DIR=rtl-old
GENERATED_RTL_OLD=$(RTL_OLD_DIR)/_generated.sv
GENERATED_RTL_OLD_SRC=$(RTL_OLD_DIR)/gen_modules.py $(RTL_OLD_DIR)/display.py
GENERATED_RTL_OLD_GENERATOR=$(RTL_OLD_DIR)/gen_modules.py
KAZE_OLD=$(RTL_OLD_DIR)/kaze.py
RTL_OLD_PYCACHE=$(RTL_OLD_DIR)/__pycache__

.PHONY: generated-rtl-old
generated-rtl-old: $(GENERATED_RTL_OLD)

$(GENERATED_RTL_OLD): $(GENERATED_RTL_OLD_SRC) $(KAZE_OLD)
	$(GENERATED_RTL_OLD_GENERATOR) $(GENERATED_RTL_OLD)

.PHONY: generated-rtl-old-clean
generated-rtl-old-clean:
	$(RM) $(RM_FLAGS) $(GENERATED_RTL_OLD)
	$(RM) $(RM_FLAGS) $(RTL_OLD_PYCACHE)

SIM_DIR=sim
BUSTER_DIR=$(SIM_DIR)/buster
FIFO_DIR=$(SIM_DIR)/fifo
MARV_DIR=$(SIM_DIR)/marv
PEEK_BUFFER_DIR=$(SIM_DIR)/peek-buffer

.PHONY: sim
sim: buster fifo marv peek-buffer

.PHONY: buster
buster:
	cd $(BUSTER_DIR) && cargo build --release

.PHONY: fifo
fifo:
	cd $(FIFO_DIR) && cargo build --release

.PHONY: marv
marv:
	cd $(MARV_DIR) && cargo build --release

.PHONY: peek-buffer
peek-buffer:
	cd $(PEEK_BUFFER_DIR) && cargo build --release

.PHONY: sim-clean
sim-clean: fifo-clean marv-clean peek-buffer-clean

.PHONY: fifo-clean
fifo-clean:
	cd $(FIFO_DIR) && cargo clean

.PHONY: marv-clean
marv-clean:
	cd $(MARV_DIR) && cargo clean

.PHONY: peek-buffer-clean
peek-buffer-clean:
	cd $(PEEK_BUFFER_DIR) && cargo clean

TEST_DIR=test

.PHONY: test
test: buster-test compliance-test fifo-test peek-buffer-test rtl-test

.PHONY: buster-test
buster-test: buster
	cd $(BUSTER_DIR) && cargo test --release

.PHONY: compliance-test
compliance-test: marv
	make -C $(TEST_DIR)/riscv-compliance

.PHONY: fifo-test
fifo-test: fifo
	cd $(FIFO_DIR) && cargo test --release && cargo run --release -- 10 10000000

.PHONY: peek-buffer-test
peek-buffer-test: peek-buffer
	cd $(PEEK_BUFFER_DIR) && cargo test --release && cargo run --release -- 10 10000000

.PHONY: rtl-test
rtl-test: rtl
	cd $(RTL_DIR) && cargo test --release

.PHONY: test-clean
test-clean: compliance-test-clean

.PHONY: compliance-test-clean
compliance-test-clean:
	make clean -C $(TEST_DIR)/riscv-compliance
