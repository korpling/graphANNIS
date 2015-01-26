#ifndef GRAPHSTORAGEREGISTRY_H
#define GRAPHSTORAGEREGISTRY_H

#include "edgedb.h"

#include <map>

namespace annis
{

class GraphStorageRegistry
{
public:
  GraphStorageRegistry();
  ~GraphStorageRegistry();

  std::string getName(const EdgeDB* db);
  EdgeDB* createEdgeDB(std::string name, StringStorage &strings, const Component &component);

  std::string getOptimizedImpl(const Component& component);
  EdgeDB* createEdgeDB(StringStorage &strings, const Component &component);

  void setImplementation(std::string implName, ComponentType type);
  void setImplementation(std::string implName, ComponentType type, std::string layer);
  void setImplementation(std::string implName, ComponentType type, std::string layer, std::string name);
public:
  const std::string linear = "linear";
  const std::string coverage = "coverage";
  const std::string prepostorder = "prepostorder";
  const std::string fallback = "fallback";

private:

  std::map<Component, std::string> componentToImpl;

};

}

#endif // GRAPHSTORAGEREGISTRY_H
