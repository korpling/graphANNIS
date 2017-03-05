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
#include <annis/graphstorageholder.h>    // for GraphStorageHolder
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

namespace annis { class WriteableGraphStorage; }  // lines 43-43
namespace annis { namespace api { class GraphUpdate; } }  // lines 40-40

namespace annis
{
  
class DB
{
public:
  DB();

  bool loadRelANNIS(std::string dirPath);
  bool load(std::string dir, bool preloadComponents=true);
  bool save(std::string dir);

  inline std::string getNodeName(const nodeid_t &id) const
  {
    std::string result = "";

    std::vector<Annotation> anno = nodeAnnos.getAnnotations(strings, id, annis_ns, annis_node_name);
    if(!anno.empty())
    {
      result = strings.str(anno[0].val);
    }
    return result;
  }

  inline boost::optional<nodeid_t> getNodeID(const std::string& nodeName)
  {
    std::pair<bool, nodeid_t> nodeNameID = strings.findID(nodeName);
    if(nodeNameID.first)
    {
      auto it = nodeAnnos.inverseAnnotations.find(
         {annisNodeNameStringID, annisNamespaceStringID, nodeNameID.second});

      if(it != nodeAnnos.inverseAnnotations.end())
      {
         return boost::optional<nodeid_t>(it->second);
      }
    }
    return boost::optional<nodeid_t>();
  }

  inline std::string getNodeDocument(const nodeid_t &id) const
  {
    std::string result = "";

    std::vector<Annotation> anno = nodeAnnos.getAnnotations(strings, id, annis_ns, "document");
    if(!anno.empty())
    {
      result = strings.str(anno[0].val);
    }
    return result;
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

  void convertComponent(Component c, std::string impl = "");

  void optimizeAll(const std::map<Component, std::string> &manualExceptions = std::map<Component, std::string>());

  void ensureAllComponentsLoaded();

  size_t estimateMemorySize();


  void update(const api::GraphUpdate& u);

  void clear();

  virtual ~DB();
public:

  StringStorage strings;
  AnnoStorage<nodeid_t> nodeAnnos;

  GraphStorageHolder edges;

  std::uint64_t currentChangeID;

private:


  std::uint32_t annisNamespaceStringID;
  std::uint32_t annisEmptyStringID;
  std::uint32_t annisTokStringID;
  std::uint32_t annisNodeNameStringID;

private:
  bool loadRelANNISCorpusTab(std::string dirPath, std::map<std::uint32_t, std::string> &corpusIDToName,
    bool isANNIS33Format);
  bool loadRelANNISNode(std::string dirPath, std::map<std::uint32_t, std::string> &corpusIDToName,
    bool isANNIS33Format);
  bool loadRelANNISRank(const std::string& dirPath,
                        const std::map<uint32_t, std::shared_ptr<WriteableGraphStorage> > &componentToGS,
                        bool isANNIS33Format);

  bool loadEdgeAnnotation(const std::string& dirPath,
                          const std::map<uint32_t, std::shared_ptr<WriteableGraphStorage> > &pre2GS,
                          const std::map<std::uint32_t, Edge>& pre2Edge,
                          bool isANNIS33Format);

  

  void addDefaultStrings();

  nodeid_t nextFreeNodeID() const;


};

} // end namespace annis
