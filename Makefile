OBJ_DIR=obj_dir

ALU_PREFIX=alu
ALU_VM_PREFIX=V$(ALU_PREFIX)

ALU_DRIVER=$(OBJ_DIR)/$(ALU_VM_PREFIX)

RM=rm
RM_FLAGS=-rf

.PHONY: all dirs test clean

all: $(ALU_DRIVER)

dirs: $(OBJ_DIR)

$(OBJ_DIR):
	mkdir -p $(OBJ_DIR)

$(ALU_DRIVER): dirs rtl/alu.sv sim/alu_driver.cpp
	verilator -Wall -O3 --x-assign fast --noassert -cc rtl/alu.sv --exe sim/alu_driver.cpp
	$(MAKE) -j -C $(OBJ_DIR) -f $(ALU_VM_PREFIX).mk

test: $(ALU_DRIVER)
	$(ALU_DRIVER)

clean:
	$(RM) $(RM_FLAGS) $(OBJ_DIR)
