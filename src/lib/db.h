#ifndef ANNISDB_H
#define ANNISDB_H

#include <string>
#include <stx/btree_map>
#include <stx/btree_multimap>
#include <stx/btree_set>
#include <cstdint>
#include <iostream>
#include <sstream>

#include "tupel.h"
#include "comparefunctions.h"

namespace annis
{

class DB
{
public:
  DB();

  bool loadRelANNIS(std::string file);

  Node getNodeByID(std::uint32_t id);
  std::vector<NodeAnnotation> getNodeAnnotationsByID(std::uint32_t id);

  std::vector<Edge> getEdgesBetweenNodes(std::uint32_t sourceID, std::uint32_t targetID);

private:
  stx::btree_map<std::uint32_t, Node> nodes;
  stx::btree_multimap<std::uint32_t, NodeAnnotation> nodeAnnotations;
  stx::btree_set<Edge, compEdges> edges;

  std::vector<std::string> nextCSV(std::istream &in);

  bool loadRelANNISRank(const std::string& dirPath);

  std::uint32_t uint32FromString(const std::string& str)
  {
    std::uint32_t result = 0;
    std::stringstream stream(str);
    stream >> result;
    return result;
  }
};

} // end namespace annis
#endif // ANNISDB_H
