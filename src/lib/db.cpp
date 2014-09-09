#include "db.h"

#include <iostream>
#include <fstream>
#include <sstream>
#include <limits>

#include <boost/format.hpp>
#include <humblelogging/api.h>

HUMBLE_LOGGER(logger, "annis4");

using namespace annis;

DB::DB()
{
}

std::vector<std::string> DB::nextCSV(std::istream& in)
{
  std::vector<std::string> result;
  std::string line;

  std::getline(in, line);
  std::stringstream lineStream(line);
  std::string cell;

  while(std::getline(lineStream, cell, '\t'))
  {
    result.push_back(cell);
  }
  return result;
}

bool DB::loadRelANNIS(std::string dirPath)
{
  nodes.clear();
  nodeAnnotations.clear();

  std::string nodeTabPath = dirPath + "/node.tab";
  HL_INFO(logger, (boost::format("loading %1%") % nodeTabPath).str());

  std::ifstream in;
  in.open(nodeTabPath, std::ifstream::in);
  if(!in.good()) return false;

  std::vector<std::string> line;
  while((line = nextCSV(in)).size() > 0)
  {
    std::uint32_t nodeNr;
    std::stringstream nodeNrStream(line[0]);
    nodeNrStream >> nodeNr;
    Node n;
    n.id = nodeNr;
    n.name = line[4];
    nodes[nodeNr] = n;
  }

  in.close();

  std::string nodeAnnoTabPath = dirPath + "/node_annotation.tab";
  HL_INFO(logger, (boost::format("loading %1%") % nodeAnnoTabPath).str());

  in.open(nodeAnnoTabPath, std::ifstream::in);
  if(!in.good()) return false;

  while((line = nextCSV(in)).size() > 0)
  {
    u_int32_t nodeNr = uint32FromString(line[0]);
    Annotation anno;
    anno.ns = line[1];
    anno.name = line[2];
    anno.val = line[3];
    nodeAnnotations.insert2(nodeNr, anno);
  }

  in.close();

  bool result = loadRelANNISRank(dirPath);


  return result;
}

bool DB::loadRelANNISRank(const std::string &dirPath)
{
  typedef stx::btree_map<std::uint32_t, std::uint32_t>::const_iterator UintMapIt;

  bool result = true;

  std::ifstream in;
  std::string rankTabPath = dirPath + "/rank.tab";
  HL_INFO(logger, (boost::format("loading %1%") % rankTabPath).str());

  in.open(rankTabPath, std::ifstream::in);
  if(!in.good()) return false;

  std::vector<std::string> line;

  // first run: collect all pre-order values for a node
  stx::btree_map<std::uint32_t, std::uint32_t> pre2NodeID;
  stx::btree_map<std::uint32_t, Edge> pre2Edge;
  while((line = nextCSV(in)).size() > 0)
  {
    pre2NodeID.insert2(uint32FromString(line[0]),uint32FromString(line[2]));
  }

  in.close();

  in.open(rankTabPath, std::ifstream::in);
  if(!in.good()) return false;

  // second run: get the actual edges
  while((line = nextCSV(in)).size() > 0)
  {
    std::uint32_t parent = uint32FromString(line[4]);
    UintMapIt it = pre2NodeID.find(parent);
    if(it != pre2NodeID.end())
    {
      Edge e;
      e.source = uint32FromString(line[2]);
      e.target = it->second;
      e.component = uint32FromString(line[3]);

      // since we ignore the pre-order value
      // we might add an edge several times if it has several
      // rank entries
      edges.insert(e);

      pre2Edge.insert2(uint32FromString(line[0]), e);
    }
    else
    {
      result = false;
    }
  }

  in.close();

  if(result)
  {

    result = loadEdgeAnnotation(dirPath, pre2Edge);
  }

  return result;
}

