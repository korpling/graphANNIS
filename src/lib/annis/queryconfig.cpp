#include "queryconfig.h"

#include <thread>

annis::QueryConfig::QueryConfig()
  : optimize(true), forceFallback(false), avoidNestedBySwitch(true), nonParallelJoinImpl(NonParallelJoin::index) , threadPool(nullptr)

{
  size_t numOfCPUs = std::thread::hardware_concurrency();
  if(numOfCPUs >= 2)
  {
    threadPool = std::make_shared<ThreadPool>(std::thread::hardware_concurrency()-1);
  }
}
