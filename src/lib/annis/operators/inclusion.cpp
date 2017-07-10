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

#include "inclusion.h"

#include <annis/wrapper.h>                    // for ListWrapper
#include <boost/container/vector.hpp>         // for operator!=
#include <utility>                            // for pair, move
#include <vector>                             // for vector
#include "annis/annostorage.h"                // for AnnoStorage
#include "annis/db.h"                         // for DB
#include "annis/graphstorage/graphstorage.h"  // for ReadableGraphStorage
#include "annis/iterators.h"                  // for EdgeIterator, AnnoIt
#include "annis/operators/operator.h"         // for Operator
#include "annis/util/helper.h"                // for TokenHelper
#include <annis/types.h>


using namespace annis;

Inclusion::Inclusion(const DB &db, DB::GetGSFuncT getGSFunc)
  : db(db),
    anyNodeAnno(Init::initAnnotation(db.getNodeTypeStringID(), 0, db.getNamespaceStringID())),
    tokHelper(getGSFunc, db)
{
  gsOrder = getGSFunc(ComponentType::ORDERING, annis_ns, "");
  gsLeftToken = getGSFunc(ComponentType::LEFT_TOKEN, annis_ns, "");
  gsRightToken = getGSFunc(ComponentType::RIGHT_TOKEN, annis_ns, "");
  gsCoverage = getGSFunc(ComponentType::COVERAGE, annis_ns, "");

}

bool Inclusion::filter(const Match &lhs, const Match &rhs)
{
  auto lhsTokenRange = tokHelper.leftRightTokenForNode(lhs.node);
  int spanLength = gsOrder->distance({lhsTokenRange.first, lhsTokenRange.second});

  auto rhsTokenRange = tokHelper.leftRightTokenForNode(rhs.node);

  if(gsOrder->isConnected({lhsTokenRange.first, rhsTokenRange.first}, 0, spanLength)
     && gsOrder->isConnected({rhsTokenRange.second, lhsTokenRange.second}, 0, spanLength)
    )
  {
    return true;
  }
  return false;
}


std::unique_ptr<AnnoIt> Inclusion::retrieveMatches(const annis::Match &lhs)
{

  std::unique_ptr<ListWrapper> w = std::unique_ptr<ListWrapper>(new ListWrapper());

  nodeid_t leftToken;
  nodeid_t rightToken;
  int spanLength = 0;
  if(tokHelper.isToken(lhs.node))
  {
    // is token
    leftToken = lhs.node;
    rightToken = lhs.node;
  }
  else
  {
    leftToken = gsLeftToken->getOutgoingEdges(lhs.node)[0];
    rightToken = gsRightToken->getOutgoingEdges(lhs.node)[0];
    spanLength = gsOrder->distance({leftToken, rightToken});
  }

  // find each token which is between the left and right border
  std::unique_ptr<EdgeIterator> itIncludedStart = gsOrder->findConnected(leftToken, 0, spanLength);
  for(std::pair<bool, nodeid_t> includedStart = itIncludedStart->next();
      includedStart.first;
      includedStart = itIncludedStart->next())
  {
    const nodeid_t& includedTok = includedStart.second;
    // add the token itself
    w->addMatch({includedTok, anyNodeAnno});

    // add aligned nodes
    for(const auto& leftAlignedNode : gsLeftToken->getOutgoingEdges(includedTok))
    {
      std::vector<nodeid_t> outEdges = gsRightToken->getOutgoingEdges(leftAlignedNode);
      if(!outEdges.empty())
      {
        nodeid_t includedEndCandiate = outEdges[0];
        if(gsOrder->isConnected({includedEndCandiate, rightToken}, 0, spanLength))
        {
          w->addMatch({leftAlignedNode, anyNodeAnno});
        }
      }
    }
  }

  return std::move(w);
}

double Inclusion::selectivity() 
{
  if(gsOrder == nullptr || gsCoverage == nullptr)
  {
    return Operator::selectivity();
  }

  auto statsCov = gsCoverage->getStatistics();
  auto statsOrder = gsOrder->getStatistics();


  double numOfToken = statsOrder.nodes;

  if(statsCov.nodes == 0)
  {
    // only token in this corpus
    return 1.0 / numOfToken;
  }
  else
  {

    // Assume two nodes have inclusion coverage if the left- and right-most covered token is inside the
    // covered range of the other node.
    return ( (double) statsCov.fanOut95Percentile / (double) numOfToken);
  }
}



Inclusion::~Inclusion()
{
}


