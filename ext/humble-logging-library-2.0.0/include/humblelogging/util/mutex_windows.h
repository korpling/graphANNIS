#ifndef HUMBLE_MUTEX_WINDOWS_HEADER
#define HUMBLE_MUTEX_WINDOWS_HEADER

#include <Windows.h>

#include "humblelogging/defines.h"

namespace humble {
namespace logging {

class HUMBLE_EXPORT_API Mutex
{
public:
  Mutex();
  ~Mutex();
  void lock();
  void unlock();

private:
  CRITICAL_SECTION _criticalSection;
};

}} // End of namespace.
#endif
