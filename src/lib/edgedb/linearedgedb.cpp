#include "linearedgedb.h"
#include "../dfs.h"
#include "../annotationsearch.h"

#include <fstream>
#include <set>
#include <limits>

#include <boost/archive/binary_oarchive.hpp>
#include <boost/archive/binary_iarchive.hpp>
#include <boost/serialization/map.hpp>
#include <boost/serialization/string.hpp>
#include <boost/serialization/vector.hpp>

#include <boost/format.hpp>
#include <humblelogging/api.h>

HUMBLE_LOGGER(logger, "annis4");

using namespace annis;
using namespace std;

LinearEdgeDB::LinearEdgeDB(StringStorage& strings, const Component& component)
{
}

void LinearEdgeDB::clear()
{
  edgeAnno.clear();
  node2pos.clear();
  nodeChains.clear();
}

void LinearEdgeDB::copy(const DB& db, const ReadableGraphStorage& orig)
{
  // find all root nodes
  set<nodeid_t> roots;

  // add all nodes to root list
  AnnotationNameSearch nodes(db, annis_ns, annis_node_name);

  while(nodes.hasNext())
  {
    nodeid_t n = nodes.next().node;
    // insert all nodes to the root candidate list which are part of this component
    if(!orig.getOutgoingEdges(n).empty())
    {
      roots.insert(n);
    }
  }

  nodes.reset();
  while(nodes.hasNext())
  {
    nodeid_t source = nodes.next().node;

    std::vector<nodeid_t> outEdges = orig.getOutgoingEdges(source);
    for(auto target : outEdges)
    {
      Edge e = {source, target};

      // remove the nodes that have an incoming edge from the root list
      roots.erase(target);

      std::vector<Annotation> edgeAnnos = orig.getEdgeAnnotations(e);
      for(auto a : edgeAnnos)
      {
        edgeAnno.addEdgeAnnotation(e, a);
      }
    }
  }


  for(auto& rootNode : roots)
  {
    // iterate over all edges beginning from the root
    nodeChains[rootNode] = std::vector<nodeid_t>();
    std::vector<nodeid_t>& chain = nodeChains[rootNode];
    chain.push_back(rootNode);
    node2pos[rootNode] = Init::initRelativePosition(rootNode,chain.size()-1);

    CycleSafeDFS it(*this, rootNode, 1, uintmax);

    uint32_t pos=1;
    for(pair<bool, nodeid_t> node = it.next(); node.first; node = it.next(), pos++)
    {
      chain.push_back(node.second);
      node2pos[node.second] = Init::initRelativePosition(rootNode,chain.size()-1);
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
    if(posSource.root == posTarget.root && posSource.pos <= posTarget.pos)
    {
      int diff = posTarget.pos - posSource.pos;
      if(diff >= 0)
      {
        return ((unsigned int) diff) >= minDistance && ((unsigned int) diff) <= maxDistance;
      }
    }
  }
  return false;
}

std::unique_ptr<EdgeIterator> LinearEdgeDB::findConnected(nodeid_t sourceNode, unsigned int minDistance, unsigned int maxDistance) const
{
  return std::unique_ptr<EdgeIterator>(new LinearIterator(*this, sourceNode, minDistance, maxDistance));
}

int LinearEdgeDB::distance(const Edge &edge) const
{
  typedef stx::btree_map<nodeid_t, RelativePosition>::const_iterator PosIt;

  PosIt posSourceIt = node2pos.find(edge.source);
  PosIt posTargetIt = node2pos.find(edge.target);
  if(posSourceIt != node2pos.end() && posTargetIt != node2pos.end())
  {
    RelativePosition posSource = posSourceIt->second;
    RelativePosition posTarget = posTargetIt->second;
    if(posSource.root == posTarget.root && posSource.pos <= posTarget.pos)
    {
      int diff = posTarget.pos - posSource.pos;
      if(diff >= 0)
      {
        return diff;
      }
    }
  }
  return -1;
}


bool LinearEdgeDB::save(string dirPath)
{
  bool result = edgeAnno.save(dirPath);

  ofstream out;

  out.open(dirPath + "/node2pos.btree");
  node2pos.dump(out);
  out.close();

  out.open(dirPath + "/nodeChains.archive", ios::binary);
  boost::archive::binary_oarchive oa(out);
  oa << nodeChains;
  out.close();

  return result;
}

std::vector<nodeid_t> LinearEdgeDB::getOutgoingEdges(nodeid_t node) const
{
  std::vector<nodeid_t> result;
  auto it = node2pos.find(node);
  if(it != node2pos.end())
  {
    auto pos = it->second;
    auto chainIt = nodeChains.find(pos.root);
    if(chainIt != nodeChains.end())
    {
      const std::vector<nodeid_t>& chain = chainIt->second;
      if(pos.pos < (chain.size()-1))
      {
        result.push_back(chain[pos.pos+1]);
      }
    }
  }
  return result;
}

std::vector<nodeid_t> LinearEdgeDB::getIncomingEdges(nodeid_t node) const
{
  std::vector<nodeid_t> result;
  auto it = node2pos.find(node);
  if(it != node2pos.end())
  {
    auto pos = it->second;
    auto chainIt = nodeChains.find(pos.root);
    if(chainIt != nodeChains.end())
    {
      const std::vector<nodeid_t>& chain = chainIt->second;
      if(pos.pos - 1 > 0 && !chain.empty())
      {
        result.push_back(chain[pos.pos-1]);
      }
    }
  }
  return result;
}

bool LinearEdgeDB::load(string dirPath)
{
  bool result = edgeAnno.save(dirPath);
  ifstream in;


  in.open(dirPath + "/node2pos.btree");
  result = result && node2pos.restore(in);
  in.close();

  in.open(dirPath + "/nodeChains.archive", ios::binary);
  boost::archive::binary_iarchive ia(in);
  ia >> nodeChains;
  in.close();

  return result;
}

LinearEdgeDB::~LinearEdgeDB()
{

}

LinearIterator::LinearIterator(const LinearEdgeDB &edb, nodeid_t startNode,
                               unsigned int minDistance, unsigned int maxDistance)
  : edb(edb), minDistance(minDistance), maxDistance(maxDistance), startNode(startNode),
    chain(nullptr)
{
  reset();
}

pair<bool, nodeid_t> LinearIterator::next()
{
  bool found = false;
  nodeid_t node = 0;
  if(chain != nullptr && currentPos <= endPos && currentPos < chain->size())
  {
    found = true;
    node = chain->at(currentPos);
    chain->at(currentPos);
    currentPos++;
  }
  return std::pair<bool, nodeid_t>(found, node);
}

void LinearIterator::reset()
{
  typedef stx::btree_map<nodeid_t, RelativePosition>::const_iterator PosIt;
  typedef map<nodeid_t, std::vector<nodeid_t> >::const_iterator NodeChainIt;

  PosIt posSourceIt = edb.node2pos.find(startNode);
  if(posSourceIt != edb.node2pos.end())
  {
    const RelativePosition& relPos = posSourceIt->second;
    currentPos = relPos.pos;
    NodeChainIt itNodeChain = edb.nodeChains.find(relPos.root);
    if(itNodeChain != edb.nodeChains.end())
    {
      chain = &(itNodeChain->second);
    }

    // define where to stop
    if(maxDistance == uintmax)
    {
      endPos = uintmax;
    }
    else
    {
      endPos = currentPos + maxDistance;
    }
    // add the minium distance
    currentPos = currentPos + minDistance;

  }
}

LinearIterator::~LinearIterator()
{

}
