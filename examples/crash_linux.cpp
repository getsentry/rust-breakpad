#include <stdio.h>
#include "client/linux/handler/exception_handler.h"

namespace {
bool callback(const google_breakpad::MinidumpDescriptor &descriptor,
              void *context,
              bool succeeded) {
  if (succeeded) {
    printf("Dumped to: %s\n", descriptor.path());
  } else {
    printf(
        "Could not generate dump. If running in docker, pass --security-opt "
        "seccomp:unconfined");
  }

  return succeeded;
}

void crash() {
  volatile int *a = (int *)(NULL);
  *a = 1;
}

void start() {
  crash();
}
}

int main(int argc, char *argv[]) {
  google_breakpad::MinidumpDescriptor descriptor("target");
  google_breakpad::ExceptionHandler eh(descriptor, 0, callback, 0, true, -1);

  start();
  return 0;
}
