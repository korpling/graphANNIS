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

#include "abstractedgeoperator.h"

#include <annis/util/comparefunctions.h>            // for checkAnnotationEqual
#include <annis/wrapper.h>                          // for ListWrapper
#include <google/btree.h>                           // for btree_iterator
#include <google/btree_map.h>                       // for btree_map
#include <google/btree_set.h>                       // for btree_set
#include <stddef.h>                                 // for size_t
#include <algorithm>                                // for max, min, move
#include <cmath>                                    // for ceil
#include <limits>                                   // for numeric_limits
#include <set>                                      // for set
#include <utility>                                  // for pair
#include "annis/annosearch/nodebyedgeannosearch.h"  // for NodeByEdgeAnnoSearch
#include "annis/annostorage.h"                      // for AnnoStorage
#include "annis/graphstorage/graphstorage.h"        // for ReadableGraphStorage
#include "annis/iterators.h"                        // for EdgeIterator, AnnoIt
#include "annis/stringstorage.h"                    // for StringStorage
#include <annis/types.h>

#include <annis/annosearch/exactannokeysearch.h>

using namespace annis;

AbstractEdgeOperator::AbstractEdgeOperator(ComponentType componentType, std::string ns, std::string name,
                     DB::GetGSFuncT getGraphStorageFunc,
                     const DB& db,
                     unsigned int minDistance, unsigned int maxDistance)
  : componentType(componentType),
    getGraphStorageFunc(getGraphStorageFunc),
    db(db),
    strings(db.strings), ns(ns), name(name),
    minDistance(minDistance), maxDistance(maxDistance),
    anyAnno(Init::initAnnotation()), edgeAnno(anyAnno)
{
  initGraphStorage();
}

AbstractEdgeOperator::AbstractEdgeOperator(ComponentType componentType, std::string name,
                     DB::GetAllGSFuncT getAllGraphStorageFunc,
                     const DB& db,
                     unsigned int minDistance, unsigned int maxDistance)
  : componentType(componentType),
    getAllGraphStorageFunc(getAllGraphStorageFunc),
    db(db),
    strings(db.strings), ns(""), name(name),
    minDistance(minDistance), maxDistance(maxDistance),
    anyAnno(Init::initAnnotation()), edgeAnno(anyAnno)
{
  initGraphStorage();
}


AbstractEdgeOperator::AbstractEdgeOperator(ComponentType componentType, std::string ns, std::string name,
    DB::GetGSFuncT getGraphStorageFunc,
    const DB& db,
    const Annotation& edgeAnno)
  : componentType(componentType),
    getGraphStorageFunc(getGraphStorageFunc),
    db(db),
    strings(db.strings), ns(ns), name(name),
    minDistance(1), maxDistance(1),
    anyAnno(Init::initAnnotation()), edgeAnno(edgeAnno)
{
  initGraphStorage();
}

AbstractEdgeOperator::AbstractEdgeOperator(ComponentType componentType, std::string name,
    DB::GetAllGSFuncT getAllGraphStorageFunc,
    const DB& db,
    const Annotation& edgeAnno)
  : componentType(componentType),
    getAllGraphStorageFunc(getAllGraphStorageFunc),
    db(db),
    strings(db.strings), ns(""), name(name),
    minDistance(1), maxDistance(1),
    anyAnno(Init::initAnnotation()), edgeAnno(edgeAnno)
{
  initGraphStorage();
}


std::unique_ptr<AnnoIt> AbstractEdgeOperator::retrieveMatches(const Match &lhs)
{
  std::unique_ptr<ListWrapper> w = std::unique_ptr<ListWrapper>(new ListWrapper());


  // add the rhs nodes of all of the edge storages
  if(gs.size() == 1)
  {
     std::unique_ptr<EdgeIterator> it = gs[0]->findConnected(lhs.node, minDistance, maxDistance);
     for(auto m = it->next(); m; m = it->next())
     {
       if(checkEdgeAnnotation(gs[0], lhs.node, *m))
       {
         // directly add the matched node since when having only one component
         // no duplicates are possible
         w->addMatch(*m);
       }
     }
  }
  else if(gs.size() > 1)
  {
    btree::btree_set<nodeid_t> uniqueResult;
    for(auto e : gs)
    {
      std::unique_ptr<EdgeIterator> it = e->findConnected(lhs.node, minDistance, maxDistance);
      for(auto m = it->next(); m; m = it->next())
      {
        if(checkEdgeAnnotation(e, lhs.node, *m))
        {
          uniqueResult.insert(*m);
        }
      }
    }
    for(const auto& n : uniqueResult)
    {
      w->addMatch(n);
    }
  }
  return std::move(w);
}

