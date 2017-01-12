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

#include <annis/operators/inclusion.h>

#include <annis/wrapper.h>

using namespace annis;

Inclusion::Inclusion(const DB &db, GraphStorageHolder& gsh)
  : db(db),
    anyNodeAnno(Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID())),
    tokHelper(gsh, db)
{
  gsOrder = gsh.getGraphStorage(ComponentType::ORDERING, annis_ns, "");
  gsLeftToken = gsh.getGraphStorage(ComponentType::LEFT_TOKEN, annis_ns, "");
  gsRightToken = gsh.getGraphStorage(ComponentType::RIGHT_TOKEN, annis_ns, "");
  gsCoverage = gsh.getGraphStorage(ComponentType::COVERAGE, annis_ns, "");

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
  if(db.nodeAnnos.getAnnotations(db.strings, lhs.node, annis_ns, annis_tok).empty() == false)
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
    return ((statsCov.avgFanOut) / numOfToken);
  }
}



Inclusion::~Inclusion()
{
}


