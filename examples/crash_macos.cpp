// To build:
// g++ -g -o linux_test_app -I ../../ -L../../client/mac crash_macos.cc \
//   -lbreakpad

#include "client/mac/handler/exception_handler.h"

namespace {

void CrashFunction() {
  int *i = reinterpret_cast<int*>(0x45);
  *i = 5;  // crash!
}

void SomeOtherFunction() {
  CrashFunction();
}

}

int main(int argc, char **argv) {
  google_breakpad::ExceptionHandler eh("target", nullptr, nullptr, nullptr, true, nullptr);
  SomeOtherFunction();
  return 0;
}
