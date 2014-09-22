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
  DB();

  bool loadRelANNIS(std::string dirPath);
  bool load(std::string dirPath);
  bool save(std::string dirPath);

  bool hasNode(nodeid_t id);
  std::vector<Annotation> getNodeAnnotationsByID(const nodeid_t &id)
  {
    typedef stx::btree_multimap<nodeid_t, Annotation>::const_iterator AnnoIt;

    std::vector<Annotation> result;
    result.reserve(10);
    std::pair<AnnoIt,AnnoIt> itRange = nodeAnnotations.equal_range(id);
    for(AnnoIt itAnnos = itRange.first;
        itAnnos != itRange.second; itAnnos++)
    {
      result.push_back(itAnnos->second);
    }

    return result;
  }

  std::vector<Component> getDirectConnected(const Edge& edge);
  const EdgeDB* getEdgeDB(const Component& component);
  std::vector<Annotation> getEdgeAnnotations(const Component& component,
                                             const Edge& edge);
  std::string info();
  virtual ~DB();


  StringStorage strings;

private:
  stx::btree_multimap<nodeid_t, Annotation> nodeAnnotations;
  stx::btree_multimap<Annotation, nodeid_t, compAnno> inverseNodeAnnotations;

  std::map<Component, EdgeDB*, compComponent> edgeDatabases;

  bool loadRelANNISNode(std::string dirPath);
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
