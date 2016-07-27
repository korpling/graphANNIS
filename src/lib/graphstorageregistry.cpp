#include <annis/graphstorageregistry.h>

#include <annis/graphstorage/adjacencyliststorage.h>
#include <annis/graphstorage/linearstorage.h>
#include <annis/graphstorage/prepostorderstorage.h>

#include <memory>

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
    return std::unique_ptr<ReadableGraphStorage>(new LinearP32(strings, component));
  }
  else if(name == linearP16)
  {
    return std::unique_ptr<ReadableGraphStorage>(new LinearP16(strings, component));
  }
  else if(name == linearP8)
  {
    return std::unique_ptr<ReadableGraphStorage>(new LinearP8(strings, component));
  }
  else if(name == prepostorderO32L32)
  {
    return std::unique_ptr<ReadableGraphStorage>(new PrePostOrderO32L32(strings, component));
  }
  else if(name == prepostorderO32L8)
  {
    return std::unique_ptr<ReadableGraphStorage>(new PrePostOrderO32L8(strings, component));
  }
  else if(name == prepostorderO16L32)
  {
    return std::unique_ptr<ReadableGraphStorage>(new PrePostOrderO16L32(strings, component));
  }
  else if(name == prepostorderO16L8)
  {
    return std::unique_ptr<ReadableGraphStorage>(new PrePostOrderO16L8(strings, component));
  }
  else if(name == fallback)
  {
    return std::unique_ptr<ReadableGraphStorage>(new AdjacencyListStorage(strings, component));
  }

  return std::unique_ptr<ReadableGraphStorage>();
}

std::string GraphStorageRegistry::getOptimizedImpl(const Component &component, GraphStatistic stats)
{
  std::string result = getImplByRegistry(component);
  if(result.empty())
  {
    result = getImplByHeuristics(component, stats);
  }
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

void GraphStorageRegistry::setImplementation(std::string implName, ComponentType type)
{
  Component c = {type, "", ""};
  componentToImpl[c] = implName;
}

void GraphStorageRegistry::setImplementation(std::string implName, ComponentType type, std::string layer)
{
  Component c = {type, layer, ""};
  componentToImpl[c] = implName;
}

void GraphStorageRegistry::setImplementation(std::string implName, ComponentType type, std::string layer, std::string name)
{
  Component c = {type, layer, name};
  componentToImpl[c] = implName;
}

std::string GraphStorageRegistry::getImplByRegistry(const Component &component)
{
  std::string result = "";
  // try to find a fully matching entry
  auto it = componentToImpl.find(component);
  if(it != componentToImpl.end())
  {
    result = it->second;
  }
  else
  {
    // try without the name
    Component withoutName = {component.type, component.layer, ""};
    it = componentToImpl.find(withoutName);
    if(it != componentToImpl.end())
    {
      result = it->second;
    }
    else
    {
      // try only the component type
      Component onlyType = {component.type, "", ""};
      it = componentToImpl.find(onlyType);
      if(it != componentToImpl.end())
      {
        result = it->second;
      }
    }
  }

  return result;
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
