BUILD_DIR = target/debug/libraries

ifneq (, $(findstring darwin, $(TARGET)))
	LIBSTD = c++
else ifneq (, $(findstring freebsd, $(TARGET)))
	LIBSTD = c++
else
	LIBSTD = stdc++
endif

FLAGS = -fPIC

ifeq ($(DEBUG), false)
	FLAGS += -O3
else
	FLAGS += -O0 -g
endif

CFLAGS += \
	$(FLAGS) \
	$(NULL)

CXXFLAGS += \
	$(FLAGS) \
	-Ibreakpad \
	-std=c++11 \
	-DBPLOG_MINIMUM_SEVERITY=SEVERITY_ERROR \
	$(NULL)

LIBRARIES = \
	disasm \
	processor \
	$(NULL)

libdisasm_OBJ = \
	breakpad/third_party/libdisasm/ia32_implicit.o \
	breakpad/third_party/libdisasm/ia32_insn.o \
	breakpad/third_party/libdisasm/ia32_invariant.o \
	breakpad/third_party/libdisasm/ia32_modrm.o \
	breakpad/third_party/libdisasm/ia32_opcode_tables.o \
	breakpad/third_party/libdisasm/ia32_operand.o \
	breakpad/third_party/libdisasm/ia32_reg.o \
	breakpad/third_party/libdisasm/ia32_settings.o \
	breakpad/third_party/libdisasm/x86_disasm.o \
	breakpad/third_party/libdisasm/x86_format.o \
	breakpad/third_party/libdisasm/x86_imm.o \
	breakpad/third_party/libdisasm/x86_insn.o \
	breakpad/third_party/libdisasm/x86_misc.o \
	breakpad/third_party/libdisasm/x86_operand_list.o \
	$(NULL)

libprocessor_OBJ = \
 	breakpad/processor/basic_code_modules.o \
 	breakpad/processor/basic_source_line_resolver.o \
 	breakpad/processor/call_stack.o \
 	breakpad/processor/cfi_frame_info.o \
 	breakpad/processor/disassembler_x86.o \
 	breakpad/processor/dump_context.o \
 	breakpad/processor/dump_object.o \
 	breakpad/processor/logging.o \
 	breakpad/processor/pathname_stripper.o \
 	breakpad/processor/process_state.o \
 	breakpad/processor/proc_maps_linux.o \
 	breakpad/processor/simple_symbol_supplier.o \
 	breakpad/processor/source_line_resolver_base.o \
 	breakpad/processor/stack_frame_cpu.o \
 	breakpad/processor/stack_frame_symbolizer.o \
 	breakpad/processor/stackwalker.o \
 	breakpad/processor/stackwalker_amd64.o \
 	breakpad/processor/stackwalker_arm.o \
 	breakpad/processor/stackwalker_arm64.o \
 	breakpad/processor/stackwalker_mips.o \
 	breakpad/processor/stackwalker_ppc.o \
 	breakpad/processor/stackwalker_ppc64.o \
 	breakpad/processor/stackwalker_sparc.o \
 	breakpad/processor/stackwalker_x86.o \
 	breakpad/processor/tokenize.o \
 	breakpad/processor/exploitability.o \
 	breakpad/processor/exploitability_linux.o \
 	breakpad/processor/exploitability_win.o \
 	breakpad/processor/minidump.o \
 	breakpad/processor/minidump_processor.o \
 	breakpad/processor/symbolic_constants_win.o \
 	cpp/c_string.o \
 	cpp/bindings.o \
 	$(NULL)

cargo: $(LIBRARIES)
	@echo cargo:rustc-link-lib=$(LIBSTD)
	@echo cargo:rustc-link-search=native=$(BUILD_DIR)

$(LIBRARIES): %: $(BUILD_DIR)/lib%.a
	@echo cargo:rustc-link-lib=static=$@

.SECONDEXPANSION:
$(LIBRARIES:%=$(BUILD_DIR)/lib%.a): %.a: $$(addprefix $(BUILD_DIR)/,$$($$(*F)_OBJ))
	$(AR) $(ARFLAGS) $@ $(filter %.o,$^)

$(BUILD_DIR)/%.o: %.c
	@mkdir -p $(@D)
	$(COMPILE.c) $(OUTPUT_OPTION) $<

$(BUILD_DIR)/%.o: %.cc
	@mkdir -p $(@D)
	$(COMPILE.cc) $(OUTPUT_OPTION) $<

$(BUILD_DIR)/%.o: %.cpp
	@mkdir -p $(@D)
	$(COMPILE.cpp) $(OUTPUT_OPTION) $<

clean:
	$(RM) -r $(BUILD_DIR)

.PHONY: all $(LIBRARIES) clean
