OBJ_DIR=obj_dir

PREFIX=alu
VM_PREFIX=V$(PREFIX)

TARGET=$(OBJ_DIR)/$(VM_PREFIX)

RM=rm
RM_FLAGS=-rf

.PHONY: all dirs test clean

all: $(TARGET)

dirs: $(OBJ_DIR)

$(OBJ_DIR):
	mkdir -p $(OBJ_DIR)

$(TARGET): dirs rtl/alu.v driver/main.cpp
	verilator -Wall -O3 --x-assign fast --noassert -cc rtl/alu.v --exe driver/main.cpp
	$(MAKE) -j -C $(OBJ_DIR) -f $(VM_PREFIX).mk

test: $(TARGET)
	$(TARGET)

clean:
	$(RM) $(RM_FLAGS) $(OBJ_DIR)
