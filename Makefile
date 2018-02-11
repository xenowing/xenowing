OBJ_DIR=obj_dir

ALU_PREFIX=alu
ALU_VM_PREFIX=V$(ALU_PREFIX)
ALU_DRIVER=$(OBJ_DIR)/$(ALU_VM_PREFIX)

ALU_TEST_DIR=sim/alu-test
ALU_TEST_SRC=$(wildcard $(ALU_TEST_DIR)/**/*.rs)
ALU_TEST=$(ALU_TEST_DIR)/target/release/alu_test

RM=rm
RM_FLAGS=-rf

.PHONY: all dirs test clean

all: $(ALU_TEST)

dirs: $(OBJ_DIR)

$(OBJ_DIR):
	mkdir -p $(OBJ_DIR)

$(ALU_DRIVER): dirs rtl/alu.sv sim/alu_driver.cpp
	verilator -Wall -O3 --x-assign fast --noassert -cc rtl/alu.sv --exe sim/alu_driver.cpp
	$(MAKE) -j -C $(OBJ_DIR) -f $(ALU_VM_PREFIX).mk

$(ALU_TEST): $(ALU_DRIVER) $(ALU_TEST_SRC)
	cd $(ALU_TEST_DIR) && cargo build --release

test: $(ALU_TEST)
	$(ALU_DRIVER) $(ALU_TEST)

clean:
	$(RM) $(RM_FLAGS) $(OBJ_DIR)
	cd $(ALU_TEST_DIR) && cargo clean
