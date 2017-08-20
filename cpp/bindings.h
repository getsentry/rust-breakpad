#ifndef SENTRY_STACKWALK_H
#define SENTRY_STACKWALK_H

#include <cstddef>
#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Structure holding all stack frames in a certain thread. Use the call_stack_*
 * family of functions to interact with a call stack.
 */
struct call_stack_t;

/**
 * Carries information about the code module loaded into the process. The field
 * debug_identifier contains the UUID of this module. Use the code_module_*
 * family of functions to interact with a code module.
 */
struct code_module_t;

/**
 * The primary interface to minidump files. Use the minidump_* family of
 * functions to interact with a minidump.
 *
 * To analyze this minidump, call minidump_process. This will return metadata
 * about the dump's threads containing all call stacks (see process_state_t).
 */
struct minidump_t;

/**
 * Result of processing a minidump. This structure is a snapshot, that can be
 * passed to a resolver for code location lookups. Use the process_state_*
 * family of functions to interact with a process state.
 */
struct process_state_t;

/**
 * Source Line Resolver based on Breakpad's FastSourceLineResolver. This class
 * handles Breakpad symbol files and resolves source code locations for stack
 * frames.
 *
 * To interact with the resolver, use the resolver_* family of functions.
 */
 struct resolver_t;

/**
 * Contains information from the stackdump, especially the frame's instruction
 * pointer. After being processed by a resolver, this struct also contains
 * source code locations and code offsets.
 */
struct stack_frame_t;

/**
 * Reads a minidump from the file system into memory. Returns an owning pointer
 * to the allocated minidump struct, if successful. If the minidump is invalid
 * or the file cannot be read, it returns a null pointer.
 *
 * Release memory of this minidump with the minidump_delete function.
 */
minidump_t *minidump_read(const char *file_path);

/**
 * Releases memory of a minidump struct. Assumes ownership of the pointer.
 */
void minidump_delete(minidump_t *dump);

/**
 * Prints debug information of the minidump to standard output.
 */
void minidump_print(minidump_t *dump);

/**
 * Analyzes the minidump and returns an owning pointer to a process_state_t
 * struct that contains loaded code modules and call stacks of all threads in
 * the minidump.
 *
 * Release memory of the process state with process_state_delete.
 */
process_state_t *minidump_process(minidump_t *dump);

/**
 * Releases memory of a process state struct. Assumes ownership of the pointer.
 */
void process_state_delete(process_state_t *state);

/**
 * Returns a weak pointer to the list of threads in the minidump. Each thread
 * is represented by the call stack structure. The number of threads is
 * returned in the size_out parameter.
 */
call_stack_t *const *process_state_threads(process_state_t *state,
                                           size_t *size_out);

/**
 * Returns the thread identifier of this callstack.
 */
uint32_t call_stack_thread_id(const call_stack_t *stack);

/**
 * Returns a weak pointer to the list of frames in a call stack. Each frame is
 * represented by the stack frame structure. The number of frames is returned
 * in the size_out parameter.
 */
stack_frame_t *const *call_stack_frames(const call_stack_t *stack,
                                        size_t *size_out);

/**
 * Returns the program counter location as an absolute virtual address.
 *
 * - For the innermost called frame in a stack, this will be an exact
 *   program counter or instruction pointer value.
 *
 * - For all other frames, this address is within the instruction that
 *   caused execution to branch to this frame's callee (although it may
 *   not point to the exact beginning of that instruction). This ensures
 *   that, when we look up the source code location for this frame, we
 *   get the source location of the call, not of the point at which
 *   control will resume when the call returns, which may be on the next
 *   line. (If the compiler knows the callee never returns, it may even
 *   place the call instruction at the very end of the caller's machine
 *   code, such that the "return address" (which will never be used)
 *   immediately after the call instruction is in an entirely different
 *   function, perhaps even from a different source file.)
 *
 * On some architectures, the return address as saved on the stack or in
 * a register is fine for looking up the point of the call. On others, it
 * requires adjustment. ReturnAddress returns the address as saved by the
 * machine.
 *
 * Use stack_frame_trust to obtain how trustworthy this instruction is.
 */
uint64_t stack_frame_instruction(const stack_frame_t *frame);

/**
 * Returns a weak pointer to the code module that hosts the instruction of the
 * stack framme. This function can return null for some frames.
 */
const code_module_t *stack_frame_module(const stack_frame_t *frame);

/**
 * Returns a weak pointer to the function name of the instruction. Can be empty
 * before running the resolver or if debug symbols are missing.
 */
const char *stack_frame_function_name(const stack_frame_t *frame);

/**
 * Returns a weak pointer to the source code file name in which the
 * instruction was declared. Can be empty before running the resolver or if
 * debug symbols are missing.
 */
const char *stack_frame_source_file_name(const stack_frame_t *frame);

/**
 * Returns the source code line at which the instruction was declared. Can be
 * empty before running the resolver or if debug symbols are missing.
 */
int stack_frame_source_line(const stack_frame_t *frame);

/**
 * Returns how well the instruction pointer derived during
 * stack walking is trusted. Since the stack walker can resort to
 * stack scanning, it can wind up with dubious frames.
 * In rough order of "trust metric".
 */
int stack_frame_trust(const stack_frame_t *frame);

/**
 * Returns an owning pointer to the name of the library or framework that
 * declares this code module.
 *
 * Release memory of this value with the string_delete function.
 */
char *code_module_debug_file(const code_module_t *module);

/**
 * Returns an owning pointer to the unique identifier of this code module.
 * Usually consists of the library's UUID and an age field. On Windows, the
 * age field is a generation counter, on all other platforms it is always
 * zero.
 *
 * Release memory of this value with the string_delete function.
 */
char *code_module_debug_identifier(const code_module_t *module);

/**
 * Creates a new source line resolver instance and returns an owning pointer
 * to it.
 *
 * Release memory of this resolver with the resolver_delete function.
 */
resolver_t *resolver_new();

/**
 * Releases memory of a resolver object. Assumes ownership of the pointer.
 */
void resolver_delete(resolver_t *resolver);

/**
 * Adds new symbols for the given code module from a Breakpad symbol file in
 * the file system. The file name is given as a weak pointer in the symbol_file
 * parameter. Returns whether the symbol map was built successfully.
 */
bool resolver_load_symbols(resolver_t *resolver,
                           const code_module_t *module,
                           const char *symbol_file);

/**
 * Tries to locate the frame's instruction in the loaded code modules and sets
 * its source code fields. If no symbosl can be found for the frame, it is not
 * touched.
 *
 * This method expects a weak pointer to a mutble frame.
 */
void resolver_fill_frame(resolver_t *resolver, stack_frame_t *frame);

#ifdef __cplusplus
}
#endif

#endif
