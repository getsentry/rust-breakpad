#ifndef SENTRY_PROCESSOR_H
#define SENTRY_PROCESSOR_H

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
 * Snapshot of the state of a process during its crash. This object is obtained
 * by processing Minidumps using the process_* family of functions. To interact
 * with ProcessStates use the process_state_* family of functions.
 */
struct process_state_t;

/**
 * Source Line Resolver based on Breakpad's BasicSourceLineResolver. This class
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
 * Reads a minidump from the file system into memory and processes it. Returns
 * an owning pointer to a process_state_t struct that contains loaded code
 * modules and call stacks of all threads of the process during the crash.
 *
 * Processing the minidump can fail if the file is corrupted or does not exit.
 * The function will return NULL and an error code in result_out.
 *
 * Release memory of the process state with process_state_delete.
 */
process_state_t *process_minidump(const char *file_path, int *result_out);

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
 * Releases memory of a stack frame. Assumes ownership of the pointer.
 */
void stack_frame_delete(stack_frame_t *frame);

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
 * Returns the source code line at which the instruction was declared. Can
 * be empty before running the resolver or if debug symbols are missing.
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
 * Returns the path or file name that the code module was loaded from.
 *
 * The return value is an owning pointer. Release memory with string_delete.
 */
char *code_module_code_file(const code_module_t *module);

/**
 * An identifying string used to discriminate between multiple versions and
 * builds of the same code module.  This may contain a uuid, timestamp,
 * version number, or any combination of this or other information, in an
 * implementation-defined format.
 *
 * The return value is an owning pointer. Release memory with string_delete.
 */
char *code_module_code_identifier(const code_module_t *module);

/**
 * Returns the filename containing debugging information of this code
 * module.  If debugging information is stored in a file separate from the
 * code module itself (as is the case when .pdb or .dSYM files are used),
 * this will be different from code_file.  If debugging information is
 * stored in the code module itself (possibly prior to stripping), this
 * will be the same as code_file.
 *
 * The return value is an owning pointer. Release memory with string_delete.
 */
char *code_module_debug_file(const code_module_t *module);

/**
 * Returns a string identifying the specific version and build of the
 * associated debug file.  This may be the same as code_identifier when
 * the debug_file and code_file are identical or when the same identifier
 * is used to identify distinct debug and code files.
 *
 * It usually comprises the library's UUID and an age field. On Windows, the
 * age field is a generation counter, on all other platforms it is mostly
 * zero.
 *
 * The return value is an owning pointer. Release memory with string_delete.
 */
char *code_module_debug_identifier(const code_module_t *module);

/**
 * Creates a new source line resolver instance and returns an owning pointer
 * to it. Symbols are loaded from a Breakpad symbol file in the file system.
 * The file name is given as a weak pointer in the symbol_file parameter.
 *
 * Release memory of this resolver with the resolver_delete function.
 */
resolver_t *resolver_new(const char *symbol_file);

/**
 * Releases memory of a resolver object. Assumes ownership of the pointer.
 */
void resolver_delete(resolver_t *resolver);

/**
 * Returns whether the loaded symbol file was corrupt or can be used for
 * symbol resolution.
 */
bool resolver_is_corrupt(const resolver_t *resolver);

/**
 * Tries to locate the frame's instruction in the loaded code modules. Returns
 * an owning pointer to a new resolved stack frame instance. If no  symbols can
 * be found for the frame, a clone of the input is returned.
 *
 * This method expects a weak pointer to a frame. Release memory of this frame
 * with the stack_frame_delete function.
 */
stack_frame_t *resolver_resolve_frame(const resolver_t *resolver, const stack_frame_t *frame);

#ifdef __cplusplus
}
#endif

#endif
