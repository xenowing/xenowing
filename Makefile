RM=rm
RM_FLAGS=-rf

.PHONY: all
all: rtl sim generated-rtl-old

.PHONY: clean
clean: rtl-clean sim-clean generated-rtl-old-clean test-clean

RTL_DIR=rtl

.PHONY: rtl
rtl:
	cd $(RTL_DIR) && cargo build --release

.PHONY: rtl-clean
rtl-clean:
	cd $(RTL_DIR) && cargo clean

SIM_DIR=sim
FIFO_DIR=$(SIM_DIR)/fifo
MARV_DIR=$(SIM_DIR)/marv
PEEK_BUFFER_DIR=$(SIM_DIR)/peek-buffer

.PHONY: sim
sim: fifo marv peek-buffer

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

RTL_OLD_DIR=rtl-old
GENERATED_RTL_OLD=$(RTL_OLD_DIR)/_generated.sv
GENERATED_RTL_OLD_SRC=$(RTL_OLD_DIR)/gen_modules.py $(RTL_OLD_DIR)/uart.py $(RTL_OLD_DIR)/display.py
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

TEST_DIR=test

.PHONY: test
test: compliance-test fifo-test peek-buffer-test

.PHONY: compliance-test
compliance-test: marv
	make -C $(TEST_DIR)/riscv-compliance

.PHONY: fifo-test
fifo-test: fifo
	cd $(FIFO_DIR) && cargo test --release && cargo run --release -- 10 10000000

.PHONY: peek-buffer-test
peek-buffer-test: peek-buffer
	cd $(PEEK_BUFFER_DIR) && cargo test --release && cargo run --release -- 10 10000000

.PHONY: test-clean
test-clean: compliance-test-clean

.PHONY: compliance-test-clean
compliance-test-clean:
	make clean -C $(TEST_DIR)/riscv-compliance
