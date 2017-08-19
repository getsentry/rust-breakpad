#include "google_breakpad/processor/minidump.h"

#include "c_mapping.h"
#include "bindings.h"

using google_breakpad::Minidump;

typedef_extern_c(minidump_t, Minidump);

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
