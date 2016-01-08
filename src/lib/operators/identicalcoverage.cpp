/* 
 * File:   IdenticalCoverage.cpp
 * Author: thomas
 * 
 * Created on 8. Januar 2016, 13:58
 */

#include "identicalcoverage.h"
#include "wrapper.h"

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
    spanLength = gsOrder->distance(Init::initEdge(leftToken, rightToken));
  }

  // find each token which is between the left and right border
  std::unique_ptr<EdgeIterator> itTokBetween = gsOrder->findConnected(leftToken, 0, spanLength);
  for(std::pair<bool, nodeid_t> tokBetween = itTokBetween->next();
      tokBetween.first;
      tokBetween = itTokBetween->next())
  {
    // add the token itself
    w->addMatch({tokBetween.second, anyNodeAnno});
    
    // add all right aligned nodes
    for(const auto& rightAlignedNode : gsRightToken->getOutgoingEdges(tokBetween.second))
    {
      w->addMatch({rightAlignedNode, anyNodeAnno});
    }

  }
  
  return std::unique_ptr<AnnoIt>(w);
}



IdenticalCoverage::~IdenticalCoverage()
{
}

