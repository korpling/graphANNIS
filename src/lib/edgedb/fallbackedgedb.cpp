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
    FallbackDFSIterator dfs(*this, edge.source, minDistance, maxDistance);
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
        new FallbackDFSIterator(*this, sourceNode, minDistance, maxDistance));
}

int FallbackEdgeDB::distance(const Edge &edge) const
{
  FallbackDFSIterator dfs(*this, edge.source, 0, uintmax);
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

    // check if distance was changed to detect the completion of a subgraph
    if(lastDistance >= result.distance)
    {
      // remove all nodes from the path set that are below the parent node
      for(auto it=distanceToNode.find(result.distance); it != distanceToNode.end(); it = distanceToNode.erase(it))
      {
        nodesInCurrentPath.erase(it->second);
      }
    }

    lastDistance = result.distance;
    traversalStack.pop();

    if(result.distance >= minDistance && result.distance <= maxDistance)
    {
      // get the next node
      result.found = true;
    }

    // add the remaining child nodes
    if(result.distance < maxDistance)
    {
      // add the outgoing edges to the stack
      auto outgoing = edb.getOutgoingEdges(result.node);
      for(const auto& outNodeID : outgoing)
      {
        if(nodesInCurrentPath.find(outNodeID) == nodesInCurrentPath.end())
        {
          traversalStack.push(pair<nodeid_t, unsigned int>(outNodeID,
                                                           result.distance+1));
          nodesInCurrentPath.insert(outNodeID);
          distanceToNode.insert({result.distance+1, outNodeID});
        }
        else
        {
          // we detected a cycle!
          std::cerr << "ERROR: cycle detected" << std::endl;
        }
      }
    }
  }
  return result;
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
  lastDistance = 0;
  nodesInCurrentPath.insert(startNode);
  distanceToNode.insert({0, startNode});
}

void FallbackDFSIterator::reset()
{
  // clear the stack
  while(!traversalStack.empty())
  {
    traversalStack.pop();
  }
  nodesInCurrentPath.clear();
  distanceToNode.clear();

  initStack();
}
