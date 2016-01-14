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

  std::string getName(const ReadableGraphStorage *db);
  ReadableGraphStorage* createGraphStorage(std::string name, StringStorage &strings, const Component &component);

  std::string getOptimizedImpl(const Component& component, GraphStatistic stats);
  ReadableGraphStorage* createGraphStorage(StringStorage &strings, const Component &component, GraphStatistic stats);

  void setImplementation(std::string implName, ComponentType type);
  void setImplementation(std::string implName, ComponentType type, std::string layer);
  void setImplementation(std::string implName, ComponentType type, std::string layer, std::string name);
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
  std::string getImplByRegistry(const Component& component);
  std::string getImplByHeuristics(const Component& component, GraphStatistic stats);

  std::string getPrePostOrderBySize(const GraphStatistic& stats, bool isTree)
  {
    std::string result = prepostorderO32L32;
    if(stats.valid)
    {
      if(isTree)
      {
        if(stats.nodes < std::numeric_limits<uint16_t>::max()
           && stats.maxDepth < std::numeric_limits<int8_t>::max())
        {
          result = prepostorderO16L8;
        }
        else if(stats.nodes < std::numeric_limits<uint16_t>::max()
                && stats.maxDepth < std::numeric_limits<int32_t>::max())
        {
          result = prepostorderO16L32;
        }
        else if( stats.nodes < std::numeric_limits<uint32_t>::max()
                && stats.maxDepth < std::numeric_limits<int8_t>::max())
        {
          result = prepostorderO32L8;
        }
      }
      else
      {
        if(stats.maxDepth < std::numeric_limits<int8_t>::max())
        {
          result = prepostorderO32L8;
        }
      }
    }
    return result;
  }

};
}
