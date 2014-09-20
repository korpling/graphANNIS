#include "linearedgedb.h"
#include <fstream>
#include <set>
#include <limits>

using namespace annis;
using namespace std;

LinearEdgeDB::LinearEdgeDB(StringStorage& strings, const Component& component)
  : FallbackEdgeDB(strings, component)
{
}

void LinearEdgeDB::clear()
{
  FallbackEdgeDB::clear();
  node2pos.clear();
  pos2node.clear();
}

void LinearEdgeDB::calculateIndex()
{
  typedef stx::btree_set<Edge, compEdges>::const_iterator EdgeIt;
  // find all root nodes
  set<nodeid_t> roots;

  // add all nodes to root list
  for(EdgeIt it=getEdgesBegin();
      it != getEdgesEnd(); it++)

  {
    roots.insert((*it).source);
  }
  // remove the ones that have an ingoing edge
  for(EdgeIt edgeIt=getEdgesBegin();
      edgeIt != getEdgesEnd(); edgeIt++)

  {
    set<nodeid_t>::const_iterator rootIt = roots.find((*edgeIt).target);
    if(rootIt != roots.end())
    {
      roots.erase(rootIt);
    }
  }

  for(auto& rootNode : roots)
  {
    // iterate over all edges beginning from the root
    pos2node[initRelativePosition(rootNode,0)] = rootNode;
    node2pos[rootNode] = initRelativePosition(rootNode,0);

    FallbackDFSIterator it(*this, rootNode, 1, numeric_limits<uint32_t>::max());

    uint32_t pos=1;
    for(pair<bool, nodeid_t> node = it.next(); node.first; node = it.next(), pos++)
    {
      pos2node[initRelativePosition(rootNode,pos)] = node.second;
      node2pos[node.second] = initRelativePosition(rootNode,pos);
    }
  }


}

bool LinearEdgeDB::isConnected(const Edge &edge, unsigned int minDistance, unsigned int maxDistance) const
{
  typedef stx::btree_map<nodeid_t, RelativePosition>::const_iterator PosIt;

  PosIt posSourceIt = node2pos.find(edge.source);
  PosIt posTargetIt = node2pos.find(edge.target);
  if(posSourceIt != node2pos.end() && posTargetIt != node2pos.end())
  {
    RelativePosition posSource = posSourceIt->second;
    RelativePosition posTarget = posTargetIt->second;
    if(posSource.node == posTarget.node && posSource.pos < posTarget.pos)
    {
      unsigned int diff = posTarget.pos - posSource.pos;
      return diff >= minDistance && diff <= maxDistance;
    }
  }
  return false;
}

EdgeIterator *LinearEdgeDB::findConnected(nodeid_t sourceNode, unsigned int minDistance, unsigned int maxDistance) const
{
  return new LinearIterator(*this, sourceNode, minDistance, maxDistance);
}


bool LinearEdgeDB::save(string dirPath)
{
  bool result = FallbackEdgeDB::save(dirPath);

  ofstream out;

  out.open(dirPath + "/node2pos.btree");
  node2pos.dump(out);
  out.close();

  out.open(dirPath + "/pos2node.btree");
  pos2node.dump(out);
  out.close();

  return result;
}

bool LinearEdgeDB::load(string dirPath)
{
  bool result = FallbackEdgeDB::load(dirPath);
  ifstream in;


  in.open(dirPath + "/node2pos.btree");
  result = result && node2pos.restore(in);
  in.close();

  in.open(dirPath + "/pos2node.btree");
  result = result && pos2node.restore(in);
  in.close();

  return result;
}

LinearEdgeDB::~LinearEdgeDB()
{

}

LinearIterator::LinearIterator(const LinearEdgeDB &edb, std::uint32_t startNode,
                               unsigned int minDistance, unsigned int maxDistance)
  : edb(edb)
{
  typedef stx::btree_map<nodeid_t, RelativePosition>::const_iterator PosIt;
  PosIt posSourceIt = edb.node2pos.find(startNode);
  if(posSourceIt != edb.node2pos.end())
  {
    currentPos = posSourceIt->second;
    // define where to stop
    endPos = currentPos.pos + maxDistance;
    // add the minium distance
    currentPos.pos = currentPos.pos + minDistance;

  }
}

pair<bool, nodeid_t> LinearIterator::next()
{
  typedef stx::btree_map<RelativePosition, nodeid_t, compRelativePosition>::const_iterator NodeIt;
  bool found = false;
  nodeid_t node;
  if(currentPos.pos <= endPos)
  {
    NodeIt nextIt = edb.pos2node.find(currentPos);
    if(nextIt != edb.pos2node.end())
    {
      found = true;
      node = nextIt->second;
      currentPos.pos++;
    }
  }
  return std::pair<bool, nodeid_t>(found, node);
}
