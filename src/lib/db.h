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
public:
  DB();

  bool loadRelANNIS(std::string dirPath);
  bool load(std::string dirPath);
  bool save(std::string dirPath);

  Node getNodeByID(std::uint32_t id);
  std::vector<Annotation> getNodeAnnotationsByID(const std::uint32_t &id);

  std::vector<Component> getDirectConnected(const Edge& edge);
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
  virtual ~DB();

private:
  stx::btree_map<std::uint32_t, Node> nodes;
  stx::btree_multimap<std::uint32_t, Annotation> nodeAnnotations;

  stx::btree_map<std::uint32_t, std::string> stringStorageByID;
  stx::btree_map<std::string, std::uint32_t> stringStorageByValue;

  std::map<Component, EdgeDB*, compComponent> edgeDatabases;

  std::vector<std::string> nextCSV(std::istream &in);
  void writeCSVLine(std::ostream &out, std::vector<std::string> data);

  bool loadRelANNISRank(const std::string& dirPath, std::map<uint32_t, EdgeDB*>& componentToEdgeDB);

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

  void clear();

  EdgeDB *createEdgeDBForComponent(const std::string& type, const std::string& ns,
                       const std::string& name);
};

} // end namespace annis
#endif // ANNISDB_H
