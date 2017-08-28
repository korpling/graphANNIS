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

#include "precedence.h"
#include <annis/wrapper.h>                    // for ListWrapper
#include <algorithm>                          // for min, move
#include <utility>                            // for pair
#include "annis/db.h"                         // for DB
#include "annis/graphstorage/graphstorage.h"  // for ReadableGraphStorage
#include "annis/iterators.h"                  // for EdgeIterator, AnnoIt
#include "annis/operators/operator.h"         // for Operator
#include "annis/util/helper.h"                // for TokenHelper


using namespace annis;


Precedence::Precedence(const DB &db, DB::GetGSFuncT getGraphStorageFunc, unsigned int minDistance, unsigned int maxDistance)
  : tokHelper(getGraphStorageFunc, db),
    gsOrder(getGraphStorageFunc(ComponentType::ORDERING, annis_ns, "")),
    gsLeft(getGraphStorageFunc(ComponentType::LEFT_TOKEN, annis_ns, "")),
    anyTokAnno(Init::initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID())),
    anyNodeAnno(Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID())),
    minDistance(minDistance), maxDistance(maxDistance)
{
}

Precedence::Precedence(const DB &db, DB::GetGSFuncT getGraphStorageFunc,
                       std::string segmentation,
                       unsigned int minDistance, unsigned int maxDistance)
  : tokHelper(getGraphStorageFunc, db),
    gsOrder(getGraphStorageFunc(ComponentType::ORDERING, annis_ns, segmentation)),
    gsLeft(getGraphStorageFunc(ComponentType::LEFT_TOKEN, annis_ns, "")),
    anyTokAnno(Init::initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID())),
    anyNodeAnno(Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID())),
    minDistance(minDistance), maxDistance(maxDistance),
    segmentation(segmentation)
{
}

std::unique_ptr<AnnoIt> Precedence::retrieveMatches(const Match &lhs)
{
  std::unique_ptr<ListWrapper> w = std::unique_ptr<ListWrapper>(new ListWrapper());

  if(gsOrder)
  {
    std::unique_ptr<EdgeIterator> edgeIterator;
    nodeid_t startNode;
    if(segmentation)
    {
      startNode = lhs.node;
    }
    else
    {
      startNode = tokHelper.rightTokenForNode(lhs.node);
    }

    edgeIterator = gsOrder->findConnected(startNode,
                                          minDistance, maxDistance);
    // materialize a list of all matches and wrap it
    for(boost::optional<nodeid_t> matchedToken = edgeIterator->next();
        matchedToken; matchedToken = edgeIterator->next())
    {
      // get all nodes that are left-aligned to this token
      for(const auto& n : gsLeft->getOutgoingEdges(*matchedToken))
      {
        w->addMatch(Init::initMatch(anyNodeAnno, n));
      }
      // add the actual token to the list as well
      w->addMatch(Init::initMatch(anyNodeAnno, *matchedToken));
    }
  }

  return std::move(w);
}

bool Precedence::filter(const Match &lhs, const Match &rhs)
{
  nodeid_t startNode;
  nodeid_t endNode;
  if(segmentation)
  {
    startNode = lhs.node;
    endNode = rhs.node;
  }
  else
  {
    startNode = tokHelper.rightTokenForNode(lhs.node);
    endNode = tokHelper.leftTokenForNode(rhs.node);
  }

  if(gsOrder->isConnected(Init::initEdge(startNode, endNode),
                           minDistance, maxDistance))
  {
    return true;
  }
  return false;

}

std::string Precedence::description() 
{
  if(minDistance == 1 && maxDistance == 1)
  {
    if(segmentation)
    {
      return "." + *segmentation;
    }
    else
    {
      return ".";
    }
  }
  else if(minDistance == 0 && maxDistance == 0)
  {
    if(segmentation)
    {
      return "." + *segmentation + "*";
    }
    else
    {
      return ".*";
    }
  }
  else if(minDistance == maxDistance)
  {
    if(segmentation)
    {
      return "." + *segmentation + "," + std::to_string(minDistance);
    }
    else
    {
      return "." + std::to_string(minDistance);
    }
  }
  else
  {
    if(segmentation)
    {
      return "." + *segmentation + "," + std::to_string(minDistance) + "," + std::to_string(maxDistance);
    }
    else
    {
      return "." + std::to_string(minDistance) + "," + std::to_string(maxDistance);
    }
  }
}

double Precedence::selectivity() 
{
  if(gsOrder == nullptr)
  {
    return Operator::selectivity();
  }
  
  GraphStatistic stats = gsOrder->getStatistics();
  unsigned int maxPossibleDist = std::min(maxDistance, stats.maxDepth);
  unsigned int numOfDescendants = maxPossibleDist - minDistance + 1;
  return (double) numOfDescendants / (double) (stats.nodes/2);
}


Precedence::~Precedence()
{

}
