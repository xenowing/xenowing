RM=rm
RM_FLAGS=-rf

.PHONY: all
all: rtl sim test doc generated-rtl-old

.PHONY: clean
clean: rtl-clean sim-clean test-clean doc-clean generated-rtl-old-clean

RTL_DIR=rtl

.PHONY: rtl
rtl:
	cd $(RTL_DIR) && cargo build --release

.PHONY: rtl-clean
rtl-clean:
	cd $(RTL_DIR) && cargo clean

SIM_DIR=sim
MARV_DIR=$(SIM_DIR)/marv

.PHONY: sim
sim: marv

.PHONY: marv
marv:
	cd $(MARV_DIR) && cargo build --release

.PHONY: sim-clean
sim-clean: marv-clean

.PHONY: marv-clean
marv-clean:
	cd $(SIM_DIR)/marv && cargo clean

TEST_DIR=test

.PHONY: test
test:
	make -C $(TEST_DIR)/riscv-compliance

.PHONY: test-clean
test-clean:
	make clean -C $(TEST_DIR)/riscv-compliance

DOC_DIR=doc
MEM_TOPOLOGY=$(DOC_DIR)/mem_topology.pdf
MEM_TOPOLOGY_SRC=$(DOC_DIR)/mem_topology.dot

.PHONY: doc
doc: $(MEM_TOPOLOGY)

$(MEM_TOPOLOGY): $(MEM_TOPOLOGY_SRC)
	dot -Tpdf $(MEM_TOPOLOGY_SRC) -o $(MEM_TOPOLOGY)

.PHONY: doc-clean
doc-clean:
	$(RM) $(RM_FLAGS) $(MEM_TOPOLOGY)

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
