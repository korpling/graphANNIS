#include "fallbackedgedb.h"

#include <fstream>
#include <limits>

using namespace annis;
using namespace std;

FallbackEdgeDB::FallbackEdgeDB(StringStorage &strings, const Component &component)
  : strings(strings), component(component)
{
}

void FallbackEdgeDB::addEdge(const Edge &edge)
{
  if(edge.source != edge.target)
  {
    edges.insert(edge);
  }
}

void FallbackEdgeDB::addEdgeAnnotation(const Edge& edge, const Annotation &anno)
{
  edgeAnnotations.insert2(edge, anno);
}

void FallbackEdgeDB::clear()
{
  edges.clear();
  edgeAnnotations.clear();
}

bool FallbackEdgeDB::isConnected(const Edge &edge, unsigned int minDistance, unsigned int maxDistance) const
{
  typedef stx::btree_set<Edge>::const_iterator EdgeIt;
  if(minDistance == 1 && maxDistance == 1)
  {
    EdgeIt it = edges.find(edge);
    if(it != edges.end())
    {
      return true;
    }
    else
    {
      return false;
    }
  }
  else
  {
    CycleSafeDFS dfs(*this, edge.source, minDistance, maxDistance);
    DFSIteratorResult result = dfs.nextDFS();
    while(result.found)
    {
      if(result.node == edge.target)
      {
        return true;
      }
      result = dfs.nextDFS();
    }
  }

  return false;
}

std::unique_ptr<EdgeIterator> FallbackEdgeDB::findConnected(nodeid_t sourceNode,
                                                 unsigned int minDistance,
                                                 unsigned int maxDistance) const
{
  return std::unique_ptr<EdgeIterator>(
        new CycleSafeDFS(*this, sourceNode, minDistance, maxDistance));
}

int FallbackEdgeDB::distance(const Edge &edge) const
{
  CycleSafeDFS dfs(*this, edge.source, 0, uintmax);
  DFSIteratorResult result = dfs.nextDFS();
  while(result.found)
  {
    if(result.node == edge.target)
    {
      return result.distance;
    }
    result = dfs.nextDFS();
  }
  return -1;
}

std::vector<Annotation> FallbackEdgeDB::getEdgeAnnotations(const Edge& edge) const
{
  typedef stx::btree_multimap<Edge, Annotation>::const_iterator ItType;

  std::vector<Annotation> result;

  std::pair<ItType, ItType> range =
      edgeAnnotations.equal_range(edge);

  for(ItType it=range.first; it != range.second; ++it)
  {
    result.push_back(it->second);
  }

  return result;
}

std::vector<nodeid_t> FallbackEdgeDB::getOutgoingEdges(nodeid_t node) const
{
  typedef stx::btree_set<Edge>::const_iterator EdgeIt;

  vector<nodeid_t> result;

  EdgeIt lowerIt = edges.lower_bound(Init::initEdge(node, numeric_limits<uint32_t>::min()));
  EdgeIt upperIt = edges.upper_bound(Init::initEdge(node, numeric_limits<uint32_t>::max()));

  for(EdgeIt it = lowerIt; it != upperIt; it++)
  {
    result.push_back(it->target);
  }

  return result;
}

std::vector<nodeid_t> FallbackEdgeDB::getIncomingEdges(nodeid_t node) const
{
  // this is a extremly slow approach, there should be more efficient methods for other
  // edge databases
  // TODO: we should also concider to add another index

  std::vector<nodeid_t> result;
  result.reserve(10);
  for(auto& e : edges)
  {
    if(e.target == node)
    {
      result.push_back(e.source);
    }
  }
  return result;
}

bool FallbackEdgeDB::load(std::string dirPath)
{
  clear();

  ifstream in;

  in.open(dirPath + "/edges.btree");
  edges.restore(in);
  in.close();

  in.open(dirPath + "/edgeAnnotations.btree");
  edgeAnnotations.restore(in);
  in.close();

  return true;

}

bool FallbackEdgeDB::save(std::string dirPath)
{
  ofstream out;

  out.open(dirPath + "/edges.btree");
  edges.dump(out);
  out.close();

  out.open(dirPath + "/edgeAnnotations.btree");
  edgeAnnotations.dump(out);
  out.close();

  return true;
}

std::uint32_t FallbackEdgeDB::numberOfEdges() const
{
  return edges.size();
}

