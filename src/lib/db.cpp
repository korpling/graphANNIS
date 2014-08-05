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

bool DB::loadNodeStorage(std::string dirPath)
{
  nodes.clear();
  nodeAnnotations.clear();

  std::string nodeTabPath = dirPath + "/node.tab";
  std::cout << "loading " << nodeTabPath << std::endl;

  std::ifstream in;
  in.open(nodeTabPath, std::ifstream::in);

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
}

Node DB::getNodeByID(std::uint32_t id)
{
  stx::btree_map<std::uint32_t, Node>::const_iterator itNode = nodes.find(id);
  if(itNode == nodes.end())
  {
    // TODO: don't use exception here
    throw("Unknown node");
  }
  return itNode.data();
}
