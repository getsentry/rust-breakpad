# Environment defaults
TOP_DIR ?= ..
OUT_DIR ?= target
OPT_LEVEL ?= 0
OMIT_FP ?= true

# Flags for both C and C++
FLAGS += \
	-g \
	-fPIC \
	-O$(OPT_LEVEL) \
	$(if $OMIT_FP, -fomit-frame-pointer) \
	$(NULL)

CFLAGS += \
	$(FLAGS) \
	$(NULL)

CXXFLAGS += \
	$(FLAGS) \
	-I$(TOP_DIR) \
	-I$(TOP_DIR)/third_party/breakpad \
	$(NULL)

LIBRARIES = \
	client \
	$(NULL)

libclient_OBJ = \
	third_party/breakpad/client/minidump_file_writer.o \
	third_party/breakpad/client/mac/crash_generation/crash_generation_client.o \
	third_party/breakpad/client/mac/handler/dynamic_images.o \
	third_party/breakpad/client/mac/handler/exception_handler.o \
	third_party/breakpad/client/mac/handler/minidump_generator.o \
	third_party/breakpad/client/mac/handler/breakpad_nlist_64.o \
	third_party/breakpad/common/convert_UTF.o \
	third_party/breakpad/common/md5.o \
	third_party/breakpad/common/string_conversion.o \
	third_party/breakpad/common/mac/bootstrap_compat.o \
	third_party/breakpad/common/mac/file_id.o \
	third_party/breakpad/common/mac/macho_id.o \
	third_party/breakpad/common/mac/macho_utilities.o \
	third_party/breakpad/common/mac/macho_walker.o \
	third_party/breakpad/common/mac/string_utilities.o \
	third_party/breakpad/common/mac/MachIPC.o \
	$(NULL)

run: $(OUT_DIR)/crash_macos
	-$(shell ./$<)
	mv $(wildcard $(OUT_DIR)/*.dmp) $(OUT_DIR)/example.dmp
	cargo test --test example --color always -- --nocapture
	$(RM) $(OUT_DIR)/example.dmp

$(OUT_DIR)/crash_macos: $(OUT_DIR)/examples/crash_macos.o $(OUT_DIR)/libclient.a
	$(LINK.cpp) -o $@ $^ -framework CoreFoundation
	dsymutil $@

$(LIBRARIES): %: $(OUT_DIR)/lib%.a

.SECONDEXPANSION:
$(LIBRARIES:%=$(OUT_DIR)/lib%.a): %.a: $$(addprefix $(OUT_DIR)/,$$($$(*F)_OBJ)) $$(addprefix $(OUT_DIR)/,$$($$(*F)_$$(PLATFORM)_OBJ))
	$(AR) $(ARFLAGS) $@ $(filter %.o,$^)

$(OUT_DIR)/%.o: $(TOP_DIR)/%.c
	@mkdir -p $(@D)
	$(COMPILE.c) $(OUTPUT_OPTION) $<

$(OUT_DIR)/%.o: $(TOP_DIR)/%.cc
	@mkdir -p $(@D)
	$(COMPILE.cc) $(OUTPUT_OPTION) $<

$(OUT_DIR)/%.o: $(TOP_DIR)/%.cpp
	@mkdir -p $(@D)
	$(COMPILE.cpp) $(OUTPUT_OPTION) $<

$(OUT_DIR)/%.o: $(TOP_DIR)/%.mm
	@mkdir -p $(@D)
	$(COMPILE.cpp) $(OUTPUT_OPTION) $<

clean:
	$(RM) -r $(OUT_DIR)

.PHONY: run clean
