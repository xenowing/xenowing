OBJ_DIR=obj_dir
TRACE_DIR=trace

ALU_PREFIX=alu
ALU_VM_PREFIX=V$(ALU_PREFIX)
ALU_DRIVER=$(OBJ_DIR)/$(ALU_VM_PREFIX)

ALU_TEST_DIR=sim/alu-test
ALU_TEST_SRC=$(wildcard $(ALU_TEST_DIR)/**/*.rs)
ALU_TEST=$(ALU_TEST_DIR)/target/release/alu_test

DDR3_TEST_PREFIX=ddr3_test
DDR3_TEST_VM_PREFIX=V$(DDR3_TEST_PREFIX)
DDR3_TEST_DRIVER=$(OBJ_DIR)/$(DDR3_TEST_VM_PREFIX)

DDR3_TEST_DIR=sim/ddr3-test
DDR3_TEST_SRC=$(wildcard $(DDR3_TEST_DIR)/**/*.rs)
DDR3_TEST=$(DDR3_TEST_DIR)/target/release/ddr3_test
DDR3_TRACE=$(TRACE_DIR)/ddr3_test.vcd

RM=rm
RM_FLAGS=-rf

.PHONY: all dirs test clean

all: $(ALU_TEST) $(DDR3_TEST)

dirs: $(OBJ_DIR)

$(OBJ_DIR):
	mkdir -p $(OBJ_DIR)

$(ALU_DRIVER): dirs rtl/cpu/alu.sv sim/alu_driver.cpp
	verilator -Wall -O3 --x-assign fast --noassert -cc rtl/cpu/alu.sv --exe sim/alu_driver.cpp
	$(MAKE) -j -C $(OBJ_DIR) -f $(ALU_VM_PREFIX).mk

$(ALU_TEST): $(ALU_DRIVER) $(ALU_TEST_SRC)
	cd $(ALU_TEST_DIR) && cargo build --release

$(DDR3_TEST_DRIVER): dirs rtl/test/ddr3_test.sv sim/ddr3_test_driver.cpp
	verilator -Wall -O3 --x-assign fast --noassert --trace -cc rtl/test/ddr3_test.sv rtl/test/led_clock_divider.sv --exe sim/ddr3_test_driver.cpp
	$(MAKE) -j -C $(OBJ_DIR) -f $(DDR3_TEST_VM_PREFIX).mk

$(DDR3_TEST): $(DDR3_TEST_DRIVER) $(DDR3_TEST_SRC)
	cd $(DDR3_TEST_DIR) && cargo build --release

test: $(ALU_TEST) $(DDR3_TEST)
	$(ALU_DRIVER) $(ALU_TEST)
	$(DDR3_TEST_DRIVER) $(DDR3_TEST) $(DDR3_TRACE)

clean:
	$(RM) $(RM_FLAGS) $(OBJ_DIR)
	$(RM) $(RM_FLAGS) $(TRACE_DIR)
	cd $(ALU_TEST_DIR) && cargo clean
	cd $(DDR3_TEST_DIR) && cargo clean
