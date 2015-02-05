#include "overlap.h"
#include "../wrapper.h"

#include <stx/btree_set>
using namespace annis;

Overlap::Overlap(const DB &db)
  : db(db), tokHelper(db), anyNodeAnno(Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID()))
{
  gsOrder = db.getGraphStorage(ComponentType::ORDERING, annis_ns, "");
  gsCoverage = db.getGraphStorage(ComponentType::COVERAGE, annis_ns, "");
  gsInverseCoverage = db.getGraphStorage(ComponentType::INVERSE_COVERAGE, annis_ns, "");

}

std::unique_ptr<AnnoIt> Overlap::retrieveMatches(const annis::Match &lhs)
{
  ListWrapper* w = new ListWrapper();
  std::unique_ptr<AnnoIt> result(w);

  stx::btree_set<nodeid_t> uniqueResultSet;

  // get covered token of lhs
  std::unique_ptr<EdgeIterator> coveredByLeftIt
      = gsCoverage->findConnected(lhs.node);
  for(auto leftToken = coveredByLeftIt->next();
      leftToken.first; leftToken = coveredByLeftIt->next())
  {

    // get all nodes that are covering the token
    std::vector<nodeid_t> overlapCandidates = gsInverseCoverage->getOutgoingEdges(leftToken.second);
    for(const auto& c : overlapCandidates)
    {
      uniqueResultSet.insert(c);
    }
     // also add the token itself
    uniqueResultSet.insert(leftToken.second);
  }

  // add all unique matches to result
  for(const auto& m : uniqueResultSet)
  {
    w->addMatch(Init::initMatch(anyNodeAnno, m));
  }

  return result;
}

bool Overlap::filter(const Match &lhs, const Match &rhs)
{
  nodeid_t lhsLeftToken = tokHelper.leftTokenForNode(lhs.node);
  nodeid_t lhsRightToken = tokHelper.rightTokenForNode(lhs.node);
  nodeid_t rhsLeftToken = tokHelper.leftTokenForNode(rhs.node);
  nodeid_t rhsRightToken = tokHelper.rightTokenForNode(rhs.node);

  if(gsOrder->distance(Init::initEdge(lhsLeftToken, rhsRightToken)) >= 0
     && gsOrder->distance(Init::initEdge(rhsLeftToken, lhsRightToken)) >= 0)
  {
    return true;
  }
  return false;
}

Overlap::~Overlap()
{

}