std::uint32_t FallbackEdgeDB::numberOfEdgeAnnotations() const
{
  return edgeAnnotations.size();
}

FallbackDFSIterator::FallbackDFSIterator(const FallbackEdgeDB &edb,
                                                     std::uint32_t startNode,
                                                     unsigned int minDistance,
                                                     unsigned int maxDistance)
  : edb(edb), minDistance(minDistance), maxDistance(maxDistance), startNode(startNode)
{
  initStack();
}

DFSIteratorResult FallbackDFSIterator::nextDFS()
{
  DFSIteratorResult result;
  result.found = false;

  while(!result.found && !traversalStack.empty())
  {
    pair<uint32_t, unsigned int> stackEntry = traversalStack.top();
    result.node = stackEntry.first;
    result.distance = stackEntry.second;


    // we are entering a new node
    if(beforeEnterNode(result.node, result.distance))
    {
      result.found = enterNode(result.node, result.distance);
    }
    else
    {
      traversalStack.pop();
    }
  }
  return result;
}

bool FallbackDFSIterator::enterNode(nodeid_t node, unsigned int distance)
{
  bool found = false;

  traversalStack.pop();

  if(distance >= minDistance && distance <= maxDistance)
  {
    // get the next node
    found = true;
  }

  // add the remaining child nodes
  if(distance < maxDistance)
  {
    // add the outgoing edges to the stack
    auto outgoing = edb.getOutgoingEdges(node);
    for(const auto& outNodeID : outgoing)
    {

      traversalStack.push(pair<nodeid_t, unsigned int>(outNodeID, distance+1));
    }
  }
  return found;
}


std::pair<bool, nodeid_t> FallbackDFSIterator::next()
{
  DFSIteratorResult result = nextDFS();
  return std::pair<bool, nodeid_t>(result.found, result.node);
}

void FallbackDFSIterator::initStack()
{
  // add the initial value to the stack
  traversalStack.push({startNode, 0});
}

void FallbackDFSIterator::reset()
{
  // clear the stack
  while(!traversalStack.empty())
  {
    traversalStack.pop();
  }

  initStack();
}


CycleSafeDFS::CycleSafeDFS(const FallbackEdgeDB &edb, std::uint32_t startNode, unsigned int minDistance, unsigned int maxDistance)
  : FallbackDFSIterator(edb, startNode, minDistance, maxDistance)
{

}

void CycleSafeDFS::initStack()
{
  FallbackDFSIterator::initStack();

  lastDistance = 0;

  nodesInCurrentPath.insert(startNode);
  distanceToNode.insert({0, startNode});
}

void CycleSafeDFS::reset()
{
  nodesInCurrentPath.clear();
  distanceToNode.clear();

  FallbackDFSIterator::reset();
}

bool CycleSafeDFS::enterNode(nodeid_t node, unsigned int distance)
{
  nodesInCurrentPath.insert(node);
  distanceToNode.insert({distance, node});

  lastDistance = distance;

  return FallbackDFSIterator::enterNode(node, distance);
}

bool CycleSafeDFS::beforeEnterNode(nodeid_t node, unsigned int distance)
{
  if(lastDistance >= distance)
  {
    // A subgraph was completed.
    // Remove all nodes from the path set that are below the parent node:
    for(auto it=distanceToNode.find(distance); it != distanceToNode.end(); it = distanceToNode.erase(it))
    {
      nodesInCurrentPath.erase(it->second);
    }
  }

  if(nodesInCurrentPath.find(node) == nodesInCurrentPath.end())
  {
    return true;
  }
  else
  {
    // we detected a cycle!
    std::cerr << "------------------------------" << std::endl;
    std::cerr << "ERROR: cycle detected when inserting node " << node << std::endl;
    std::cerr << "distanceToNode: ";
    for(auto itPath = distanceToNode.begin(); itPath != distanceToNode.end(); itPath++)
    {
      std::cerr << itPath->first << "->" << itPath->second << " ";
    }
    std::cerr << endl;
    std::cerr << "nodesInCurrentPath: ";
    for(auto itPath = nodesInCurrentPath.begin(); itPath != nodesInCurrentPath.end(); itPath++)
    {
      std::cerr << *itPath << " ";
    }
    std::cerr << endl;
    std::cerr << "------------------------------" << std::endl;

    lastDistance = distance;

    return false;
  }
}

CycleSafeDFS::~CycleSafeDFS()
{

}
