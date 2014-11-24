#ifndef ANNISDB_H
#define ANNISDB_H

#include <string>
#include <stx/btree_map>
#include <stx/btree_multimap>
#include <stx/btree_set>
#include <cstdint>
#include <iostream>
#include <sstream>
#include <map>
#include <vector>
#include <list>

#include "types.h"
#include "comparefunctions.h"
#include "edgedb.h"
#include "stringstorage.h"

namespace annis
{
class DB
{
  friend class AnnotationNameSearch;
  typedef std::map<Component, EdgeDB*, compComponent>::const_iterator EdgeDBIt;
public:
  DB(bool useSpecializedEdgeDB = true);

  bool loadRelANNIS(std::string dirPath);
  bool load(std::string dirPath);
  bool save(std::string dirPath);

  bool hasNode(nodeid_t id);
  inline std::list<Annotation> getNodeAnnotationsByID(const nodeid_t &id) const
  {
    typedef stx::btree_multimap<nodeid_t, Annotation>::const_iterator AnnoIt;

    std::list<Annotation> result;
    std::pair<AnnoIt,AnnoIt> itRange = nodeAnnotations.equal_range(id);
    for(AnnoIt itAnnos = itRange.first;
        itAnnos != itRange.second; itAnnos++)
    {
      result.push_back(itAnnos->second);
    }

    return result;
  }

  inline std::string getNodeName(const nodeid_t &id) const
  {
    std::string result = "";

    std::pair<bool, Annotation> anno = getNodeAnnotation(id, annis_ns, annis_node_name);
    if(anno.first)
    {
      result = strings.str(anno.second.val);
    }
    return result;
  }

  inline std::string getNodeDocument(const nodeid_t &id) const
  {
    std::string result = "";

    std::pair<bool, Annotation> anno = getNodeAnnotation(id, annis_ns, "document");
    if(anno.first)
    {
      result = strings.str(anno.second.val);
    }
    return result;
  }

  inline std::pair<bool, Annotation> getNodeAnnotation(const nodeid_t &id, const std::string& ns, const std::string& name) const
  {
    typedef stx::btree_multimap<nodeid_t, Annotation>::const_iterator AnnoIt;

    std::pair<bool, Annotation> result;
    result.first = false;

    std::pair<bool, std::uint32_t> nsID = strings.findID(ns);
    std::pair<bool, std::uint32_t> nameID = strings.findID(name);

    if(nsID.first && nameID.first)
    {
      std::pair<AnnoIt,AnnoIt> itRange = nodeAnnotations.equal_range(id);
      for(AnnoIt itAnnos = itRange.first;
          itAnnos != itRange.second; itAnnos++)
      {
        Annotation anno = itAnnos->second;
        if(anno.ns == nsID.second && anno.name == nameID.second)
        {
          result.first = true;
          result.second = anno;
        }
      }
    }

    return result;
  }

  std::vector<Component> getDirectConnected(const Edge& edge);
  const EdgeDB* getEdgeDB(const Component& component) const;
  const EdgeDB* getEdgeDB(ComponentType type, const std::string& layer, const std::string& name) const;
  std::vector<const EdgeDB *> getAllEdgeDBForType(ComponentType type) const;

  std::vector<Annotation> getEdgeAnnotations(const Component& component,
                                             const Edge& edge);
  std::string info();

  std::uint32_t getNamespaceStringID() const {return annisNamespaceStringID;}
  std::uint32_t getNodeNameStringID() const {return annisNodeNameStringID;}
  std::uint32_t getEmptyStringID() const {return annisEmptyStringID;}
  std::uint32_t getTokStringID() const {return annisTokStringID;}

  virtual ~DB();


  StringStorage strings;

private:
  stx::btree_multimap<nodeid_t, Annotation> nodeAnnotations;
  stx::btree_multimap<Annotation, nodeid_t, compAnno> inverseNodeAnnotations;

  std::map<Component, EdgeDB*, compComponent> edgeDatabases;

  std::uint32_t annisNamespaceStringID;
  std::uint32_t annisEmptyStringID;
  std::uint32_t annisTokStringID;
  std::uint32_t annisNodeNameStringID;

  bool useSpecializedEdgeDB;

  bool loadRelANNISCorpusTab(std::string dirPath, std::map<std::uint32_t, std::uint32_t>& corpusIDToName);
  bool loadRelANNISNode(std::string dirPath, std::map<std::uint32_t, std::uint32_t>& corpusIDToName);
  bool loadRelANNISRank(const std::string& dirPath,
                        const std::map<uint32_t, EdgeDB*>& componentToEdgeDB);

  bool loadEdgeAnnotation(const std::string& dirPath,
                          const std::map<std::uint32_t, EdgeDB* >& pre2EdgeDB,
                          const std::map<std::uint32_t, Edge>& pre2Edge);

  void addNodeAnnotation(nodeid_t nodeID, Annotation& anno)
  {
    nodeAnnotations.insert2(nodeID, anno);
    inverseNodeAnnotations.insert2(anno, nodeID);
  }

  void clear();
  void addDefaultStrings();

  EdgeDB *createEdgeDBForComponent(const std::string& shortType, const std::string& layer,
                       const std::string& name);
  EdgeDB *createEdgeDBForComponent(ComponentType ctype, const std::string& layer,
                       const std::string& name);
};

} // end namespace annis
#endif // ANNISDB_H
