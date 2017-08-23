#ifndef SENTRY_DUMP_SYMS_H
#define SENTRY_DUMP_SYMS_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Creates symbols for the binary at the given location. On some systems,
 * debug symbols are extracted into a secondary file (e.g. dSYM on Darwin).
 * In this case, specify this file in secondary_path.
 *
 * If the file(s) cannot be opened or no debug information is included, this
 * function will return NULL. Otherwise, the breakpad symbols in ASCII format
 * are returned.
 *
 * This method returns an owning pointer to the string. Use string_delete to
 * release its memory.
 */
char *create_symbols(const char *file_path, const char *secondary_path);

#ifdef __cplusplus
}
#endif

#endif
