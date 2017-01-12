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

#include "graphstorageregistry.h"

#include <annis/graphstorage/adjacencyliststorage.h>  // for AdjacencyListSt...
#include <annis/graphstorage/linearstorage.h>         // for LinearStorage
#include <annis/graphstorage/prepostorderstorage.h>   // for PrePostOrderSto...
#include <cstdint>                                    // for uint32_t, int32_t
#include <memory>                                     // for unique_ptr, dyn...
#include <utility>                                    // for pair
#include <vector>                                     // for vector
#include "annis/annostorage.h"                        // for AnnoStorage
#include "annis/graphstorage/graphstorage.h"          // for ReadableGraphSt...
#include <annis/types.h>

namespace annis { class StringStorage; }

using namespace annis;

using PrePostOrderO32L32 = PrePostOrderStorage<uint32_t, int32_t>;
using PrePostOrderO32L8 = PrePostOrderStorage<uint32_t, int8_t>;
using PrePostOrderO16L32 = PrePostOrderStorage<uint16_t, int32_t>;
using PrePostOrderO16L8 = PrePostOrderStorage<uint16_t, int8_t>;

using LinearP32 = LinearStorage<uint32_t>;
using LinearP16 = LinearStorage<uint16_t>;
using LinearP8 = LinearStorage<uint8_t>;

const std::string GraphStorageRegistry::linearP32 = "linear";
const std::string GraphStorageRegistry::linearP16 = "linearP16";
const std::string GraphStorageRegistry::linearP8 = "linearP8";
const std::string GraphStorageRegistry::prepostorderO32L32 = "prepostorder";
const std::string GraphStorageRegistry::prepostorderO32L8 = "prepostorderO32L8";
const std::string GraphStorageRegistry::prepostorderO16L32 = "prepostorderO16L32";
const std::string GraphStorageRegistry::prepostorderO16L8 = "prepostorderO16L8";
const std::string GraphStorageRegistry::fallback = "fallback";

GraphStorageRegistry::GraphStorageRegistry()
{
}

GraphStorageRegistry::~GraphStorageRegistry()
{

}

std::string annis::GraphStorageRegistry::getName(std::weak_ptr<const ReadableGraphStorage> weakDB)
{
  if(auto db =  weakDB.lock())
  {
    if(std::dynamic_pointer_cast<const LinearP32>(db) != nullptr)
    {
      return linearP32;
    }
    else if(std::dynamic_pointer_cast<const LinearP16>(db) != nullptr)
    {
      return linearP16;
    }
    else if(std::dynamic_pointer_cast<const LinearP8>(db) != nullptr)
    {
      return linearP8;
    }
    else if(std::dynamic_pointer_cast<const PrePostOrderO32L32>(db) != nullptr)
    {
      return prepostorderO32L32;
    }
    else if(std::dynamic_pointer_cast<const PrePostOrderO32L8>(db) != nullptr)
    {
      return prepostorderO32L8;
    }
    else if(std::dynamic_pointer_cast<const PrePostOrderO16L32>(db) != nullptr)
    {
      return prepostorderO16L32;
    }
    else if(std::dynamic_pointer_cast<const PrePostOrderO16L8>(db) != nullptr)
    {
      return prepostorderO16L8;
    }
    else if(std::dynamic_pointer_cast<const AdjacencyListStorage>(db) != nullptr)
    {
      return fallback;
    }
  }
  return "";
}

std::unique_ptr<ReadableGraphStorage> GraphStorageRegistry::createGraphStorage(std::string name, StringStorage& strings, const Component& component)
{
  if(name == linearP32)
  {
    return std::unique_ptr<ReadableGraphStorage>(new LinearP32());
  }
  else if(name == linearP16)
  {
    return std::unique_ptr<ReadableGraphStorage>(new LinearP16());
  }
  else if(name == linearP8)
  {
    return std::unique_ptr<ReadableGraphStorage>(new LinearP8());
  }
  else if(name == prepostorderO32L32)
  {
    return std::unique_ptr<ReadableGraphStorage>(new PrePostOrderO32L32());
  }
  else if(name == prepostorderO32L8)
  {
    return std::unique_ptr<ReadableGraphStorage>(new PrePostOrderO32L8());
  }
  else if(name == prepostorderO16L32)
  {
    return std::unique_ptr<ReadableGraphStorage>(new PrePostOrderO16L32());
  }
  else if(name == prepostorderO16L8)
  {
    return std::unique_ptr<ReadableGraphStorage>(new PrePostOrderO16L8());
  }
  else if(name == fallback)
  {
    return std::unique_ptr<ReadableGraphStorage>(new AdjacencyListStorage());
  }

  return std::unique_ptr<ReadableGraphStorage>();
}

std::string GraphStorageRegistry::getOptimizedImpl(const Component &component, GraphStatistic stats)
{
  std::string result = getImplByHeuristics(component, stats);

  if(result.empty())
  {
    result = fallback;
  }

  return result;
}

std::unique_ptr<ReadableGraphStorage> GraphStorageRegistry::createGraphStorage(StringStorage &strings, const Component &component, GraphStatistic stats)
{
  std::string implName = getOptimizedImpl(component, stats);
  return createGraphStorage(implName, strings, component);
}

std::string GraphStorageRegistry::getImplByHeuristics(const Component &component, GraphStatistic stats)
{
  std::string result = fallback;

  if(stats.valid)
  {
    if(stats.maxDepth <= 1)
    {
      // if we don't have any deep graph structures an adjencency list is always fasted (and has no overhead)
      result = fallback;
    }
    else if(stats.rootedTree)
    {
      if(stats.maxFanOut <= 1)
      {
        // a tree where all nodes belong to the same path
        if(stats.maxDepth < std::numeric_limits<uint8_t>::max())
        {
          result = linearP8;
        }
        else if(stats.maxDepth < std::numeric_limits<uint16_t>::max())
        {
          result = linearP16;
        }
        else if(stats.maxDepth < std::numeric_limits<uint32_t>::max())
        {
          result = linearP32;
        }
      }
      else
      {
        // we have a real tree
        result = getPrePostOrderBySize(stats, true);
      }
    }
    else if(!stats.cyclic)
    {
      // it might be still wise to use pre/post order if the graph is "almost" a tree, thus
      // does not have many exceptions
      if(stats.dfsVisitRatio <= 1.03)
      {
        // there is no more than 3% overhead
        // TODO: how to determine the border?
        result = getPrePostOrderBySize(stats, false);
      }
    }
  }


  return result;
}
