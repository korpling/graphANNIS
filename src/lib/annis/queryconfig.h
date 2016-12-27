#pragma once

#include <map>
#include <memory>
#include <annis/util/threadpool.h>

#include <annis/types.h>

namespace annis
{
  struct QueryConfig
  {
    bool optimize;
    bool forceFallback;
    bool avoidNestedBySwitch;

    bool useSeedJoin;

    std::map<Component, std::string> overrideImpl;
    std::shared_ptr<ThreadPool> threadPool;

  public:
    QueryConfig();
  };
}

