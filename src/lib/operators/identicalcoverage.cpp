/* 
 * File:   IdenticalCoverage.cpp
 * Author: thomas
 * 
 * Created on 8. Januar 2016, 13:58
 */

#include <annis/operators/identicalcoverage.h>
#include <annis/wrapper.h>

#include <set>
#include <vector>
#include <algorithm>

using namespace annis;

IdenticalCoverage::IdenticalCoverage(const DB &db)
: db(db), tokHelper(db),
  anyNodeAnno(Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID()))
{
  gsOrder = db.getGraphStorage(ComponentType::ORDERING, annis_ns, "");
  gsLeftToken = db.getGraphStorage(ComponentType::LEFT_TOKEN, annis_ns, "");
  gsRightToken = db.getGraphStorage(ComponentType::RIGHT_TOKEN, annis_ns, "");
  gsCoverage = db.getGraphStorage(ComponentType::COVERAGE, annis_ns, "");
}

bool IdenticalCoverage::filter(const Match& lhs, const Match& rhs)
{
  auto lhsTokRange = tokHelper.leftRightTokenForNode(lhs.node);
  auto rhsTokRange = tokHelper.leftRightTokenForNode(rhs.node);
  return lhsTokRange == rhsTokRange;
}

std::unique_ptr<AnnoIt> IdenticalCoverage::retrieveMatches(const Match& lhs)
{ 
  nodeid_t leftToken;
  nodeid_t rightToken;
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
  }
  
  // find each non-token node that is left-aligned with the left token and right aligned with the right token
  auto leftAligned = gsLeftToken->getOutgoingEdges(leftToken);
  bool includeToken = leftToken == rightToken;
  
  // check for shortcuts where we don't need to return a complicated ListWrapper
  if(includeToken && leftAligned.empty())
  {
    // we only need to return a single match
    return std::unique_ptr<SingleElementWrapper>(new SingleElementWrapper(Init::initMatch(anyNodeAnno, leftToken)));
  }
  else if(!includeToken &&  leftAligned.size() == 1)
  {
    // check if also right aligned
    auto candidateRight = gsRightToken->getOutgoingEdges(leftAligned[0])[0];
    if(candidateRight == rightToken)
    {
      return std::unique_ptr<SingleElementWrapper>(new SingleElementWrapper(Init::initMatch(anyNodeAnno, leftAligned[0])));
    }
    else
    {
      // empty result
      return std::unique_ptr<AnnoIt>();
    }
  } // end shortcuts
  
  // use the ListWrapper as default case for matches with more than one result
  std::unique_ptr<ListWrapper> w = std::unique_ptr<ListWrapper>(new ListWrapper());
  
  // add the connected token itself as a match the span covers only one token
  if(includeToken)
  {
    w->addMatch({leftToken, anyNodeAnno});
  }
  
  for(const auto& candidate : leftAligned)
  {
    // check if also right aligned
    auto candidateRight = gsRightToken->getOutgoingEdges(candidate)[0];
    if(candidateRight == rightToken)
    {
      w->addMatch({candidate, anyNodeAnno});
    }
  }

  return w;
}

double IdenticalCoverage::selectivity() 
{
  if(gsOrder == nullptr || gsCoverage == nullptr)
  {
    return Operator::selectivity();
  }
  auto statsCov = gsCoverage->getStatistics();
  auto statsOrder = gsOrder->getStatistics();
  if(statsCov.nodes == 0)
  {
    // only token in this corpus
    return 1.0 / (double) statsOrder.nodes;
  }
  else
  {
    // The fan-out is the selectivity for the number of covered token.
    // Use a constant that dependends on the number of token to estimate the number of included
    // nodes.
    // TODO: which statistics do we need to calculate the better number?
    return statsCov.avgFanOut * 0.8; 
  }
}


IdenticalCoverage::~IdenticalCoverage()
{
}

