#include "db.h"

#include <iostream>
#include <fstream>
#include <sstream>

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
  std::cout << "loading " << nodeTabPath << std::endl;

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
  std::cout << "loading " << nodeAnnoTabPath << std::endl;

  in.open(nodeAnnoTabPath, std::ifstream::in);
  if(!in.good()) return false;

  while((line = nextCSV(in)).size() > 0)
  {
    u_int32_t nodeNr = uint32FromString(line[0]);
    NodeAnnotation anno;
    anno.nodeId = nodeNr;
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

  std::ifstream in;
  std::string rankTabPath = dirPath + "/rank.tab";
  std::cout << "loading " << rankTabPath << std::endl;

  in.open(rankTabPath, std::ifstream::in);
  if(!in.good()) return false;

  std::vector<std::string> line;

  // first run: collect all pre-order values for a node
  stx::btree_map<std::uint32_t, std::uint32_t> pre2NodeID;
  while((line = nextCSV(in)).size() > 0)
  {
    pre2NodeID.insert2(uint32FromString(line[1]),uint32FromString(line[2]));
  }

  int test = pre2NodeID.size();

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
      e.target = uint32FromString(line[2]);
      e.source = it->second;
      e.component = uint32FromString(line[3]);

      // since we ignore the pre-order value
      // we might add an edge several times if it has several
      // rank entries
      edges.insert(e);
    }
  }

  in.close();
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

std::vector<NodeAnnotation> DB::getNodeAnnotationsByID(std::uint32_t id)
{
  typedef stx::btree_multimap<std::uint32_t, NodeAnnotation>::const_iterator AnnoIt;

  std::vector<NodeAnnotation> result;

  std::pair<AnnoIt,AnnoIt> itRange = nodeAnnotations.equal_range(id);

  for(AnnoIt itAnnos = itRange.first;
      itAnnos != itRange.second; itAnnos++)
  {
    result.push_back(itAnnos->second);
  }

  return result;
}
