#include "precedence.h"
#include "../wrapper.h"

using namespace annis;


Precedence::Precedence(const DB &db, unsigned int minDistance, unsigned int maxDistance)
  : tokHelper(db),
    gsOrder(db.getGraphStorage(ComponentType::ORDERING, annis_ns, "")),
    gsLeft(db.getGraphStorage(ComponentType::LEFT_TOKEN, annis_ns, "")),
    anyTokAnno(Init::initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID())),
    anyNodeAnno(Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID())),
    minDistance(minDistance), maxDistance(maxDistance)
{
}

std::unique_ptr<AnnoIt> Precedence::retrieveMatches(const Match &lhs)
{
  std::unique_ptr<AnnoIt> result(nullptr);

  nodeid_t lhsRightToken = tokHelper.rightTokenForNode(lhs.node);
  std::unique_ptr<EdgeIterator> edgeIterator = gsOrder->findConnected(lhsRightToken,
                                                       minDistance, maxDistance);

  ListWrapper* w = new ListWrapper();
  result.reset(w);
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

  return result;
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

Precedence::~Precedence()
{

}