bool AbstractEdgeOperator::filter(const Match &lhs, const Match &rhs)
{
  // check if the two nodes are connected in *any* of the edge storages
  for(auto e : gs)
  {
    if(e->isConnected(Init::initEdge(lhs.node, rhs.node), minDistance, maxDistance))
    {
      if(checkEdgeAnnotation(e, lhs.node, rhs.node))
      {
        return true;
      }
    }

  }
  return false;
}


void AbstractEdgeOperator::initGraphStorage()
{
  gs.clear();
  if(getAllGraphStorageFunc)
  {
    gs = (*getAllGraphStorageFunc)(componentType, name);
  }
  else if(getGraphStorageFunc)
  {
    // directly add the only known edge storage
    if(auto e = (*getGraphStorageFunc)(componentType, ns, name))
    {
      gs.push_back(e);
    }
  }
}

bool AbstractEdgeOperator::checkEdgeAnnotation(std::shared_ptr<const ReadableGraphStorage> e, nodeid_t source, nodeid_t target)
{
  if(edgeAnno == anyAnno)
  {
    return true;
  }
  else if(edgeAnno.val == 0 || edgeAnno.val == std::numeric_limits<std::uint32_t>::max())
  {
    // must be a valid value
    return false;
  }
  else
  {
    // check if the edge has the correct annotation first
    auto edgeAnnoList = e->getEdgeAnnotations(Init::initEdge(source, target));
    for(const auto& anno : edgeAnnoList)
    {
      if(checkAnnotationEqual(edgeAnno, anno))
      {
        return true;
      }
    } // end for each annotation of candidate edge

  }
  return false;
}

double AbstractEdgeOperator::selectivity() 
{
  if(gs.size() == 0)
  {
    // will not find anything
    return 0.0;
  }

  ExactAnnoKeySearch nodeSearch(db, annis_ns, annis_node_name);
  double maxNodes = nodeSearch.guessMaxCount();

  double worstSel = 0.0;
  
  for(std::weak_ptr<const ReadableGraphStorage> gPtr: gs)
  {
    if(auto g = gPtr.lock())
    {

      double graphStorageSelectivity = 0.0;
      const auto& stat = g->getStatistics();
      if(stat.valid)
      {
        if(stat.cyclic)
        {
          // can get all other nodes
          return 1.0;
        }


        // get number of nodes reachable from min to max distance
        std::uint32_t maxPathLength = std::min(maxDistance, stat.maxDepth);
        std::uint32_t minPathLength = std::max(0, (int) minDistance-1);

        if (stat.avgFanOut > 1.0)
        {
          // Assume two complete k-ary trees (with the average fan-out as k)
          // as defined in "Thomas Cormen: Introduction to algorithms (2009), page 1179)
          // with the maximum and minimum height. Calculate the number of nodes for both complete trees and
          // subtract them to get an estimation of the number of nodes that fullfull the path length criteria.
          double k = stat.avgFanOut;

          double reachableMax = std::ceil((std::pow(k, (double) maxPathLength) - 1.0) / (k - 1.0 ));
          double reachableMin = std::ceil((std::pow(k, (double) minPathLength) - 1.0) / (k - 1.0));

          double reachable =  reachableMax - reachableMin;

          graphStorageSelectivity = reachable  / maxNodes;
        }
        else
        {
          // We can't use the formula for complete k-ary trees because we can't divide by zero and don't want negative
          // numbers. Use the simplified estimation with multiplication instead.
          double reachableMax = std::ceil(stat.avgFanOut * (double) maxPathLength);
          double reachableMin = std::ceil(stat.avgFanOut * (double) minPathLength);

          graphStorageSelectivity =  (reachableMax - reachableMin) /  maxNodes;
        }

      }
      else
      {
         // assume a default selecivity for this graph storage operator
         graphStorageSelectivity = 0.01;
      }

      worstSel = std::max(worstSel, graphStorageSelectivity);
    }
  }
  
  // return worst selectivity
  return worstSel;
}

