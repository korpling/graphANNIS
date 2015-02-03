#include "graphstorageregistry.h"

#include "edgedb/coverageedb.h"
#include "edgedb/fallbackedgedb.h"
#include "edgedb/linearedgedb.h"
#include "edgedb/prepostorderstorage.h"

using namespace annis;

using PrePostOrderStorageO32L32 = PrePostOrderStorage<uint32_t, int32_t>;
using PrePostOrderStorageO32L8 = PrePostOrderStorage<uint32_t, int8_t>;

const std::string GraphStorageRegistry::linear = "linear";
const std::string GraphStorageRegistry::coverage = "coverage";
const std::string GraphStorageRegistry::prepostorderO32L32 = "prepostorder";
const std::string GraphStorageRegistry::prepostorderO32L8 = "prepostorderO32L8";
const std::string GraphStorageRegistry::fallback = "fallback";

GraphStorageRegistry::GraphStorageRegistry()
{
  // set default values
  setImplementation(coverage, ComponentType::COVERAGE);
  setImplementation(linear, ComponentType::ORDERING);
  setImplementation(prepostorderO32L32, ComponentType::DOMINANCE);

}

GraphStorageRegistry::~GraphStorageRegistry()
{

}

std::string annis::GraphStorageRegistry::getName(const annis::ReadableGraphStorage *db)
{
  if(dynamic_cast<const CoverageEdgeDB*>(db) != nullptr)
  {
    return coverage;
  }
  else if(dynamic_cast<const LinearEdgeDB*>(db) != nullptr)
  {
    return linear;
  }
  else if(dynamic_cast<const PrePostOrderStorageO32L32*>(db) != nullptr)
  {
    return prepostorderO32L32;
  }
  else if(dynamic_cast<const FallbackEdgeDB*>(db) != nullptr)
  {
    return fallback;
  }
  return "";
}

ReadableGraphStorage *GraphStorageRegistry::createEdgeDB(std::string name, StringStorage& strings, const Component& component)
{
  if(name == coverage)
  {
    return new CoverageEdgeDB(strings, component);
  }
  else if(name == linear)
  {
    return new LinearEdgeDB(strings, component);
  }
  else if(name == prepostorderO32L32)
  {
    return new PrePostOrderStorageO32L32(strings, component);
  }
  else if(name == fallback)
  {
    return new FallbackEdgeDB(strings, component);
  }

  return nullptr;
}

std::string GraphStorageRegistry::getOptimizedImpl(const Component &component, GraphStatistic stats)
{
  std::string result = fallback;
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

ReadableGraphStorage *GraphStorageRegistry::createEdgeDB(StringStorage &strings, const Component &component, GraphStatistic stats)
{
  std::string implName = getOptimizedImpl(component, stats);
  return createEdgeDB(implName, strings, component);
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
