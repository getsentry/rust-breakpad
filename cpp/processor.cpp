#include <fstream>
#include <type_traits>
#include <vector>

#include "google_breakpad/processor/basic_source_line_resolver.h"
#include "google_breakpad/processor/call_stack.h"
#include "google_breakpad/processor/minidump_processor.h"
#include "google_breakpad/processor/process_state.h"
#include "google_breakpad/processor/stack_frame.h"
#include "processor/module_factory.h"

#include "cpp/c_mapping.h"
#include "cpp/c_string.h"
#include "cpp/mmap_symbol_supplier.h"
#include "cpp/processor.h"

using google_breakpad::BasicModuleFactory;
using google_breakpad::BasicSourceLineResolver;
using google_breakpad::CallStack;
using google_breakpad::CodeModule;
using google_breakpad::MinidumpProcessor;
using google_breakpad::ProcessState;
using google_breakpad::StackFrame;

namespace {

// Factory for modules to resolve stack frames.
BasicModuleFactory factory;

// Defines the private nested type BasicSourceLineResolver::Module
using ResolverModule =
    typename std::remove_pointer<decltype(factory.CreateModule(""))>::type;

StackFrame *clone_stack_frame(const StackFrame *frame) {
  if (frame == nullptr) {
    return nullptr;
  }

  auto *clone = new StackFrame();
  if (clone == nullptr) {
    return nullptr;
  }

  // We only need to clone instructions that are not later overwritten by the
  // resolver. Therefore, we assume this is a pristine unresolved frame.
  clone->instruction = frame->instruction;
  clone->module = frame->module;
  clone->trust = frame->trust;

  return clone;
}

}  // namespace

typedef_extern_c(call_stack_t, CallStack);
typedef_extern_c(code_module_t, CodeModule);
typedef_extern_c(process_state_t, ProcessState);
typedef_extern_c(resolver_t, ResolverModule);
typedef_extern_c(stack_frame_t, StackFrame);

process_state_t *process_minidump(const char *file_path,
                                  size_t symbol_count,
                                  symbol_entry_t *symbols,
                                  int *result_out) {
  if (file_path == nullptr) {
    *result_out = google_breakpad::PROCESS_ERROR_MINIDUMP_NOT_FOUND;
    return nullptr;
  }

  ProcessState *state = new ProcessState();
  if (state == nullptr) {
    *result_out = -1;  // Memory allocation issue
    return nullptr;
  }

  BasicSourceLineResolver resolver;
  MmapSymbolSupplier supplier(symbol_count, symbols);
  MinidumpProcessor processor(&supplier, &resolver);

  *result_out = processor.Process(file_path, state);
  if (*result_out != google_breakpad::PROCESS_OK) {
    delete state;
    return nullptr;
  }

  return process_state_t::cast(state);
}

void process_state_delete(process_state_t *state) {
  if (state != nullptr) {
    delete process_state_t::cast(state);
  }
}

call_stack_t *const *process_state_threads(process_state_t *state,
                                           size_t *size_out) {
  if (state == nullptr) {
    return nullptr;
  }

  auto *threads = process_state_t::cast(state)->threads();
  if (size_out != nullptr) {
    *size_out = threads->size();
  }

  return reinterpret_cast<call_stack_t *const *>(threads->data());
}

uint32_t call_stack_thread_id(const call_stack_t *stack) {
  return (stack == nullptr) ? 0 : call_stack_t::cast(stack)->tid();
}

stack_frame_t *const *call_stack_frames(const call_stack_t *stack,
                                        size_t *size_out) {
  if (stack == nullptr) {
    return nullptr;
  }

  auto *frames = call_stack_t::cast(stack)->frames();
  if (size_out != nullptr) {
    *size_out = frames->size();
  }

  return reinterpret_cast<stack_frame_t *const *>(frames->data());
}

void stack_frame_delete(stack_frame_t *frame) {
  if (frame != nullptr) {
    delete stack_frame_t::cast(frame);
  }
}

uint64_t stack_frame_instruction(const stack_frame_t *frame) {
  if (frame == nullptr) {
    return 0;
  }

  return stack_frame_t::cast(frame)->instruction;
}

const code_module_t *stack_frame_module(const stack_frame_t *frame) {
  if (frame == nullptr) {
    return nullptr;
  }

  return code_module_t::cast(stack_frame_t::cast(frame)->module);
}

const char *stack_frame_function_name(const stack_frame_t *frame) {
  if (frame == nullptr) {
    return nullptr;
  }

  return stack_frame_t::cast(frame)->function_name.c_str();
}

const char *stack_frame_source_file_name(const stack_frame_t *frame) {
  if (frame == nullptr) {
    return nullptr;
  }

  return stack_frame_t::cast(frame)->source_file_name.c_str();
}

int stack_frame_source_line(const stack_frame_t *frame) {
  if (frame == nullptr) {
    return 0;
  }

  return stack_frame_t::cast(frame)->source_line;
}

int stack_frame_trust(const stack_frame_t *frame) {
  if (frame == nullptr) {
    return StackFrame::FRAME_TRUST_NONE;
  }

  return stack_frame_t::cast(frame)->trust;
}

uint64_t code_module_base_address(const code_module_t *module) {
  return code_module_t::cast(module)->base_address();
}

uint64_t code_module_size(const code_module_t *module) {
  return code_module_t::cast(module)->size();
}

char *code_module_code_file(const code_module_t *module) {
  if (module == nullptr) {
    return nullptr;
  }

  return string_from(code_module_t::cast(module)->code_file());
}

char *code_module_code_identifier(const code_module_t *module) {
  if (module == nullptr) {
    return nullptr;
  }

  return string_from(code_module_t::cast(module)->code_identifier());
}

char *code_module_debug_file(const code_module_t *module) {
  if (module == nullptr) {
    return nullptr;
  }

  return string_from(code_module_t::cast(module)->debug_file());
}

char *code_module_debug_identifier(const code_module_t *module) {
  if (module == nullptr) {
    return nullptr;
  }

  return string_from(code_module_t::cast(module)->debug_identifier());
}

resolver_t *resolver_new(const char *symbol_file) {
  std::ifstream in(symbol_file);
  if (!in.good()) {
    return nullptr;
  }

  in.seekg(0, std::ios::end);
  std::streampos length = in.tellg();
  std::vector<char> buffer(length);

  in.seekg(0, std::ios::beg);
  in.read(&buffer[0], length);

  auto *module = factory.CreateModule("");
  if (module == nullptr) {
    return nullptr;
  }

  module->LoadMapFromMemory(&buffer[0], length);
  return resolver_t::cast(module);
}

void resolver_delete(resolver_t *resolver) {
  if (resolver != nullptr) {
    delete resolver_t::cast(resolver);
  }
}

bool resolver_is_corrupt(const resolver_t *resolver) {
  return resolver_t::cast(resolver)->IsCorrupt();
}

stack_frame_t *resolver_resolve_frame(const resolver_t *resolver,
                                      const stack_frame_t *frame) {
  if (resolver == nullptr || frame == nullptr) {
    return nullptr;
  }

  auto *clone = clone_stack_frame(stack_frame_t::cast(frame));
  if (clone == nullptr) {
    return nullptr;
  }

  resolver_t::cast(resolver)->LookupAddress(clone);
  return stack_frame_t::cast(clone);
}
