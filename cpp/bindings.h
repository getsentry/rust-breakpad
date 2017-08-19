#ifndef SENTRY_STACKWALK_H
#define SENTRY_STACKWALK_H

#include <cstddef>
#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

struct call_stack_t;
struct code_module_t;
struct minidump_t;
struct process_state_t;
struct resolver_t;
struct stack_frame_t;

minidump_t *minidump_read(const char *file_path);
void minidump_delete(minidump_t *dump);
void minidump_print(minidump_t *dump);
process_state_t *minidump_process(minidump_t *dump);

void process_state_delete(process_state_t *state);
call_stack_t *const *process_state_threads(process_state_t *state,
                                           size_t *size_out);

uint32_t call_stack_thread_id(const call_stack_t *stack);
stack_frame_t *const *call_stack_frames(const call_stack_t *stack,
                                        size_t *size_out);

uint64_t stack_frame_instruction(const stack_frame_t *frame);
const code_module_t *stack_frame_module(const stack_frame_t *frame);
const char *stack_frame_function_name(const stack_frame_t *frame);
const char *stack_frame_source_file_name(const stack_frame_t *frame);
int stack_frame_source_line(const stack_frame_t *frame);
int stack_frame_trust(const stack_frame_t *frame);

char *code_module_debug_file(const code_module_t *module);
char *code_module_debug_identifier(const code_module_t *module);

resolver_t *resolver_new();
void resolver_delete(resolver_t *resolver);
bool resolver_load_symbols(resolver_t *resolver,
                           const code_module_t *module,
                           const char *symbol_file);
void resolver_fill_frame(resolver_t *resolver, stack_frame_t *frame);

#ifdef __cplusplus
}
#endif

#endif
