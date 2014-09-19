#include "fallbackedgedb.h"

#include <fstream>

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
    edges.insert2(edge.source, edge.target);
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

const Component &FallbackEdgeDB::getComponent()
{
  return component;
}

bool FallbackEdgeDB::isConnected(const Edge &edge, unsigned int minDistance, unsigned int maxDistance) const
{
  typedef stx::btree_multimap<uint32_t, uint32_t>::const_iterator EdgeIt;
  if(minDistance == 0 && maxDistance == 0)
  {
    return false;
  }
  else if(minDistance == 1 && maxDistance == 1)
  {
    pair<EdgeIt, EdgeIt> range = edges.equal_range(edge.source);
    for(EdgeIt it = range.first; it != range.second; it++)
    {
      if(it->second == edge.target)
      {
        return true;
      }
    }
    return false;
  }
  else
  {
    FallbackDFSIterator dfs(*this, edge.source, minDistance, maxDistance);
    std::pair<bool, std::uint32_t> result = dfs.next();
    while(result.first)
    {
      if(result.second == edge.target)
      {
        return true;
      }
      result = dfs.next();
    }
  }

  return false;
}

EdgeIterator *FallbackEdgeDB::findConnected(std::uint32_t sourceNode,
                                                 unsigned int minDistance,
                                                 unsigned int maxDistance) const
{
  return new FallbackDFSIterator(*this, sourceNode, minDistance, maxDistance);
}

std::vector<Annotation> FallbackEdgeDB::getEdgeAnnotations(const Edge& edge) const
{
  typedef stx::btree_multimap<Edge, Annotation, compEdges>::const_iterator ItType;

  std::vector<Annotation> result;

  std::pair<ItType, ItType> range =
      edgeAnnotations.equal_range(edge);

  for(ItType it=range.first; it != range.second; ++it)
  {
    result.push_back(it->second);
  }

  return result;
}

std::vector<std::uint32_t> FallbackEdgeDB::getOutgoingEdges(std::uint32_t sourceNode) const
{
  typedef stx::btree_multimap<uint32_t, uint32_t>::const_iterator EdgeIt;

  vector<uint32_t> result;
  pair<EdgeIt, EdgeIt> range = edges.equal_range(sourceNode);
  for(EdgeIt it = range.first; it != range.second; it++)
  {
    result.push_back(it->second);
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
  : edb(edb), minDistance(minDistance), maxDistance(maxDistance)
{
  traversalStack.push(pair<uint32_t,unsigned int>(startNode, 0));
}

std::pair<bool, std::uint32_t> FallbackDFSIterator::next()
{
  bool found = false;
  uint32_t node;
  while(!found && !traversalStack.empty())
  {
    pair<uint32_t, unsigned int> stackEntry = traversalStack.top();
    node = stackEntry.first;
    unsigned int distance = stackEntry.second;
    traversalStack.pop();

    if(distance >= minDistance && distance <= maxDistance)
    {
      // get the next node
      found = true;
    }

    // add the remaining child nodes
    if(distance < maxDistance)
    {
      // add the edges to the stack
      pair<EdgeIt, EdgeIt> children = edb.edges.equal_range(node);
      for(EdgeIt it=children.first; it != children.second; it++)
      {
        traversalStack.push(pair<uint32_t, unsigned int>(it->second, distance+1));
      }
    }
  }
  return std::pair<bool, uint32_t>(found, node);
}
