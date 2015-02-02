#include "fallbackedgedb.h"

#include "../dfs.h"
#include "../exactannokeysearch.h"

#include <fstream>
#include <limits>

using namespace annis;
using namespace std;

FallbackEdgeDB::FallbackEdgeDB(StringStorage &strings, const Component &component)
  : strings(strings), component(component)
{
}

void FallbackEdgeDB::copy(const DB &db, const ReadableGraphStorage &orig)
{
  clear();

  ExactAnnoKeySearch nodes(db, annis_ns, annis_node_name);
  while(nodes.hasNext())
  {
    nodeid_t source = nodes.next().node;
    std::vector<nodeid_t> outEdges = orig.getOutgoingEdges(source);
    for(auto target : outEdges)
    {
      Edge e = {source, target};
      addEdge(e);
      std::vector<Annotation> edgeAnnos = orig.getEdgeAnnotations(e);
      for(auto a : edgeAnnos)
      {
        addEdgeAnnotation(e, a);
      }
    }
  }

  calculateIndex();
}

void FallbackEdgeDB::addEdge(const Edge &edge)
{
  if(edge.source != edge.target)
  {
    edges.insert(edge);
    statistics.valid = false;
  }
}

void FallbackEdgeDB::addEdgeAnnotation(const Edge& edge, const Annotation &anno)
{
  edgeAnnos.addEdgeAnnotation(edge, anno);
}

void FallbackEdgeDB::clear()
{
  edges.clear();
  edgeAnnos.clear();

  statistics.valid = false;
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
        new UniqueDFS(*this, sourceNode, minDistance, maxDistance));
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
  return edgeAnnos.getEdgeAnnotations(edge);
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

  edgeAnnos.load(dirPath);

  return true;

}

bool FallbackEdgeDB::save(std::string dirPath)
{
  ofstream out;

  out.open(dirPath + "/edges.btree");
  edges.dump(out);
  out.close();

  edgeAnnos.save(dirPath);

  return true;
}

std::uint32_t FallbackEdgeDB::numberOfEdges() const
{
  return edges.size();
}

std::uint32_t FallbackEdgeDB::numberOfEdgeAnnotations() const
{
  return edgeAnnos.numberOfEdgeAnnotations();
}

void FallbackEdgeDB::calculateStatistics()
{
  statistics.valid = false;
  statistics.maxFanOut = 0;
  statistics.maxDepth = 0;
  statistics.avgFanOut = 0.0;
  statistics.cyclic = false;

  double numOfNodes = 0.0;
  double sumFanOut = 0.0;

  nodeid_t lastNodeID = 0;
  uint32_t currentFanout = 0;

  // find all root nodes
  set<nodeid_t> roots;
  for(const auto& e : edges)
  {
    roots.insert(e.source);
  }

  for(const auto& e : edges)
  {
    roots.erase(e.target);

    if(lastNodeID != e.source)
    {
      statistics.maxFanOut = std::max(statistics.maxFanOut, currentFanout);
      sumFanOut += currentFanout;

      numOfNodes++;
      currentFanout = 0;
      lastNodeID = e.source;
    }
    else
    {
      currentFanout++;
    }
  }

  for(const auto& rootNode : roots)
  {
    CycleSafeDFS dfs(*this, rootNode, 0, uintmax, false);
    for(auto n = dfs.nextDFS(); n.found; n = dfs.nextDFS())
    {
      statistics.maxDepth = std::max(statistics.maxDepth, n.distance);
    }
    if(dfs.cyclic())
    {
      statistics.cyclic = true;
    }
  }

  if(sumFanOut > 0 && numOfNodes > 0)
  {
    statistics.avgFanOut = sumFanOut / numOfNodes;
    statistics.valid = true;
  }

}
