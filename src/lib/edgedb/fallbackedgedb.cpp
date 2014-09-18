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
  edges.insert2(edge.source, edge.target);
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

bool FallbackEdgeDB::isConnected(const Edge &edge, unsigned int distance) const
{
  if(distance == 0)
  {
    return false;
  }
  else if(distance == 1)
  {
    stx::btree_map<uint32_t, uint32_t>::const_iterator it
        = edges.find(edge.source);
    if(it == edges.end())
    {
      return false;
    }
    else
    {
      return it->second == edge.target;
    }
  }
  else
  {
    throw("Not implemented yet");
  }
}

AnnotationIterator *FallbackEdgeDB::findConnected(const StringStorage& strings,
                                                 std::uint32_t sourceNode,
                                                 unsigned int minDistance,
                                                 unsigned int maxDistance) const
{
  return new FallbackReachableIterator(strings, *this, sourceNode, minDistance, maxDistance);
}

std::vector<Annotation> FallbackEdgeDB::getEdgeAnnotations(const Edge& edge)
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

FallbackReachableIterator::FallbackReachableIterator(
                                                     const StringStorage& strings,
                                                     const FallbackEdgeDB &edb,
                                                     std::uint32_t startNode,
                                                     unsigned int minDistance,
                                                     unsigned int maxDistance)
  : edb(edb), minDistance(minDistance), maxDistance(maxDistance)
{
  nodeNameID = strings.findID("node_name").second;
  nodeNamespaceID = strings.findID(annis_ns).second;
  emptyValID = strings.findID("annis_ns").second;

  EdgeIt it = edb.edges.find(startNode);
  if(it != edb.edges.end())
  {
    traversalStack.push(it);
  }
}

bool FallbackReachableIterator::hasNext()
{
  return !traversalStack.empty();
}

Match FallbackReachableIterator::next()
{
  Match result;
  if(!traversalStack.empty())
  {
    EdgeIt it = traversalStack.top();
    traversalStack.pop();


    // get the next node
    result.first = it->second;
    result.second.name = nodeNameID;
    result.second.ns = nodeNamespaceID;
    result.second.val = emptyValID; // TODO: do we need to catch the real value here?

    // update iterator and add it to the stack again if there are more siblings
    it++;
    if(it != edb.edges.end())
    {
      traversalStack.push(it);
    }


  }
  return result;
}
