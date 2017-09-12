#include "client/mac/handler/exception_handler.h"

namespace {
bool callback(const char *dump_dir,
              const char *minidump_id,
              void *context,
              bool succeeded) {
  if (succeeded) {
    printf("Dumped to: %s/%s.dmp\n", dump_dir, minidump_id);
  } else {
    printf("Could not generate dump.");
  }

  return succeeded;
}

void crash() {
  int *i = reinterpret_cast<int *>(0x45);
  *i = 5;  // crash!
}

void start() {
  crash();
}
}

int main(int argc, char **argv) {
  google_breakpad::ExceptionHandler eh("target", 0, callback, 0, true, 0);
  start();
  return 0;
}
