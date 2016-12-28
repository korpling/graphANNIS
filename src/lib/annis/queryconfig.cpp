#include "queryconfig.h"

#include <thread>

annis::QueryConfig::QueryConfig()
  : optimize(true), forceFallback(false), avoidNestedBySwitch(true) ,
    numOfBackgroundTasks(0), threadPool(nullptr)

{
  size_t numOfCPUs = std::thread::hardware_concurrency();
  if(numOfCPUs > 0)
  {
    numOfBackgroundTasks = numOfCPUs-1;
  }
}
