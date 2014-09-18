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

#include "types.h"
#include "comparefunctions.h"
#include "edgedb.h"

namespace annis
{
class DB
{
  friend class AnnotationNameSearch;
public:
  DB();

  bool loadRelANNIS(std::string dirPath);
  bool load(std::string dirPath);
  bool save(std::string dirPath);

  bool hasNode(std::uint32_t id);
  std::vector<Annotation> getNodeAnnotationsByID(const std::uint32_t &id);

  std::vector<Component> getDirectConnected(const Edge& edge);
  const EdgeDB* getEdgeDB(const Component& component);
  std::vector<Annotation> getEdgeAnnotations(const Component& component,
                                             const Edge& edge);
  std::string info();

  const std::string& str(std::uint32_t id)
  {
    typedef stx::btree_map<std::uint32_t, std::string>::const_iterator ItType;
    ItType it = stringStorageByID.find(id);
    if(it != stringStorageByID.end())
    {
      return it->second;
    }
    else
    {
      throw("Unknown string ID");
    }
  }

  std::pair<bool, std::uint32_t> findString(const std::string& str)
  {
    typedef stx::btree_map<std::string, std::uint32_t>::const_iterator ItType;
    std::pair<bool, std::uint32_t> result;
    result.first = false;
    ItType it = stringStorageByValue.find(str);
    if(it != stringStorageByValue.end())
    {
      result.first = true;
      result.second = it->second;
    }
    return result;
  }

  virtual ~DB();

private:
  stx::btree_multimap<std::uint32_t, Annotation> nodeAnnotations;
  stx::btree_multimap<Annotation, std::uint32_t, compAnno> inverseNodeAnnotations;

  stx::btree_map<std::uint32_t, std::string> stringStorageByID;
  stx::btree_map<std::string, std::uint32_t> stringStorageByValue;

  std::map<Component, EdgeDB*, compComponent> edgeDatabases;

  std::vector<std::string> nextCSV(std::istream &in);
  void writeCSVLine(std::ostream &out, std::vector<std::string> data);

  bool loadRelANNISNode(std::string dirPath);
  bool loadRelANNISRank(const std::string& dirPath,
                        const std::map<uint32_t, EdgeDB*>& componentToEdgeDB);

  bool loadEdgeAnnotation(const std::string& dirPath,
                          const std::map<std::uint32_t, EdgeDB* >& pre2EdgeDB,
                          const std::map<std::uint32_t, Edge>& pre2Edge);

  std::uint32_t addString(const std::string& str);

  std::uint32_t uint32FromString(const std::string& str)
  {
    std::uint32_t result = 0;
    std::stringstream stream(str);
    stream >> result;
    return result;
  }

  std::string stringFromUInt32(const std::uint32_t& val)
  {
    std::stringstream stream("");
    stream << val;
    return stream.str();
  }

  void addNodeAnnotation(std::uint32_t nodeID, Annotation& anno)
  {
    nodeAnnotations.insert2(nodeID, anno);
    inverseNodeAnnotations.insert2(anno, nodeID);
  }

  void clear();

  EdgeDB *createEdgeDBForComponent(const std::string& shortType, const std::string& layer,
                       const std::string& name);
  EdgeDB *createEdgeDBForComponent(ComponentType ctype, const std::string& layer,
                       const std::string& name);
};

} // end namespace annis
#endif // ANNISDB_H
