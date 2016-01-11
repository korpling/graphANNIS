/* 
 * File:   IdenticalCoverage.cpp
 * Author: thomas
 * 
 * Created on 8. Januar 2016, 13:58
 */

#include "identicalcoverage.h"
#include "wrapper.h"

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
}

bool IdenticalCoverage::filter(const Match& lhs, const Match& rhs)
{
  return tokHelper.leftTokenForNode(lhs.node) == tokHelper.leftTokenForNode(rhs.node)
    && tokHelper.rightTokenForNode(lhs.node) == tokHelper.rightTokenForNode(rhs.node);
}

std::unique_ptr<AnnoIt> IdenticalCoverage::retrieveMatches(const Match& lhs)
{
  ListWrapper* w = new ListWrapper();
  
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
  
  // add the connected token itself as a match the span covers only one token
  if(leftToken == rightToken)
  {
    w->addMatch({leftToken, anyNodeAnno});
  }
  
  // find each node that is left-aligned with the left token and right aligned with the right token
  auto leftAligned = gsLeftToken->getOutgoingEdges(leftToken);
  auto rightAligned = gsRightToken->getOutgoingEdges(rightToken);
  std::sort(leftAligned.begin(), leftAligned.end());
  std::sort(rightAligned.begin(), rightAligned.end());
  std::vector<nodeid_t> resultIDs(std::max(leftAligned.size(), rightAligned.size()));
  
  auto it = 
    std::set_intersection(leftAligned.begin(), leftAligned.end(), 
    rightAligned.begin(), rightAligned.end(), 
    resultIDs.begin());
  resultIDs.resize(it - resultIDs.begin());

  for(const auto& matchedID : resultIDs)
  {
    w->addMatch({matchedID, anyNodeAnno});

  }
  
  return std::unique_ptr<AnnoIt>(w);
}



IdenticalCoverage::~IdenticalCoverage()
{
}

