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

#include <annis/annostorage.h>           // for AnnoStorage
#include <annis/stringstorage.h>         // for StringStorage
#include <annis/types.h>                 // for nodeid_t, Annotation, annis_ns
#include <stddef.h>                      // for size_t
#include <boost/container/flat_map.hpp>  // for flat_multimap
#include <boost/container/vector.hpp>    // for operator!=, vec_iterator
#include <boost/optional/optional.hpp>   // for optional
#include <cstdint>                       // for uint32_t, uint64_t
#include <map>                           // for map
#include <memory>                        // for allocator_traits<>::value_type
#include <string>                        // for string, operator<<, char_traits
#include <utility>                       // for pair
#include <vector>                        // for vector
#include <annis/graphstorageregistry.h>

namespace annis { class WriteableGraphStorage; }  // lines 43-43
namespace annis { namespace api { class GraphUpdate; } }  // lines 40-40

namespace annis
{
  
class DB
{
public:

  using GetGSFuncT = std::function<std::shared_ptr<const ReadableGraphStorage> (ComponentType type, const std::string &layer, const std::string &name)>;
  using GetAllGSFuncT = std::function<std::vector<std::shared_ptr<const ReadableGraphStorage>> (ComponentType type, const std::string &name)>;

  DB();

  bool load(std::string dir, bool preloadComponents=true);
  bool save(std::string dir);

  inline std::string getNodeName(const nodeid_t &id) const
  {
    std::string result = "";

    boost::optional<Annotation> anno = nodeAnnos.getAnnotations(strings, id, annis_ns, annis_node_name);
    if(anno)
    {
      result = strings.str(anno->val);
    }
    return result;
  }

  std::string getNodeType(const nodeid_t &id) const
  {
    std::string result = "";

    boost::optional<Annotation> anno = nodeAnnos.getAnnotations(strings, id, annis_ns, annis_node_type);
    if(anno)
    {
      result = strings.str(anno->val);
    }
    return result;
  }

  inline boost::optional<nodeid_t> getNodeID(const std::string& nodeName) const
  {
    auto nodeNameID = strings.findID(nodeName);
    if(nodeNameID)
    {
      auto it = nodeAnnos.inverseAnnotations.find(
         {annisNodeNameStringID, annisNamespaceStringID, *nodeNameID});

      if(it != nodeAnnos.inverseAnnotations.end())
      {
         return boost::optional<nodeid_t>(it->second);
      }
    }
    return boost::optional<nodeid_t>();
  }

  std::string getNodeDebugName(const nodeid_t &id) const;


  std::vector<Component> getDirectConnected(const Edge& edge) const;
  std::vector<Component> getAllComponents() const;

  std::vector<Annotation> getEdgeAnnotations(const Component& component,
                                             const Edge& edge);
  std::string info();

  inline std::uint32_t getNamespaceStringID() const {return annisNamespaceStringID;}
  inline std::uint32_t getNodeNameStringID() const {return annisNodeNameStringID;}
  inline std::uint32_t getEmptyStringID() const {return annisEmptyStringID;}
  inline std::uint32_t getTokStringID() const {return annisTokStringID;}
  inline std::uint32_t getNodeTypeStringID() const {return annisNodeTypeID;}

  std::shared_ptr<annis::WriteableGraphStorage> createWritableGraphStorage(ComponentType ctype, const std::string& layer,
                       const std::string& name);

  std::shared_ptr<const ReadableGraphStorage> getGraphStorage(ComponentType type, const std::string& layer, const std::string& name);
  std::vector<std::shared_ptr<const ReadableGraphStorage>> getAllGraphStorages(ComponentType type, const std::string& name);

  void convertComponent(Component c, std::string impl = "");

  void optimizeAll(const std::map<Component, std::string> &manualExceptions = std::map<Component, std::string>());

  bool allGraphStoragesLoaded() const;
  bool isGraphStorageLoaded(ComponentType type, const std::string& layer, const std::string& name) const;

  bool allGraphStoragesLoaded(ComponentType type, const std::string& name);
  void ensureAllComponentsLoaded();

  size_t estimateMemorySize() const;

  void update(const api::GraphUpdate& u);

  void clear();

  nodeid_t nextFreeNodeID() const;

  virtual ~DB();
public:

  StringStorage strings;
  AnnoStorage<nodeid_t> nodeAnnos;

  const GetGSFuncT f_getGraphStorage;
  const GetAllGSFuncT f_getAllGraphStorages;


private:

  std::uint64_t currentChangeID;

  std::uint32_t annisNamespaceStringID;
  std::uint32_t annisEmptyStringID;
  std::uint32_t annisTokStringID;
  std::uint32_t annisNodeNameStringID;
  std::uint32_t annisNodeTypeID;

  /**
   * Map containing all available graph storages.
   */
  std::map<Component, std::shared_ptr<ReadableGraphStorage>> graphStorages;
  /**
   * A map from not yet loaded components to it's location on disk.
   */
  std::map<Component, std::string> notLoadedLocations;
  GraphStorageRegistry gsRegistry;

private:

  void addDefaultStrings();

  void loadGraphStorages(std::string dirPath, bool preloadComponents);
  void saveGraphStorages(std::string dirPath);

  bool ensureGraphStorageIsLoaded(const Component& c);
  size_t estimateGraphStorageMemorySize() const;
  std::string gsInfo() const;
  std::string debugComponentString(const Component& c) const;



};

} // end namespace annis
