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
#include <annis/wrapper.h>

using namespace annis;


Precedence::Precedence(const DB &db, GraphStorageHolder& gsh, unsigned int minDistance, unsigned int maxDistance)
  : tokHelper(gsh, db),
    gsOrder(gsh.getGraphStorage(ComponentType::ORDERING, annis_ns, "")),
    gsLeft(gsh.getGraphStorage(ComponentType::LEFT_TOKEN, annis_ns, "")),
    anyTokAnno(Init::initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID())),
    anyNodeAnno(Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID())),
    minDistance(minDistance), maxDistance(maxDistance)
{
}

std::unique_ptr<AnnoIt> Precedence::retrieveMatches(const Match &lhs)
{
  std::unique_ptr<ListWrapper> w = std::unique_ptr<ListWrapper>(new ListWrapper());

  nodeid_t lhsRightToken = tokHelper.rightTokenForNode(lhs.node);
  std::unique_ptr<EdgeIterator> edgeIterator = gsOrder->findConnected(lhsRightToken,
                                                       minDistance, maxDistance);

  // materialize a list of all matches and wrap it
  for(std::pair<bool, nodeid_t> matchedToken = edgeIterator->next();
      matchedToken.first; matchedToken = edgeIterator->next())
  {
    // get all nodes that are left-aligned to this token
    for(const auto& n : gsLeft->getOutgoingEdges(matchedToken.second))
    {
      w->addMatch(Init::initMatch(anyNodeAnno, n));
    }
    // add the actual token to the list as well
    w->addMatch(Init::initMatch(anyNodeAnno, matchedToken.second));
  }

  return std::move(w);
}

bool Precedence::filter(const Match &lhs, const Match &rhs)
{
  nodeid_t lhsRightToken = tokHelper.rightTokenForNode(lhs.node);
  nodeid_t rhsLeftToken = tokHelper.leftTokenForNode(rhs.node);
  if(gsOrder->isConnected(Init::initEdge(lhsRightToken, rhsLeftToken),
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
    return ".";
  }
  else if(minDistance == 0 && maxDistance == 0)
  {
    return ".*";
  }
  else if(minDistance == maxDistance)
  {
    return "." + std::to_string(minDistance);
  }
  else
  {
    return "." + std::to_string(minDistance) + "," + std::to_string(maxDistance);
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
