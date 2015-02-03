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

  std::string getName(const ReadableGraphStorage *db);
  ReadableGraphStorage* createEdgeDB(std::string name, StringStorage &strings, const Component &component);

  std::string getOptimizedImpl(const Component& component, GraphStatistic stats);
  ReadableGraphStorage* createEdgeDB(StringStorage &strings, const Component &component, GraphStatistic stats);

  void setImplementation(std::string implName, ComponentType type);
  void setImplementation(std::string implName, ComponentType type, std::string layer);
  void setImplementation(std::string implName, ComponentType type, std::string layer, std::string name);
public:
  static const std::string linearP32;
  static const std::string linearP16;
  static const std::string linearP8;
  static const std::string coverage;
  static const std::string prepostorderO32L32;
  static const std::string prepostorderO32L8;
  static const std::string fallback;

private:

  std::map<Component, std::string> componentToImpl;
private:
  std::string getImplByRegistry(const Component& component);
  std::string getImplByHeuristics(const Component& component, GraphStatistic stats);

};
}

#endif // GRAPHSTORAGEREGISTRY_H
