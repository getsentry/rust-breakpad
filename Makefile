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
	-I. \
	-Ithird_party/breakpad \
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
	CXXFLAGS += -DHAVE_A_OUT_H
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
	third_party/breakpad/third_party/libdisasm/ia32_implicit.o \
	third_party/breakpad/third_party/libdisasm/ia32_insn.o \
	third_party/breakpad/third_party/libdisasm/ia32_invariant.o \
	third_party/breakpad/third_party/libdisasm/ia32_modrm.o \
	third_party/breakpad/third_party/libdisasm/ia32_opcode_tables.o \
	third_party/breakpad/third_party/libdisasm/ia32_operand.o \
	third_party/breakpad/third_party/libdisasm/ia32_reg.o \
	third_party/breakpad/third_party/libdisasm/ia32_settings.o \
	third_party/breakpad/third_party/libdisasm/x86_disasm.o \
	third_party/breakpad/third_party/libdisasm/x86_format.o \
	third_party/breakpad/third_party/libdisasm/x86_imm.o \
	third_party/breakpad/third_party/libdisasm/x86_insn.o \
	third_party/breakpad/third_party/libdisasm/x86_misc.o \
	third_party/breakpad/third_party/libdisasm/x86_operand_list.o \
	$(NULL)

libprocessor_OBJ = \
	third_party/breakpad/processor/basic_code_modules.o \
	third_party/breakpad/processor/basic_source_line_resolver.o \
	third_party/breakpad/processor/call_stack.o \
	third_party/breakpad/processor/cfi_frame_info.o \
	third_party/breakpad/processor/disassembler_x86.o \
	third_party/breakpad/processor/dump_context.o \
	third_party/breakpad/processor/dump_object.o \
	third_party/breakpad/processor/logging.o \
	third_party/breakpad/processor/pathname_stripper.o \
	third_party/breakpad/processor/process_state.o \
	third_party/breakpad/processor/proc_maps_linux.o \
	third_party/breakpad/processor/simple_symbol_supplier.o \
	third_party/breakpad/processor/source_line_resolver_base.o \
	third_party/breakpad/processor/stack_frame_cpu.o \
	third_party/breakpad/processor/stack_frame_symbolizer.o \
	third_party/breakpad/processor/stackwalker.o \
	third_party/breakpad/processor/stackwalker_amd64.o \
	third_party/breakpad/processor/stackwalker_arm.o \
	third_party/breakpad/processor/stackwalker_arm64.o \
	third_party/breakpad/processor/stackwalker_mips.o \
	third_party/breakpad/processor/stackwalker_ppc.o \
	third_party/breakpad/processor/stackwalker_ppc64.o \
	third_party/breakpad/processor/stackwalker_sparc.o \
	third_party/breakpad/processor/stackwalker_x86.o \
	third_party/breakpad/processor/tokenize.o \
	third_party/breakpad/processor/exploitability.o \
	third_party/breakpad/processor/exploitability_linux.o \
	third_party/breakpad/processor/exploitability_win.o \
	third_party/breakpad/processor/minidump.o \
	third_party/breakpad/processor/minidump_processor.o \
	third_party/breakpad/processor/symbolic_constants_win.o \
	cpp/processor.o \
	$(NULL)

libsymbols_Darwin_OBJ = \
	third_party/breakpad/common/dwarf_cfi_to_module.o \
	third_party/breakpad/common/dwarf_cu_to_module.o \
	third_party/breakpad/common/dwarf_line_to_module.o \
	third_party/breakpad/common/language.o \
	third_party/breakpad/common/md5.o \
	third_party/breakpad/common/module.o \
	third_party/breakpad/common/stabs_reader.o \
	third_party/breakpad/common/stabs_to_module.o \
	third_party/breakpad/common/dwarf/bytereader.o \
	third_party/breakpad/common/dwarf/dwarf2diehandler.o \
	third_party/breakpad/common/dwarf/dwarf2reader.o \
	third_party/breakpad/common/dwarf/elf_reader.o \
	third_party/breakpad/common/mac/arch_utilities.o \
	third_party/breakpad/common/mac/dump_syms.o \
	third_party/breakpad/common/mac/file_id.o \
	third_party/breakpad/common/mac/macho_id.o \
	third_party/breakpad/common/mac/macho_reader.o \
	third_party/breakpad/common/mac/macho_utilities.o \
	third_party/breakpad/common/mac/macho_walker.o \
	cpp/mac/symbols.o \
	$(NULL)

libsymbols_Linux_OBJ = \
	third_party/breakpad/common/dwarf_cfi_to_module.o \
	third_party/breakpad/common/dwarf_cu_to_module.o \
	third_party/breakpad/common/dwarf_line_to_module.o \
	third_party/breakpad/common/language.o \
	third_party/breakpad/common/module.o \
	third_party/breakpad/common/stabs_reader.o \
	third_party/breakpad/common/stabs_to_module.o \
	third_party/breakpad/common/dwarf/bytereader.o \
	third_party/breakpad/common/dwarf/dwarf2diehandler.o \
	third_party/breakpad/common/dwarf/dwarf2reader.o \
	third_party/breakpad/common/dwarf/elf_reader.o \
	third_party/breakpad/common/linux/crc32.o \
	third_party/breakpad/common/linux/dump_symbols.o \
	third_party/breakpad/common/linux/elf_symbols_to_module.o \
	third_party/breakpad/common/linux/elfutils.o \
	third_party/breakpad/common/linux/file_id.o \
	third_party/breakpad/common/linux/linux_libc_support.o \
	third_party/breakpad/common/linux/memory_mapped_file.o \
	third_party/breakpad/common/linux/safe_readlink.o \
	cpp/linux/symbols.o \
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
