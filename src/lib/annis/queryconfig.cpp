#include "queryconfig.h"

#include <thread>

annis::QueryConfig::QueryConfig()
  : optimize(true), forceFallback(false), numOfParallelTasks(std::thread::hardware_concurrency())
{

}
