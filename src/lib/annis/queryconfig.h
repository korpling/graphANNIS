#pragma once

#include <map>
#include <memory>
#include <ThreadPool.h>

#include <annis/types.h>

namespace annis
{
  struct QueryConfig
  {
    bool optimize;
    bool forceFallback;
    bool avoidNestedBySwitch;

    std::map<Component, std::string> overrideImpl;
    std::shared_ptr<ThreadPool> threadPool;

  public:
    QueryConfig();
  };
}

