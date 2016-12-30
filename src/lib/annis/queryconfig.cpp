#include "queryconfig.h"

#include <thread>

annis::QueryConfig::QueryConfig()
  : optimize(true), forceFallback(false), avoidNestedBySwitch(true) ,
    numOfBackgroundTasks(0), enableTaskIndexJoin(false), enableThreadIndexJoin(false), threadPool(nullptr)

{
//  size_t numOfCPUs = std::thread::hardware_concurrency();
//  if(numOfCPUs > 0)
//  {
//    numOfBackgroundTasks = numOfCPUs-1;
//    threadPool = std::make_shared<ThreadPool>(numOfBackgroundTasks);
//  }
}
