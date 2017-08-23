# Defaults if not run from cargo
OUT_DIR ?= .
TARGET ?= Linux
NUM_JOBS ?= 1
DEBUG ?= false
OPT_LEVEL ?= 0

# Add parallel builds
MAKEFLAGS := --jobs=$(NUM_JOBS) $(MAKEFLAGS)

# Flags for both C and C++
FLAGS += \
	-fPIC \
	-O$(OPT_LEVEL) \
	$(NULL)

CFLAGS += \
	$(FLAGS) \
	$(NULL)

CXXFLAGS += \
	$(FLAGS) \
	-Ibreakpad \
	-std=c++11 \
	-DBPLOG_MINIMUM_SEVERITY=SEVERITY_ERROR \
	$(NULL)

# Get the operating system name for system dependent flags
#  - PLATFORM: resembles the output of uname -s
#  - LIBSTD:   the C++ standard library with C++11 support
#  - CXXFLAGS: platform dependent flags
ifneq (, $(findstring darwin, $(TARGET)))
	PLATFORM = Darwin
	LIBSTD = c++
	CXXFLAGS += -DHAVE_MACH_O_NLIST_H
else ifneq (, $(findstring freebsd, $(TARGET)))
	PLATFORM = FreeBSD
	LIBSTD = c++
else ifneq (, $(findstring linux, $(TARGET)))
	PLATFORM = Linux
	LIBSTD = stdc++
else
	PLATFORM = Windows
	LIBSTD = ""
endif

ifneq ($(DEBUG), false)
	FLAGS += -g
endif

LIBRARIES = \
	common \
	disasm \
	processor \
	symbols \
	$(NULL)

libcommon_OBJ = \
	cpp/c_string.o \
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
 	cpp/processor.o \
 	$(NULL)

libsymbols_Darwin_OBJ = \
	breakpad/common/dwarf_cfi_to_module.o \
	breakpad/common/dwarf_cu_to_module.o \
	breakpad/common/dwarf_line_to_module.o \
	breakpad/common/language.o \
	breakpad/common/md5.o \
	breakpad/common/module.o \
	breakpad/common/stabs_reader.o \
	breakpad/common/stabs_to_module.o \
	breakpad/common/dwarf/bytereader.o \
	breakpad/common/dwarf/dwarf2diehandler.o \
	breakpad/common/dwarf/dwarf2reader.o \
	breakpad/common/dwarf/elf_reader.o \
	breakpad/common/mac/arch_utilities.o \
	breakpad/common/mac/dump_syms.o \
	breakpad/common/mac/file_id.o \
	breakpad/common/mac/macho_id.o \
	breakpad/common/mac/macho_reader.o \
	breakpad/common/mac/macho_utilities.o \
	breakpad/common/mac/macho_walker.o \
	cpp/mac/symbols.o \
	$(NULL)

cargo: $(LIBRARIES)
	@echo cargo:rustc-link-lib=$(LIBSTD)
	@echo cargo:rustc-link-search=native=$(OUT_DIR)

$(LIBRARIES): %: $(OUT_DIR)/lib%.a
	@echo cargo:rustc-link-lib=static=$@

.SECONDEXPANSION:
$(LIBRARIES:%=$(OUT_DIR)/lib%.a): %.a: $$(addprefix $(OUT_DIR)/,$$($$(*F)_OBJ)) $$(addprefix $(OUT_DIR)/,$$($$(*F)_$$(PLATFORM)_OBJ))
	$(AR) $(ARFLAGS) $@ $(filter %.o,$^)

$(OUT_DIR)/%.o: %.c
	@mkdir -p $(@D)
	$(COMPILE.c) $(OUTPUT_OPTION) $<

$(OUT_DIR)/%.o: %.cc
	@mkdir -p $(@D)
	$(COMPILE.cc) $(OUTPUT_OPTION) $<

$(OUT_DIR)/%.o: %.cpp
	@mkdir -p $(@D)
	$(COMPILE.cpp) $(OUTPUT_OPTION) $<

clean:
	$(RM) -r $(OUT_DIR)

.PHONY: all $(LIBRARIES) clean
