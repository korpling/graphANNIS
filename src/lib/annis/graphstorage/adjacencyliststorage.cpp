#include <annis/graphstorage/adjacencyliststorage.h>

#include <annis/util/size_estimator.h>

#include <annis/util/dfs.h>
#include <annis/annosearch/exactannokeysearch.h>

#include <fstream>
#include <limits>

#include <google/btree_set.h>

using namespace annis;
using namespace std;


void AdjacencyListStorage::copy(const DB &db, const ReadableGraphStorage &orig)
{
  clear();

  ExactAnnoKeySearch nodes(db, annis_ns, annis_node_name);
  Match match;
  while(nodes.next(match))
  {
    nodeid_t source = match.node;
    std::vector<nodeid_t> outEdges = orig.getOutgoingEdges(source);
    for(auto target : outEdges)
    {
      Edge e = {source, target};
      addEdge(e);
      std::vector<Annotation> annos = orig.getEdgeAnnotations(e);
      for(auto a : annos)
      {
        addEdgeAnnotation(e, a);
      }
    }
  }

  stat = orig.getStatistics();

  calculateIndex();
}

void AdjacencyListStorage::addEdge(const Edge &edge)
{
  if(edge.source != edge.target)
  {
    edges.insert(edge);
    stat.valid = false;
  }
}

void AdjacencyListStorage::addEdgeAnnotation(const Edge& edge, const Annotation &anno)
{
  edgeAnnos.addEdgeAnnotation(edge, anno);
}

void AdjacencyListStorage::clear()
{
  edges.clear();
  edgeAnnos.clear();

  stat.valid = false;
}

bool AdjacencyListStorage::isConnected(const Edge &edge, unsigned int minDistance, unsigned int maxDistance) const
{
  typedef set_t<Edge>::const_iterator EdgeIt;
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

std::unique_ptr<EdgeIterator> AdjacencyListStorage::findConnected(nodeid_t sourceNode,
                                                 unsigned int minDistance,
                                                 unsigned int maxDistance) const
{
  return std::unique_ptr<EdgeIterator>(
        new UniqueDFS(*this, sourceNode, minDistance, maxDistance));
}

int AdjacencyListStorage::distance(const Edge &edge) const
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

std::vector<Annotation> AdjacencyListStorage::getEdgeAnnotations(const Edge& edge) const
{
  return edgeAnnos.getEdgeAnnotations(edge);
}

std::vector<nodeid_t> AdjacencyListStorage::getOutgoingEdges(nodeid_t node) const
{
  typedef set_t<Edge>::const_iterator EdgeIt;

  vector<nodeid_t> result;

  for(EdgeIt it = edges.lower_bound({node, 0}); 
    it != edges.end() && it.key().source == node; it++)
  {
    result.push_back(it.key().target);
  }

  return result;
}


size_t AdjacencyListStorage::numberOfEdges() const
{
  return edges.size();
}

size_t AdjacencyListStorage::numberOfEdgeAnnotations() const
{
  return edgeAnnos.numberOfEdgeAnnotations();
}

void AdjacencyListStorage::calculateStatistics()
{
  stat.valid = false;
  stat.maxFanOut = 0;
  stat.maxDepth = 1;
  stat.avgFanOut = 0.0;
  stat.cyclic = false;
  stat.rootedTree = true;
  stat.nodes = 0;

  unsigned int sumFanOut = 0;


  btree::btree_set<nodeid_t> hasIncomingEdge;

  // find all root nodes
  btree::btree_set<nodeid_t> roots;
  btree::btree_set<nodeid_t> allNodes;
  for(const auto& e : edges)
  {
    roots.insert(e.source);
    allNodes.insert(e.source);
    allNodes.insert(e.target);

    if(stat.rootedTree)
    {
      auto findTarget = hasIncomingEdge.find(e.target);
      if(findTarget == hasIncomingEdge.end())
      {
        hasIncomingEdge.insert(e.target);
      }
      else
      {
        stat.rootedTree = false;
      }
    }
  }

  stat.nodes = static_cast<uint32_t>(allNodes.size());
  allNodes.clear();

  auto itFirstEdge = edges.begin();
  if(itFirstEdge != edges.end())
  {
    nodeid_t lastSourceID = itFirstEdge->source;
    uint32_t currentFanout = 0;

    for(const auto& e : edges)
    {
      roots.erase(e.target);

      if(lastSourceID != e.source)
      {

        stat.maxFanOut = std::max(stat.maxFanOut, currentFanout);
        sumFanOut += currentFanout;

        currentFanout = 0;
        lastSourceID = e.source;
      }
      currentFanout++;
    }
    // add the statistics for the last node
    stat.maxFanOut = std::max(stat.maxFanOut, currentFanout);
    sumFanOut += currentFanout;
  }


  std::uint64_t numberOfVisits = 0;
  if(roots.empty() && !edges.empty())
  {
    // if we have edges but no roots at all there must be a cycle
    stat.cyclic = true;
  }
  else
  {
    for(const auto& rootNode : roots)
    {
      CycleSafeDFS dfs(*this, rootNode, 0, uintmax, false);
      for(auto n = dfs.nextDFS(); n.found; n = dfs.nextDFS())
      {
        numberOfVisits++;


        stat.maxDepth = std::max(stat.maxDepth, n.distance);
      }
      if(dfs.cyclic())
      {
        stat.cyclic = true;
      }
    }
  }

  if(stat.cyclic)
  {
    stat.rootedTree = false;
    // it's infinite
    stat.maxDepth = 0;
    stat.dfsVisitRatio = 0.0;
  }
  else
  {
    if(stat.nodes > 0)
    {
      stat.dfsVisitRatio = (double) numberOfVisits / (double) stat.nodes;
    }
  }

  if(sumFanOut > 0 && stat.nodes > 0)
  {
    stat.avgFanOut =  (double) sumFanOut / (double) stat.nodes;
  }

  stat.valid = true;

}

size_t AdjacencyListStorage::estimateMemorySize()
{
  return
      size_estimation::element_size(edges)
      + edgeAnnos.estimateMemorySize()
      + sizeof(AdjacencyListStorage);
}
