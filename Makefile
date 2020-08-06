RM=rm
RM_FLAGS=-rf

.PHONY: all
all: generated-rtl rtl sim

.PHONY: clean
clean: generated-rtl-clean rtl-clean sim-clean test-clean

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

SIM_DIR=sim
APPROX_RECIPROCAL_DIR=$(SIM_DIR)/approx-reciprocal
BUSTER_DIR=$(SIM_DIR)/buster
FIFO_DIR=$(SIM_DIR)/fifo
MARV_DIR=$(SIM_DIR)/marv
PEEK_BUFFER_DIR=$(SIM_DIR)/peek-buffer
READ_CACHE_DIR=$(SIM_DIR)/read-cache

.PHONY: sim
sim: approx-reciprocal buster fifo marv peek-buffer read-cache

.PHONY: approx-reciprocal
approx-reciprocal:
	cd $(APPROX_RECIPROCAL_DIR) && cargo build --release

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

.PHONY: read-cache
read-cache:
	cd $(READ_CACHE_DIR) && cargo build --release

.PHONY: sim-clean
sim-clean: approx-reciprocal-clean buster-clean fifo-clean marv-clean peek-buffer-clean read-cache-clean

.PHONY: approx-reciprocal-clean
approx-reciprocal-clean:
	cd $(APPROX_RECIPROCAL_DIR) && cargo clean

.PHONY: buster-clean
buster-clean:
	cd $(BUSTER_DIR) && cargo clean

.PHONY: fifo-clean
fifo-clean:
	cd $(FIFO_DIR) && cargo clean

.PHONY: marv-clean
marv-clean:
	cd $(MARV_DIR) && cargo clean

.PHONY: peek-buffer-clean
peek-buffer-clean:
	cd $(PEEK_BUFFER_DIR) && cargo clean

.PHONY: read-cache-clean
read-cache-clean:
	cd $(READ_CACHE_DIR) && cargo clean

TEST_DIR=test

.PHONY: test
test: approx-reciprocal-test buster-test compliance-test fifo-test peek-buffer-test read-cache-test rtl-test

.PHONY: approx-reciprocal-test
approx-reciprocal-test: approx-reciprocal
	cd $(APPROX_RECIPROCAL_DIR) && cargo test --release

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

.PHONY: read-cache-test
read-cache-test: read-cache
	cd $(READ_CACHE_DIR) && cargo test --release && cargo run --release -- 10 2000

.PHONY: rtl-test
rtl-test: rtl
	cd $(RTL_DIR) && cargo test --release

.PHONY: test-clean
test-clean: compliance-test-clean

.PHONY: compliance-test-clean
compliance-test-clean:
	make clean -C $(TEST_DIR)/riscv-compliance
