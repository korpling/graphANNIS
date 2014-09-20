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
  typedef stx::btree_set<Edge, compEdges>::const_iterator EdgeIt;
  if(minDistance == 0 && maxDistance == 0)
  {
    return false;
  }
  else if(minDistance == 1 && maxDistance == 1)
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

EdgeIterator *FallbackEdgeDB::findConnected(nodeid_t sourceNode,
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

std::vector<nodeid_t> FallbackEdgeDB::getOutgoingEdges(nodeid_t node) const
{
  typedef stx::btree_set<Edge, compEdges>::const_iterator EdgeIt;

  vector<nodeid_t> result;

  EdgeIt lowerIt = edges.lower_bound(initEdge(node, numeric_limits<uint32_t>::min()));
  EdgeIt upperIt = edges.lower_bound(initEdge(node, numeric_limits<uint32_t>::max()));

  for(EdgeIt it = lowerIt; it != upperIt; it++)
  {
    result.push_back(it->target);
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

std::pair<bool, nodeid_t> FallbackDFSIterator::next()
{
  bool found = false;
  nodeid_t node;
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
      // add the outgoing edges to the stack
      std::vector<uint32_t> outgoing = edb.getOutgoingEdges(node);
      for(size_t idxOutgoing=0; idxOutgoing < outgoing.size(); idxOutgoing++)
      {
        traversalStack.push(pair<uint32_t, unsigned int>(outgoing[idxOutgoing], distance+1));
      }
    }
  }
  return std::pair<bool, nodeid_t>(found, node);
}
