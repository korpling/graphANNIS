#include "coverageedb.h"

#include <fstream>
#include <set>

using namespace annis;

CoverageEdgeDB::CoverageEdgeDB(StringStorage &strings, const Component &component)
  : FallbackEdgeDB(strings, component)
{
}

void CoverageEdgeDB::calculateIndex()
{
  typedef stx::btree_set<Edge, compEdges>::const_iterator EdgeIt;
  for(EdgeIt it=getEdgesBegin();
      it != getEdgesEnd(); it++)

  {
    const Edge& e = *it;
    coveringNodes.insert2(e.target, e.source);
  }
}

bool CoverageEdgeDB::save(std::string dirPath)
{
  bool result = FallbackEdgeDB::save(dirPath);

  std::ofstream out;

  out.open(dirPath + "/coveringNodes.btree");
  coveringNodes.dump(out);
  out.close();


  return result;
}

bool CoverageEdgeDB::load(std::string dirPath)
{
  bool result = FallbackEdgeDB::load(dirPath);
  std::ifstream in;

  in.open(dirPath + "/coveringNodes.btree");
  result = result && coveringNodes.restore(in);
  in.close();

//  for(stx::btree_multimap<nodeid_t, nodeid_t>::const_iterator it=coveringNodes.begin();
//      it != coveringNodes.end(); it++)
//  {
//    std::cout << "covering: " <<  it->first << "->" << it->second << std::endl;
//  }

  return result;
}

std::vector<nodeid_t> CoverageEdgeDB::getIncomingEdges(nodeid_t node) const
{
  typedef stx::btree_multimap<nodeid_t, nodeid_t>::const_iterator It;

  std::vector<nodeid_t> result;
  result.reserve(20);

  std::pair<It, It> itRange = coveringNodes.equal_range(node);
  for(It it=itRange.first; it != itRange.second; it++)
  {
    result.push_back(it->second);
  }

  return result;
}

int CoverageEdgeDB::distance(const Edge &edge) const
{
  // coverage components only have paths of length 1
  if(FallbackEdgeDB::isConnected(edge, 1, 1))
  {
    return 1;
  }

  // not connected at all
  return -1;
}

bool CoverageEdgeDB::isConnected(const Edge &edge, unsigned int /*minDistance*/, unsigned int /*maxDistance*/) const
{
  // coverage components only have paths of length 1
  if(FallbackEdgeDB::isConnected(edge, 1, 1))
  {
    return true;
  }

  // not connected at all
  return false;
}


CoverageEdgeDB::~CoverageEdgeDB()
{

}