bool DB::loadEdgeAnnotation(const std::string &dirPath,
                            const stx::btree_map<std::uint32_t, Edge>& pre2Edge)
{
  typedef stx::btree_map<std::uint32_t, Edge>::const_iterator UintMapIt;

  bool result = true;

  std::ifstream in;
  std::string edgeAnnoTabPath = dirPath + "/edge_annotation.tab";
  HL_INFO(logger, (boost::format("loading %1%") % edgeAnnoTabPath).str());

  in.open(edgeAnnoTabPath, std::ifstream::in);
  if(!in.good()) return false;

  std::vector<std::string> line;

  while((line = nextCSV(in)).size() > 0)
  {
    std::uint32_t pre = uint32FromString(line[0]);
    UintMapIt it = pre2Edge.find(pre);
    if(it != pre2Edge.end())
    {
      const Edge& e = it->second;
      Annotation anno;
      anno.ns = line[1];
      anno.name = line[2];
      anno.val = line[3];
      edgeAnnotations.insert2(e, anno);
    }
    else
    {
      result = false;
    }
  }

  in.close();

  return result;
}

Node DB::getNodeByID(std::uint32_t id)
{
  stx::btree_map<std::uint32_t, Node>::const_iterator itNode = nodes.find(id);
  if(itNode == nodes.end())
  {
    // TODO: don't use exception here
    throw("Unknown node");
  }
  return itNode->second;
}

std::vector<Edge> DB::getInEdges(std::uint32_t nodeID)
{
  typedef stx::btree_set<Edge, compEdges>::const_iterator UintSetIt;
  std::vector<Edge> result;
  Edge keyLower;
  Edge keyUpper;

  keyLower.target = nodeID;
  keyUpper.target = nodeID;

  keyLower.source = std::numeric_limits<std::uint32_t>::min();
  keyLower.component = std::numeric_limits<std::uint32_t>::min();

  keyUpper.source = std::numeric_limits<std::uint32_t>::max();
  keyUpper.component = std::numeric_limits<std::uint32_t>::max();

  UintSetIt lowerBound = edges.lower_bound(keyLower);
  UintSetIt upperBound = edges.upper_bound(keyUpper);

  for(UintSetIt it=lowerBound; it != upperBound; it++)
  {
    result.push_back(*it);
  }

  return result;
}

std::vector<Edge> DB::getEdgesBetweenNodes(std::uint32_t sourceID, std::uint32_t targetID)
{
  typedef stx::btree_set<Edge, compEdges>::const_iterator UintSetIt;
  std::vector<Edge> result;
  Edge keyLower;
  Edge keyUpper;

  keyLower.source = sourceID;
  keyUpper.source = sourceID;
  keyLower.target = targetID;
  keyUpper.target = targetID;

  keyLower.component = std::numeric_limits<std::uint32_t>::min();
  keyUpper.component = std::numeric_limits<std::uint32_t>::max();

  UintSetIt lowerBound = edges.lower_bound(keyLower);
  UintSetIt upperBound = edges.upper_bound(keyUpper);

  for(UintSetIt it=lowerBound; it != upperBound; it++)
  {
    result.push_back(*it);
  }

  return result;
}

std::vector<Annotation> DB::getNodeAnnotationsByID(const std::uint32_t& id)
{
  typedef stx::btree_multimap<std::uint32_t, Annotation>::const_iterator AnnoIt;

  std::vector<Annotation> result;

  std::pair<AnnoIt,AnnoIt> itRange = nodeAnnotations.equal_range(id);

  for(AnnoIt itAnnos = itRange.first;
      itAnnos != itRange.second; itAnnos++)
  {
    result.push_back(itAnnos->second);
  }

  return result;
}

std::vector<Annotation> DB::getEdgeAnnotations(const Edge &edge)
{
  typedef stx::btree_multimap<Edge, Annotation, compEdges>::const_iterator AnnoIt;

  std::vector<Annotation> result;

  std::pair<AnnoIt,AnnoIt> itRange = edgeAnnotations.equal_range(edge);

  for(AnnoIt itAnnos = itRange.first;
      itAnnos != itRange.second; itAnnos++)
  {
    result.push_back(itAnnos->second);
  }

  return result;

}
