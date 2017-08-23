#include <sstream>

#include "common/mac/dump_syms.h"
#include "common/mac/arch_utilities.h"
#include "common/mac/macho_utilities.h"
#include "common/scoped_ptr.h"

#include "../cpp/c_string.h"
#include "../cpp/symbols.h"

using google_breakpad::DumpSymbols;
using google_breakpad::Module;
using google_breakpad::scoped_ptr;

const bool generate_cfi = true;
const bool handle_inter_cu_refs = true;

/**
 * Copied from breakpad/tools/mac/dump_syms/dump_syms_tool.cc.
 */
static bool StackFrameEntryComparator(const Module::StackFrameEntry* a,
                                      const Module::StackFrameEntry* b) {
    return a->address < b->address;
}

/**
 * Copy the CFI data from |from_module| into |to_module|, for any non-
 * overlapping ranges.
 *
 * Copied from breakpad/tools/mac/dump_syms/dump_syms_tool.cc.
 */
static void CopyCFIDataBetweenModules(Module* to_module, const Module* from_module) {
    typedef std::vector<Module::StackFrameEntry*>::const_iterator Iterator;

    // Get the CFI data from both the source and destination modules and ensure
    // it is sorted by start address.
    std::vector<Module::StackFrameEntry*> from_data;
    from_module->GetStackFrameEntries(&from_data);
    std::sort(from_data.begin(), from_data.end(), &StackFrameEntryComparator);

    std::vector<Module::StackFrameEntry*> to_data;
    to_module->GetStackFrameEntries(&to_data);
    std::sort(to_data.begin(), to_data.end(), &StackFrameEntryComparator);

    Iterator to_it = to_data.begin();

    for (Iterator it = from_data.begin(); it != from_data.end(); ++it) {
        Module::StackFrameEntry* from_entry = *it;
        Module::Address from_entry_end = from_entry->address + from_entry->size;

        // Find the first CFI record in the |to_module| that does not have an
        // address less than the entry to be copied.
        while (to_it != to_data.end()) {
            if (from_entry->address > (*to_it)->address)
                ++to_it;
            else
                break;
        }

        // If the entry does not overlap, then it is safe to copy to |to_module|.
        if (to_it == to_data.end() || (from_entry->address < (*to_it)->address &&
            from_entry_end < (*to_it)->address)) {
            to_module->AddStackFrameEntry(new Module::StackFrameEntry(*from_entry));
        }
    }
}

/**
 * Copied and modified from breakpad/tools/mac/dump_syms/dump_syms_tool.cc.
 */
char *create_symbols(const char *src_path, const char *dsym_path) {
    std::string dsym_string(dsym_path ? dsym_path : "");
    std::string src_string(src_path ? src_path : "");

    SymbolData symbol_data = generate_cfi ? ALL_SYMBOL_DATA : NO_CFI;
    DumpSymbols dump_symbols(symbol_data, handle_inter_cu_refs);

    // For x86_64 binaries, the CFI data is in the __TEXT,__eh_frame of the
    // Mach-O file, which is not copied into the dSYM. Whereas in i386, the CFI
    // data is in the __DWARF,__debug_frame section, which is moved into the
    // dSYM. Therefore, to get x86_64 CFI data, dump_syms needs to look at both
    // the dSYM and the Mach-O file. If both paths are present and CFI was
    // requested, then consider the Module as "split" and dump all the debug data
    // from the primary debug info file, the dSYM, and then dump additional CFI
    // data from the source Mach-O file.
    bool split_module = !dsym_string.empty() && !src_string.empty() && generate_cfi;
    const string& primary_file = split_module ? dsym_string : src_string;

    if (!dump_symbols.Read(primary_file)) {
        return nullptr; // Cannot load module
    }

    // NOTE: We removed architecture overrides here.
    // NOTE: We removed header only output here.

    // Read the primary file into a Breakpad Module.
    Module* module = nullptr;
    if (!dump_symbols.ReadSymbolData(&module)) {
        return nullptr; // Cannot read symbols
    }
    scoped_ptr<Module> scoped_module(module);

    // If this is a split module, read the secondary Mach-O file, from which the
    // CFI data will be extracted.
    if (split_module && primary_file == dsym_string) {
        if (!dump_symbols.Read(src_string)) {
            return nullptr; // Cannot load CFI module
        }

        Module* cfi_module = NULL;
        if (!dump_symbols.ReadSymbolData(&cfi_module)) {
            return nullptr; // Cannot read CFI symbols
        }
        scoped_ptr<Module> scoped_cfi_module(cfi_module);

        // Ensure that the modules are for the same debug code file.
        if (cfi_module->name() != module->name() ||
            cfi_module->os() != module->os() ||
            cfi_module->architecture() != module->architecture() ||
            cfi_module->identifier() != module->identifier()) {
            return nullptr; // Modules do not match
        }

        CopyCFIDataBetweenModules(module, cfi_module);
    }

    // Write the symbol file to a string and return it.
    std::ostringstream symbol_buffer;
    return module->Write(symbol_buffer, symbol_data)
        ? string_from(symbol_buffer.str())
        : nullptr; // Failed to write symbol file
}
