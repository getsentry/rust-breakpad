#ifndef SENTRY_C_STRING_H
#define SENTRY_C_STRING_H

#ifdef __cplusplus
#include <string>

extern "C" {

char *string_from(const std::string &str);
#endif

void string_delete(char *str);

#ifdef __cplusplus
}
#endif

#endif
