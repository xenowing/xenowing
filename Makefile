DOC_DIR=doc

KAZE=rtl/kaze.py

GENERATED_RTL=rtl/_generated.sv
GENERATED_RTL_SRC=rtl/gen_modules.py rtl/uart.py rtl/display.py
GENERATED_RTL_GENERATOR=rtl/gen_modules.py

RM=rm
RM_FLAGS=-rf

.PHONY: all
all: $(GENERATED_RTL) docs

$(GENERATED_RTL): $(GENERATED_RTL_SRC) $(KAZE)
	$(GENERATED_RTL_GENERATOR) $(GENERATED_RTL)

.PHONY: docs
docs: $(DOC_DIR)/mem_topology.pdf

$(DOC_DIR)/mem_topology.pdf: $(DOC_DIR)/mem_topology.dot
	dot -Tpdf $(DOC_DIR)/mem_topology.dot -o $(DOC_DIR)/mem_topology.pdf

.PHONY: clean
clean:
	$(RM) $(RM_FLAGS) $(GENERATED_RTL)
	$(RM) $(RM_FLAGS) $(DOC_DIR)/mem_topology.pdf
