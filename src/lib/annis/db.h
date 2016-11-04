#pragma once

#include <string>
#include <google/btree_map.h>
#include <cstdint>
#include <iostream>
#include <sstream>
#include <map>
#include <vector>
#include <list>

#include <boost/optional.hpp>

#include <boost/thread/shared_mutex.hpp>
#include <boost/thread/lockable_adapter.hpp>

#include <annis/types.h>
#include <annis/stringstorage.h>
#include <annis/graphstorageregistry.h>
#include <annis/graphstorageholder.h>
#include <annis/nodeannostorage.h>

namespace annis
{

   namespace api {
      class GraphUpdate;
   }
  
class ReadableGraphStorage;
class WriteableGraphStorage;
  
class DB : public boost::shared_lockable_adapter<boost::shared_mutex>
{
public:
  DB();

  bool loadRelANNIS(std::string dirPath);
  bool load(std::string dirPath, bool preloadComponents=true);
  bool save(std::string dirPath);

  inline std::string getNodeName(const nodeid_t &id) const
  {
    std::string result = "";

    std::pair<bool, Annotation> anno = nodeAnnos.getNodeAnnotation(id, annis_ns, annis_node_name);
    if(anno.first)
    {
      result = strings.str(anno.second.val);
    }
    return result;
  }

  inline boost::optional<nodeid_t> getNodeID(const std::string& nodeName)
  {
    std::pair<bool, nodeid_t> nodeNameID = strings.findID(nodeName);
    if(nodeNameID.first)
    {
      auto it = nodeAnnos.inverseNodeAnnotations.find(
         {annisNodeNameStringID, annisNamespaceStringID, nodeNameID.second});

      if(it != nodeAnnos.inverseNodeAnnotations.end())
      {
         return boost::optional<nodeid_t>(it->second);
      }
    }
    return boost::optional<nodeid_t>();
  }

  inline std::string getNodeDocument(const nodeid_t &id) const
  {
    std::string result = "";

    std::pair<bool, Annotation> anno = nodeAnnos.getNodeAnnotation(id, annis_ns, "document");
    if(anno.first)
    {
      result = strings.str(anno.second.val);
    }
    return result;
  }

  inline std::string getNodeDebugName(const nodeid_t &id) const
  {
    std::stringstream ss;
    ss << getNodeDocument(id) << "/" << getNodeName(id) << "(" << id << ")";

    return ss.str();
  }


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

  virtual ~DB();
public:

  StringStorage strings;
  NodeAnnoStorage nodeAnnos;

  GraphStorageHolder edges;

  std::uint64_t currentChangeID;

private:


  std::uint32_t annisNamespaceStringID;
  std::uint32_t annisEmptyStringID;
  std::uint32_t annisTokStringID;
  std::uint32_t annisNodeNameStringID;

private:
  bool loadRelANNISCorpusTab(std::string dirPath, std::map<std::uint32_t, std::uint32_t>& corpusIDToName,
    bool isANNIS33Format);
  bool loadRelANNISNode(std::string dirPath, std::map<std::uint32_t, std::uint32_t>& corpusIDToName,
    bool isANNIS33Format);
  bool loadRelANNISRank(const std::string& dirPath,
                        const std::map<uint32_t, std::shared_ptr<WriteableGraphStorage> > &componentToGS,
                        bool isANNIS33Format);

  bool loadEdgeAnnotation(const std::string& dirPath,
                          const std::map<uint32_t, std::shared_ptr<WriteableGraphStorage> > &pre2GS,
                          const std::map<std::uint32_t, Edge>& pre2Edge,
                          bool isANNIS33Format);

  
  void clear();
  void addDefaultStrings();

};

} // end namespace annis
