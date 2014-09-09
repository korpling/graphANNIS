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
  std::vector<Annotation> getNodeAnnotationsByID(const std::uint32_t &id);
  std::vector<Annotation> getEdgeAnnotations(const Edge& edge);

  std::vector<Edge> getEdgesBetweenNodes(std::uint32_t sourceID, std::uint32_t targetID);
  std::vector<Edge> getInEdges(std::uint32_t nodeID);

private:
  stx::btree_map<std::uint32_t, Node> nodes;
  stx::btree_multimap<std::uint32_t, Annotation> nodeAnnotations;
  stx::btree_set<Edge, compEdges> edges;
  stx::btree_multimap<Edge, Annotation, compEdges> edgeAnnotations;

  std::vector<std::string> nextCSV(std::istream &in);

  bool loadRelANNISRank(const std::string& dirPath);
  bool loadEdgeAnnotation(const std::string& dirPath,
                          const stx::btree_map<std::uint32_t, std::uint32_t> &pre2NodeID,
                          const stx::btree_map<std::uint32_t, Edge> &pre2Edge);

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