double AbstractEdgeOperator::edgeAnnoSelectivity()
{
  // check if an edge annotation is defined
  if((edgeAnno == anyAnno))
  {
    return 1.0;
  }
  else
  {
    double worstSel = 0.0;

    for(std::weak_ptr<const ReadableGraphStorage> gPtr: gs)
    {
      if(auto g = gPtr.lock())
      {
        size_t numOfAnnos = g->numberOfEdgeAnnotations();
        if(numOfAnnos == 0)
        {
          // we won't be able to find anything if there are no annotations
          return 0.0;
        }
        else
        {
          // the edge annotation will filter the selectivity even more
          size_t guessedCount = g->getAnnoStorage().guessMaxCount(strings, edgeAnno);

          worstSel = std::max(worstSel, (double) guessedCount /  (double) numOfAnnos);
        }
      }
    }

    return worstSel;
  }
  return -1.0;
}

int64_t AbstractEdgeOperator::guessMaxCountEdgeAnnos()
{
  if(edgeAnno == anyAnno)
  {

    if(gs.size() == 1)
    {
      return gs[0]->getStatistics().nodes;
    }
    else
    {
      // TODO: implement graph storage source node search for multiple graph storages
      return -1;
    }
  }
  else
  {
    std::int64_t sum = 0;
    for(std::weak_ptr<const ReadableGraphStorage> gPtr: gs)
    {
      if(auto g = gPtr.lock())
      {
        sum += g->getAnnoStorage().guessMaxCount(strings, edgeAnno);
      }
    }
    return sum;
  }
}

std::shared_ptr<EstimatedSearch> AbstractEdgeOperator::createAnnoSearch(
    std::function<std::list<Annotation> (nodeid_t)> nodeAnnoMatchGenerator,
    bool maximalOneNodeAnno,
    bool returnsNothing,
    std::int64_t wrappedNodeCountEstimate,
    std::string debugDescription) const
{
  if(edgeAnno == anyAnno)
  {
    if(gs.size() == 1)
    {
      return gs[0]->getSourceNodeIterator(nodeAnnoMatchGenerator, maximalOneNodeAnno, returnsNothing);
    }
    else
    {
      // TODO: implement graph storage source node search for multiple graph storages
      return std::shared_ptr<EstimatedSearch>();
    }
  }
  else
  {
    std::set<Annotation> validEdgeAnnos;
    if(edgeAnno.ns == 0)
    {
      // collect all edge annotations having this name
      for(size_t i =0; i < gs.size(); i++)
      {
        const BTreeMultiAnnoStorage<Edge>& annos = gs[i]->getAnnoStorage();

        auto keysLower = annos.annoKeys.lower_bound({edgeAnno.name, 0});
        auto keysUpper = annos.annoKeys.upper_bound({edgeAnno.name, uintmax});
        for(auto itKey = keysLower; itKey != keysUpper; itKey++)
        {
          Annotation fullyQualifiedAnno = edgeAnno;
          fullyQualifiedAnno.ns = itKey->first.ns;
          validEdgeAnnos.emplace(fullyQualifiedAnno);
        }
      }
    }
    else
    {
      // there is only one valid edge annotation
      validEdgeAnnos.emplace(edgeAnno);
    }
    return std::make_shared<NodeByEdgeAnnoSearch>(gs, validEdgeAnnos, nodeAnnoMatchGenerator,
                                                  maximalOneNodeAnno,
                                                  returnsNothing,
                                                  wrappedNodeCountEstimate, debugDescription);
  }
}


std::string AbstractEdgeOperator::description() 
{
  std::string result;
  if(minDistance == 1 && maxDistance == 1)
  {
    result =  operatorString() + name;
  }
  else if(minDistance == 1 && maxDistance == std::numeric_limits<unsigned int>::max())
  {
    result = operatorString() + name + " *";
  }
  else if(minDistance == maxDistance)
  {
    result = operatorString() + name + "," + std::to_string(minDistance);
  }
  else
  {
    result = operatorString() + name + "," + std::to_string(minDistance) + "," + std::to_string(maxDistance);
  }
  
  if(!(edgeAnno == anyAnno))
  {
    if(edgeAnno.name != 0 && edgeAnno.val != 0
       && edgeAnno.name != std::numeric_limits<std::uint32_t>::max()
       && edgeAnno.val != std::numeric_limits<std::uint32_t>::max())
    {
      result += "[" + strings.str(edgeAnno.name) + "=\"" + strings.str(edgeAnno.val) + "\"]";
    }
    else
    {
      result += "[invalid anno]";
    }
  }
  
  return result;
}


AbstractEdgeOperator::~AbstractEdgeOperator()
{

}

