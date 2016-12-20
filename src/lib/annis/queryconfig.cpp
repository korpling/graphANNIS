#include "queryconfig.h"

#include <thread>

annis::QueryConfig::QueryConfig()
  : optimize(true), forceFallback(false), avoidNestedBySwitch(true),
    numOfParallelTasks(std::thread::hardware_concurrency()-1)

{

}
