#include "overlap.h"
#include "wrapper.h"

using namespace annis;

Overlap::Overlap(DB &db)
  : db(db), tokHelper(db), anyNodeAnno(Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID()))
{
  edbOrder = db.getEdgeDB(ComponentType::ORDERING, annis_ns, "");
  edbCoverage = db.getEdgeDB(ComponentType::COVERAGE, annis_ns, "");
}

std::unique_ptr<AnnoIt> Overlap::retrieveMatches(const annis::Match &lhs)
{
  ListWrapper* w = new ListWrapper();
  std::unique_ptr<AnnoIt> result(w);

  std::set<nodeid_t> uniqueResultSet;

  // get covered token of lhs
  EdgeIterator* coveredByLeftIt = edbCoverage->findConnected(lhs.node);
  for(auto leftToken = coveredByLeftIt->next();
      leftToken.first; leftToken = coveredByLeftIt->next())
  {

    // get all nodes that are covering the token
    std::vector<nodeid_t> overlapCandidates = edbCoverage->getIncomingEdges(leftToken.second);
    for(const auto& c : overlapCandidates)
    {
      uniqueResultSet.insert(c);
    }
     // also add the token itself
    uniqueResultSet.insert(leftToken.second);
  }
  delete coveredByLeftIt;

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

  if(edbOrder->distance(Init::initEdge(lhsLeftToken, rhsRightToken)) >= 0
     && edbOrder->distance(Init::initEdge(rhsLeftToken, lhsRightToken)) >= 0)
  {
    return true;
  }
  return false;
}

Overlap::~Overlap()
{

}
