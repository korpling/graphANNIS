#include "humblelogging/util/mutex_linux.h"

#include <mutex>

namespace humble {
namespace logging {

Mutex::Mutex()
  : _m(_internalMutex)
{
  // the initial state of the unique_mutex is "locked";
  _m.unlock();
}

Mutex::~Mutex()
{
}

void Mutex::lock()
{
  _m.lock();
}

void Mutex::unlock()
{
  _m.unlock();
}

}}  // End of namespace.
