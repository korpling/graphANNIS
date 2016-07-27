#pragma once


#include <annis/graphstorage/graphstorage.h>
#include <annis/graphstorageregistry.h>

#include <map>
#include <memory>

namespace annis
{

class GraphStorageHolder
{
  using GraphStorageIt = std::map<Component, std::shared_ptr<ReadableGraphStorage>>::const_iterator;

public:

  GraphStorageHolder(StringStorage& strings);
  virtual ~GraphStorageHolder();

  std::weak_ptr<const ReadableGraphStorage> getGraphStorage(const Component& component);
  std::weak_ptr<const ReadableGraphStorage> getGraphStorage(ComponentType type, const std::string& layer, const std::string& name);
  std::vector<std::weak_ptr<const ReadableGraphStorage>> getGraphStorage(ComponentType type, const std::string& name);
  std::vector<std::weak_ptr<const ReadableGraphStorage>> getGraphStorage(ComponentType type);

  size_t estimateMemorySize() const;
  std::string info();

private:
  friend class DB;

  bool load(std::string dirPath, bool preloadComponents);
  bool save(const std::string &dirPath);
  void clear();

  void ensureComponentIsLoaded(const Component& c);

  std::string debugComponentString(const Component& c);
  std::string getImplNameForPath(std::string directory);

  std::shared_ptr<ReadableGraphStorage> createGSForComponent(const std::string& shortType, const std::string& layer,
                       const std::string& name);
  std::shared_ptr<ReadableGraphStorage> createGSForComponent(ComponentType ctype, const std::string& layer,
                       const std::string& name);
  std::shared_ptr<annis::WriteableGraphStorage> createWritableGraphStorage(ComponentType ctype, const std::string& layer,
                       const std::string& name);


  ComponentType componentTypeFromShortName(std::string shortType);

private:

  StringStorage& strings;

  /**
   * Map containing all available graph storages.
   */
  std::map<Component, std::shared_ptr<ReadableGraphStorage>> container;
  /**
   * A map from not yet loaded components to it's location on disk.
   */
  std::map<Component, std::string> notLoadedLocations;
  GraphStorageRegistry registry;


};

} // end namespace annis
