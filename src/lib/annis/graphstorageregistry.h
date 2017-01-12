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

#include <annis/graphstorage/graphstorage.h>

#include <map>

namespace annis
{

class GraphStorageRegistry
{
public:
  GraphStorageRegistry();
  ~GraphStorageRegistry();

  std::string getName(std::weak_ptr<const ReadableGraphStorage> weakDB);
  std::unique_ptr<ReadableGraphStorage> createGraphStorage(std::string name, StringStorage &strings, const Component &component);

  std::string getOptimizedImpl(const Component& component, GraphStatistic stats);
  std::unique_ptr<ReadableGraphStorage> createGraphStorage(StringStorage &strings, const Component &component, GraphStatistic stats);

public:
  static const std::string linearP32;
  static const std::string linearP16;
  static const std::string linearP8;
  static const std::string prepostorderO32L32;
  static const std::string prepostorderO32L8;
  static const std::string prepostorderO16L32;
  static const std::string prepostorderO16L8;
  static const std::string fallback;

private:

  std::map<Component, std::string> componentToImpl;
private:
  std::string getImplByHeuristics(const Component& component, GraphStatistic stats);

  std::string getPrePostOrderBySize(const GraphStatistic& stats, bool isTree)
  {
    std::string result = prepostorderO32L32;
    if(stats.valid)
    {
      if(isTree)
      {
        // There are exactly two order values per node and there can be only one order value per node
        // in a tree.
        if(stats.nodes < (std::numeric_limits<uint16_t>::max() / 2)
           && static_cast<int64_t>(stats.maxDepth) < std::numeric_limits<int8_t>::max())
        {
          result = prepostorderO16L8;
        }
        else if(stats.nodes < (std::numeric_limits<uint16_t>::max() / 2)
                && static_cast<int64_t>(stats.maxDepth) < std::numeric_limits<int32_t>::max())
        {
          result = prepostorderO16L32;
        }
        else if( stats.nodes < (std::numeric_limits<uint32_t>::max() / 2)
                && static_cast<int64_t>(stats.maxDepth) < std::numeric_limits<int8_t>::max())
        {
          result = prepostorderO32L8;
        }
      }
      else
      {
        if(static_cast<int64_t>(stats.maxDepth) < std::numeric_limits<int8_t>::max())
        {
          result = prepostorderO32L8;
        }
      }
    }
    return result;
  }

};
}
