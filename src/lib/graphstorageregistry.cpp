#include "graphstorageregistry.h"

#include "edgedb/coverageedb.h"
#include "edgedb/fallbackedgedb.h"
#include "edgedb/linearedgedb.h"
#include "edgedb/prepostorderstorage.h"

using namespace annis;

GraphStorageRegistry::GraphStorageRegistry()
{

}

GraphStorageRegistry::~GraphStorageRegistry()
{

}

std::string annis::GraphStorageRegistry::getName(const annis::EdgeDB *db)
{
  if(dynamic_cast<const CoverageEdgeDB*>(db) != nullptr)
  {
    return "coverage";
  }
  else if(dynamic_cast<const LinearEdgeDB*>(db) != nullptr)
  {
    return "linear";
  }
  else if(dynamic_cast<const PrePostOrderStorage*>(db) != nullptr)
  {
    return "prepostorder";
  }
  else if(dynamic_cast<const FallbackEdgeDB*>(db) != nullptr)
  {
    return "fallback";
  }
  return "unknown";
}

EdgeDB *GraphStorageRegistry::createEdgeDB(std::string name, StringStorage& strings, const Component& component)
{
  if(name == "coverage")
  {
    return new CoverageEdgeDB(strings, component);
  }
  else if(name == "linear")
  {
    return new LinearEdgeDB(strings, component);
  }
  else if(name == "prepostorder")
  {
    return new PrePostOrderStorage(strings, component);
  }
  else if(name == "fallback")
  {
    return new FallbackEdgeDB(strings, component);
  }

  return nullptr;
}
