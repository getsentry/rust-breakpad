const bool generate_cfi = true;
const bool handle_inter_cu_refs = true;

#include <sstream>
#include <vector>

#include "common/linux/dump_symbols.h"

#include "cpp/c_string.h"
#include "cpp/symbols.h"

using google_breakpad::WriteSymbolFile;
using google_breakpad::DumpOptions;

/**
 * Copied and modified from breakpad/tools/linux/dump_syms/dump_syms.cc.
 */
char *create_symbols(const char *src_path, const char *unused) {
    if (src_path == nullptr) {
        return nullptr; // No file given
    }

    SymbolData symbol_data = generate_cfi ? ALL_SYMBOL_DATA : NO_CFI;
    DumpOptions options(symbol_data, handle_inter_cu_refs);

    std::vector<string> debug_dirs;
    std::ostringstream symbol_buffer;
    return WriteSymbolFile(src_path, debug_dirs, options, symbol_buffer)
        ? string_from(symbol_buffer.str())
        : nullptr; // Failed to write symbol file
}
