OBJ_DIR=obj_dir
TRACE_DIR=trace
DOC_DIR=doc

XENOWING_PREFIX=xenowing
XENOWING_VM_PREFIX=V$(XENOWING_PREFIX)
XENOWING_DRIVER=$(OBJ_DIR)/$(XENOWING_VM_PREFIX)
XENOWING_DRIVER_RTL=rtl/xenowing.sv rtl/program_rom_interface.sv rtl/led_interface.sv rtl/mem_mapper.sv rtl/cpu/alu.sv rtl/cpu/cpu.sv
XENOWING_DRIVER_SRC=sim/xenowing_driver.cpp

XENOWING_TEST_DIR=sim/xenowing-test
XENOWING_TEST_SRC=$(wildcard $(XENOWING_TEST_DIR)/**/*.rs)
XENOWING_TEST=$(XENOWING_TEST_DIR)/target/release/xenowing_test.dll
XENOWING_TRACE=$(TRACE_DIR)/xenowing_test.vcd

ALU_PREFIX=alu
ALU_VM_PREFIX=V$(ALU_PREFIX)
ALU_DRIVER=$(OBJ_DIR)/$(ALU_VM_PREFIX)
ALU_DRIVER_RTL=rtl/cpu/alu.sv
ALU_DRIVER_SRC=sim/alu_driver.cpp

ALU_TEST_DIR=sim/alu-test
ALU_TEST_SRC=$(wildcard $(ALU_TEST_DIR)/**/*.rs)
ALU_TEST=$(ALU_TEST_DIR)/target/release/alu_test.dll

DDR3_TEST_PREFIX=test
DDR3_TEST_VM_PREFIX=V$(DDR3_TEST_PREFIX)
DDR3_TEST_DRIVER=$(OBJ_DIR)/$(DDR3_TEST_VM_PREFIX)
DDR3_TEST_DRIVER_RTL=rtl/test/ddr3/test.sv rtl/test/ddr3/command_generator.sv rtl/test/ddr3/read_checker.sv
DDR3_TEST_DRIVER_SRC=sim/ddr3_test_driver.cpp

DDR3_TEST_DIR=sim/ddr3-test
DDR3_TEST_SRC=$(wildcard $(DDR3_TEST_DIR)/**/*.rs)
DDR3_TEST=$(DDR3_TEST_DIR)/target/release/ddr3_test.dll
DDR3_TRACE=$(TRACE_DIR)/ddr3_test.vcd

VERILATOR=verilator
VERILATOR_FLAGS=-Wall -Wno-fatal -O3 --x-assign fast --noassert --trace

RM=rm
RM_FLAGS=-rf

.PHONY: all dirs test docs clean

all: dirs $(XENOWING_DRIVER) $(XENOWING_TEST) $(ALU_DRIVER) $(ALU_TEST) $(DDR3_TEST_DRIVER) $(DDR3_TEST) docs

dirs: $(OBJ_DIR) $(TRACE_DIR)

$(OBJ_DIR):
	mkdir -p $(OBJ_DIR)

$(TRACE_DIR):
	mkdir -p $(TRACE_DIR)

$(XENOWING_DRIVER): $(XENOWING_DRIVER_RTL) $(XENOWING_DRIVER_SRC)
	$(VERILATOR) $(VERILATOR_FLAGS) -cc $(XENOWING_DRIVER_RTL) --exe $(XENOWING_DRIVER_SRC)
	$(MAKE) -j -C $(OBJ_DIR) -f $(XENOWING_VM_PREFIX).mk

$(XENOWING_TEST): $(XENOWING_TEST_SRC)
	cd $(XENOWING_TEST_DIR) && cargo build --release

$(ALU_DRIVER): $(ALU_DRIVER_RTL) $(ALU_DRIVER_SRC)
	$(VERILATOR) $(VERILATOR_FLAGS) -cc $(ALU_DRIVER_RTL) --exe $(ALU_DRIVER_SRC)
	$(MAKE) -j -C $(OBJ_DIR) -f $(ALU_VM_PREFIX).mk

$(ALU_TEST): $(ALU_TEST_SRC)
	cd $(ALU_TEST_DIR) && cargo build --release

$(DDR3_TEST_DRIVER): $(DDR3_TEST_DRIVER_RTL) $(DDR3_TEST_DRIVER_SRC)
	$(VERILATOR) $(VERILATOR_FLAGS) -cc $(DDR3_TEST_DRIVER_RTL) --exe $(DDR3_TEST_DRIVER_SRC)
	$(MAKE) -j -C $(OBJ_DIR) -f $(DDR3_TEST_VM_PREFIX).mk

$(DDR3_TEST): $(DDR3_TEST_SRC)
	cd $(DDR3_TEST_DIR) && cargo build --release

docs: $(DOC_DIR)/mem_topology.pdf

$(DOC_DIR)/mem_topology.pdf: $(DOC_DIR)/mem_topology.dot
	dot -Tpdf $(DOC_DIR)/mem_topology.dot -o $(DOC_DIR)/mem_topology.pdf

test: dirs $(XENOWING_DRIVER) $(XENOWING_TEST) $(ALU_DRIVER) $(ALU_TEST) $(DDR3_TEST_DRIVER) $(DDR3_TEST)
	$(XENOWING_DRIVER) $(XENOWING_TEST) $(XENOWING_TRACE)
	#$(ALU_DRIVER) $(ALU_TEST)
	#$(DDR3_TEST_DRIVER) $(DDR3_TEST) $(DDR3_TRACE)
	#$(DDR3_TEST_DRIVER) $(DDR3_TEST)

clean:
	$(RM) $(RM_FLAGS) $(OBJ_DIR)
	$(RM) $(RM_FLAGS) $(TRACE_DIR)
	cd $(XENOWING_TEST_DIR) && cargo clean
	cd $(ALU_TEST_DIR) && cargo clean
	cd $(DDR3_TEST_DIR) && cargo clean
