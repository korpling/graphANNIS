#include "fallbackedgedb.h"

#include <fstream>

using namespace annis;
using namespace std;

FallbackEdgeDB::FallbackEdgeDB(const Component &component)
  : component(component)
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
