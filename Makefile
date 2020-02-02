OBJ_DIR=obj_dir
TRACE_DIR=trace
DOC_DIR=doc

ifeq ($(OS),Windows_NT)
	DYNAMIC_LIB_PREFIX=
	DYNAMIC_LIB_EXT=.dll
else
	DYNAMIC_LIB_PREFIX=lib

	UNAME=$(shell uname)
	ifeq ($(UNAME),Darwin)
		DYNAMIC_LIB_EXT=.dylib
	else
		DYNAMIC_LIB_EXT=.so
	endif
endif

KAZE=rtl/kaze.py

GENERATED_RTL=rtl/_generated.sv
GENERATED_RTL_SRC=rtl/gen_modules.py rtl/uart.py rtl/display.py
GENERATED_RTL_GENERATOR=rtl/gen_modules.py

XENOWING_PREFIX=xenowing
XENOWING_VM_PREFIX=V$(XENOWING_PREFIX)
XENOWING_DRIVER=$(OBJ_DIR)/$(XENOWING_VM_PREFIX)
XENOWING_DRIVER_RTL=rtl/xenowing.sv $(GENERATED_RTL) rtl/uart/uart_clock_divider.sv rtl/uart/uart_transmitter.sv rtl/uart/uart_transmitter_interface.sv rtl/fifo.sv rtl/ram_interface.sv rtl/display_buffer.sv
XENOWING_DRIVER_SRC=sim/xenowing_driver.cpp

XENOWING_TEST_DIR=sim/xenowing-test
XENOWING_TEST_SRC=$(wildcard $(XENOWING_TEST_DIR)/**/*.rs)
XENOWING_TEST=$(XENOWING_TEST_DIR)/target/release/$(DYNAMIC_LIB_PREFIX)xenowing_test$(DYNAMIC_LIB_EXT)
XENOWING_TRACE=$(TRACE_DIR)/xenowing_test.vcd

DDR3_TEST_PREFIX=test
DDR3_TEST_VM_PREFIX=V$(DDR3_TEST_PREFIX)
DDR3_TEST_DRIVER=$(OBJ_DIR)/$(DDR3_TEST_VM_PREFIX)
DDR3_TEST_DRIVER_RTL=rtl/test/ddr3/test.sv rtl/test/ddr3/command_generator.sv rtl/test/ddr3/read_checker.sv
DDR3_TEST_DRIVER_SRC=sim/ddr3_test_driver.cpp

DDR3_TEST_DIR=sim/ddr3-test
DDR3_TEST_SRC=$(wildcard $(DDR3_TEST_DIR)/**/*.rs)
DDR3_TEST=$(DDR3_TEST_DIR)/target/release/$(DYNAMIC_LIB_PREFIX)ddr3_test$(DYNAMIC_LIB_EXT)
DDR3_TRACE=$(TRACE_DIR)/ddr3_test.vcd

VERILATOR=verilator
VERILATOR_FLAGS=-Wall -Wno-fatal -O3 --x-assign fast --noassert -CFLAGS "-O3 -std=c++11" --trace

RM=rm
RM_FLAGS=-rf

.PHONY: all dirs test docs clean

all: dirs $(GENERATED_RTL) $(XENOWING_DRIVER) $(XENOWING_TEST) $(DDR3_TEST_DRIVER) $(DDR3_TEST) docs

dirs: $(OBJ_DIR) $(TRACE_DIR)

$(OBJ_DIR):
	mkdir -p $(OBJ_DIR)

$(TRACE_DIR):
	mkdir -p $(TRACE_DIR)

$(GENERATED_RTL): $(GENERATED_RTL_SRC) $(KAZE)
	$(GENERATED_RTL_GENERATOR) $(GENERATED_RTL)

$(XENOWING_DRIVER): $(XENOWING_DRIVER_RTL) $(XENOWING_DRIVER_SRC)
	$(VERILATOR) $(VERILATOR_FLAGS) --top-module xenowing -cc $(XENOWING_DRIVER_RTL) --exe $(XENOWING_DRIVER_SRC)
	$(MAKE) -j -C $(OBJ_DIR) -f $(XENOWING_VM_PREFIX).mk

$(XENOWING_TEST): $(XENOWING_TEST_SRC) rom/rom.bin
	cd $(XENOWING_TEST_DIR) && cargo build --release

$(DDR3_TEST_DRIVER): $(DDR3_TEST_DRIVER_RTL) $(DDR3_TEST_DRIVER_SRC)
	$(VERILATOR) $(VERILATOR_FLAGS) -cc $(DDR3_TEST_DRIVER_RTL) --exe $(DDR3_TEST_DRIVER_SRC)
	$(MAKE) -j -C $(OBJ_DIR) -f $(DDR3_TEST_VM_PREFIX).mk

$(DDR3_TEST): $(DDR3_TEST_SRC)
	cd $(DDR3_TEST_DIR) && cargo build --release

docs: $(DOC_DIR)/mem_topology.pdf

$(DOC_DIR)/mem_topology.pdf: $(DOC_DIR)/mem_topology.dot
	dot -Tpdf $(DOC_DIR)/mem_topology.dot -o $(DOC_DIR)/mem_topology.pdf

test: dirs $(XENOWING_DRIVER) $(XENOWING_TEST) $(DDR3_TEST_DRIVER) $(DDR3_TEST)
	$(XENOWING_DRIVER) $(XENOWING_TEST) $(XENOWING_TRACE)
#	$(DDR3_TEST_DRIVER) $(DDR3_TEST) $(DDR3_TRACE)
#	$(DDR3_TEST_DRIVER) $(DDR3_TEST)

clean:
	$(RM) $(RM_FLAGS) $(OBJ_DIR)
	$(RM) $(RM_FLAGS) $(TRACE_DIR)
	$(RM) $(RM_FLAGS) $(GENERATED_RTL)
	cd $(XENOWING_TEST_DIR) && cargo clean
	cd $(DDR3_TEST_DIR) && cargo clean
