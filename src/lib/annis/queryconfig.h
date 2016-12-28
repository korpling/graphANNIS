#pragma once

#include <map>
#include <memory>
#include <annis/util/threadpool.h>

#include <annis/types.h>

namespace annis
{

  enum class NonParallelJoin {index, seed};
  enum class ParallelJoin {task, thread};

  struct QueryConfig
  {
    bool optimize;
    bool forceFallback;
    bool avoidNestedBySwitch;

    NonParallelJoin nonParallelJoinImpl;

    std::map<Component, std::string> overrideImpl;
    std::shared_ptr<ThreadPool> threadPool;

  public:
    QueryConfig();
  };
}

