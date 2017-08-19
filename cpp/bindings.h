#ifndef SENTRY_STACKWALK_H
#define SENTRY_STACKWALK_H

#include <cstddef>
#include <cstdint>

#ifdef  __cplusplus
extern "C" {
#endif

struct minidump_t;

minidump_t* minidump_read(const char *file_path);
void minidump_delete(minidump_t *dump);
void minidump_print(minidump_t *dump);

#ifdef  __cplusplus
}
#endif

#endif
