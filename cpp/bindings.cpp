#include "google_breakpad/processor/fast_source_line_resolver.h"
#include "google_breakpad/processor/call_stack.h"
#include "google_breakpad/processor/minidump.h"
#include "google_breakpad/processor/minidump_processor.h"
#include "google_breakpad/processor/process_state.h"
#include "google_breakpad/processor/stack_frame.h"

#include "c_mapping.h"
#include "c_string.h"
#include "bindings.h"

using google_breakpad::CallStack;
using google_breakpad::CodeModule;
using google_breakpad::FastSourceLineResolver;
using google_breakpad::Minidump;
using google_breakpad::MinidumpProcessor;
using google_breakpad::ProcessState;
using google_breakpad::StackFrame;

/**
 * Source Line Resolver based on Breakpad's FastSourceLineResolver. This class
 * handles Breakpad symbol files and resolves source code locations for stack
 * frames.
 *
 * This class does not provide any additional functionality, but exports some
 * internal functions so they can be called directly by the library client.
 * This allows us to separate minidump processing from symbol resolution.
 */
class Resolver : FastSourceLineResolver {
public:
    virtual bool LoadModule(const CodeModule *module, const string &map_file);
    virtual void FillSourceLineInfo(StackFrame *frame);
};

typedef_extern_c(call_stack_t, CallStack);
typedef_extern_c(code_module_t, CodeModule);
typedef_extern_c(minidump_t, Minidump);
typedef_extern_c(process_state_t, ProcessState);
typedef_extern_c(resolver_t, Resolver);
typedef_extern_c(stack_frame_t, StackFrame);

minidump_t *minidump_read(const char *file_path) {
    auto dump = new Minidump(file_path);

    if (!dump->Read()) {
        delete dump;
        return nullptr;
    }

    return minidump_t::cast(dump);
}

void minidump_delete(minidump_t *dump) {
    delete minidump_t::cast(dump);
}

void minidump_print(minidump_t *dump) {
    minidump_t::cast(dump)->Print();
}

process_state_t *minidump_process(minidump_t *dump) {
    MinidumpProcessor processor(nullptr, nullptr);
    ProcessState *state = new ProcessState();

    auto result = processor.Process(minidump_t::cast(dump), state);
    if (result != google_breakpad::PROCESS_OK) {
        return nullptr;
    }

    return process_state_t::cast(state);
}

void process_state_delete(process_state_t *state) {
    delete process_state_t::cast(state);
}

call_stack_t *const * process_state_threads(process_state_t *state, size_t *size_out) {
    auto *threads = process_state_t::cast(state)->threads();
    *size_out = threads->size();
    return reinterpret_cast<call_stack_t *const *>(threads->data());
}

uint32_t call_stack_thread_id(const call_stack_t *stack) {
    return call_stack_t::cast(stack)->tid();
}

stack_frame_t *const *call_stack_frames(const call_stack_t *stack, size_t *size_out) {
    auto *frames = call_stack_t::cast(stack)->frames();
    *size_out = frames->size();
    return reinterpret_cast<stack_frame_t *const *>(frames->data());
}

uint64_t stack_frame_instruction(const stack_frame_t *frame) {
    return stack_frame_t::cast(frame)->instruction;
}

const code_module_t *stack_frame_module(const stack_frame_t *frame) {
    return code_module_t::cast(stack_frame_t::cast(frame)->module);
}

const char *stack_frame_function_name(const stack_frame_t *frame) {
    return stack_frame_t::cast(frame)->function_name.c_str();
}

const char *stack_frame_source_file_name(const stack_frame_t *frame) {
    return stack_frame_t::cast(frame)->source_file_name.c_str();
}

int stack_frame_source_line(const stack_frame_t *frame) {
    return stack_frame_t::cast(frame)->source_line;
}

int stack_frame_trust(const stack_frame_t *frame) {
    return stack_frame_t::cast(frame)->trust;
}

char *code_module_debug_file(const code_module_t *module) {
    return string_from(code_module_t::cast(module)->debug_file());
}

char *code_module_debug_identifier(const code_module_t *module) {
    return string_from(code_module_t::cast(module)->debug_identifier());
}

resolver_t *resolver_new() {
    return resolver_t::cast(new Resolver());
}

void resolver_delete(resolver_t *resolver) {
    delete resolver_t::cast(resolver);
}

bool resolver_load_symbols(resolver_t *resolver, const code_module_t *module, const char *symbol_file) {
    return resolver_t::cast(resolver)->LoadModule(code_module_t::cast(module), symbol_file);
}

void resolver_fill_frame(resolver_t *resolver, stack_frame_t *frame) {
    resolver_t::cast(resolver)->FillSourceLineInfo(stack_frame_t::cast(frame));
}
