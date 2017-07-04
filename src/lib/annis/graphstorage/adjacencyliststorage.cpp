/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#include "adjacencyliststorage.h"
#include <annis/annosearch/exactannokeysearch.h>  // for ExactAnnoKeySearch
#include <annis/util/dfs.h>                       // for CycleSafeDFS, DFSIt...
#include <annis/util/size_estimator.h>            // for element_size
#include <google/btree.h>                         // for btree_iterator, btr...
#include <google/btree_set.h>                     // for btree_set
#include <stdint.h>                               // for uint32_t, uint64_t
#include <algorithm>                              // for max
#include <list>                                   // for list
#include "annis/annostorage.h"                    // for AnnoStorage
#include "annis/db.h"                             // for DB
#include "annis/graphstorage/graphstorage.h"      // for ReadableGraphStorage
#include "annis/iterators.h"                      // for EdgeIterator
#include "annis/types.h"                          // for Edge, GraphStatistic

namespace annis { class StringStorage;}

using namespace annis;
using namespace std;

AdjacencyListStorage::NodeIt::NodeIt(set_t<Edge>::const_iterator itStart,
                                     set_t<Edge>::const_iterator itEnd)
  : it(itStart), itStart(itStart), itEnd(itEnd)
{
}

bool AdjacencyListStorage::NodeIt::next(Match &m)
{
  while(it != itEnd)
  {
    if(lastNode && *lastNode != it->source)
    {
      m.node = it->source;
      lastNode = it->source;
      return true;
    }

    it++;
  }
  return false;
}

void AdjacencyListStorage::NodeIt::reset()
{
  it = itStart;
  lastNode.reset();
}

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
  edgeAnnos.calculateStatistics(db.strings);

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
   edgeAnnos.addAnnotation(edge, anno);
}

void AdjacencyListStorage::deleteEdge(const Edge &edge)
{
   edges.erase(edge);
   inverseEdges.erase({edge.target, edge.source});
   std::vector<Annotation> annos = edgeAnnos.getAnnotations(edge);
   for(Annotation a : annos)
   {
      AnnotationKey key = {a.name, a.ns};
      edgeAnnos.deleteAnnotation(edge, key);
   }
}

void AdjacencyListStorage::deleteNode(nodeid_t node)
{
  // find all both ingoing and outgoing edges
  std::list<Edge> edgesToDelete;
  for(auto it = edges.lower_bound({node, 0});
    it != edges.end() && it.key().source == node; it++)
  {
    edgesToDelete.push_back(*it);
  }

  for(auto it = inverseEdges.lower_bound({node, 0});
    it != inverseEdges.end() && it->source == node; it++)
  {
    edgesToDelete.push_back(*it);
  }

  // delete the found edges
  for(const Edge& e : edgesToDelete)
  {
     deleteEdge(e);
  }
}

void AdjacencyListStorage::deleteEdgeAnnotation(const Edge &edge, const AnnotationKey &anno)
{
  edgeAnnos.deleteAnnotation(edge, anno);
}

void AdjacencyListStorage::clear()
{
  edges.clear();
  inverseEdges.clear();
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
  return edgeAnnos.getAnnotations(edge);
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
  return edgeAnnos.numberOfAnnotations();
}

void AdjacencyListStorage::calculateStatistics(const StringStorage &strings)
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

  // also calculate the annotation statistics
  edgeAnnos.calculateStatistics(strings);

  stat.valid = true;

}

size_t AdjacencyListStorage::estimateMemorySize()
{
  return
      size_estimation::element_size(edges)
      + size_estimation::element_size(inverseEdges)
      + edgeAnnos.estimateMemorySize()
      + sizeof(AdjacencyListStorage);
}
