/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#pragma once

#include <annis/types.h>  // for Component
#include <stddef.h>       // for size_t
#include <map>            // for map
#include <memory>         // for shared_ptr
#include <string>         // for string


namespace annis
{

  class ThreadPool;

  enum class NonParallelJoin {index, seed};
  enum class ParallelJoin {task, thread};

  struct QueryConfig
  {
    /** If false do not perform any optimizations */
    bool optimize;
    bool optimize_operand_order;
    bool optimize_unbound_regex;
    bool optimize_nodeby_edgeanno;
    bool optimize_join_order;
    bool all_permutations_threshold;

    bool forceFallback;
    bool avoidNestedBySwitch;

    std::map<Component, std::string> overrideImpl;

    size_t numOfBackgroundTasks;
    bool enableTaskIndexJoin;
    bool enableThreadIndexJoin;
    bool enableSIMDIndexJoin;
    std::shared_ptr<ThreadPool> threadPool;

  public:
    QueryConfig();
  };
}

