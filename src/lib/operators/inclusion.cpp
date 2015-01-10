#include "inclusion.h"

#include "../componenttypeiterator.h"
#include "wrapper.h"


using namespace annis;

Inclusion::Inclusion(DB &db)
  : db(db),
    anyNodeAnno(Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID())),
    tokHelper(db)
{
  edbCoverage = db.getAllEdgeDBForType(ComponentType::COVERAGE);
  edbOrder = db.getEdgeDB(ComponentType::ORDERING, annis_ns, "");
  edbLeftToken = db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, "");
  edbRightToken = db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, "");

}

bool Inclusion::filter(const Match &lhs, const Match &rhs)
{
  nodeid_t lhsLeftToken = tokHelper.leftTokenForNode(lhs.node);
  nodeid_t lhsRightToken = tokHelper.rightTokenForNode(lhs.node);
  int spanLength = spanLength = edbOrder->distance(Init::initEdge(lhsLeftToken, lhsRightToken));

  nodeid_t rhsLeftToken = tokHelper.leftTokenForNode(rhs.node);
  nodeid_t rhsRightToken = tokHelper.rightTokenForNode(rhs.node);

  if(edbOrder->isConnected(Init::initEdge(lhsLeftToken, rhsLeftToken), 0, spanLength)
     && edbOrder->isConnected(Init::initEdge(lhsLeftToken, rhsRightToken)), 0, spanLength)
  {
    return true;
  }
  return false;
}


std::unique_ptr<AnnoIt> Inclusion::retrieveMatches(const annis::Match &lhs)
{
  std::unique_ptr<AnnoIt> result(nullptr);

  ListWrapper* w = new ListWrapper();
  result.reset(w);

  nodeid_t leftToken;
  nodeid_t rightToken;
  int spanLength = 0;
  if(db.getNodeAnnotation(lhs.node, annis_ns, annis_tok).first)
  {
    // is token
    leftToken = lhs.node;
    rightToken = lhs.node;
  }
  else
  {
    leftToken = edbLeftToken->getOutgoingEdges(lhs.node)[0];
    rightToken = edbRightToken->getOutgoingEdges(lhs.node)[0];
    spanLength = edbOrder->distance(Init::initEdge(leftToken, rightToken));
  }

  // find each token which is between the left and right border
  std::unique_ptr<EdgeIterator> itIncludedStart = edbOrder->findConnected(leftToken, 0, spanLength);
  for(std::pair<bool, nodeid_t> includedStart = itIncludedStart->next();
      includedStart.first;
      includedStart = itIncludedStart->next())
  {
    // add the token itself
    w->addMatch(Init::initMatch(anyNodeAnno, includedStart.second));

    // add aligned nodes
    for(const auto& leftAlignedNode : edbLeftToken->getOutgoingEdges(includedStart.second))
    {
      nodeid_t includedEndCandiate = edbRightToken->getOutgoingEdges(leftAlignedNode)[0];
      if(edbOrder->isConnected(Init::initEdge(includedEndCandiate, rightToken), 0, uintmax))
      {
        w->addMatch(Init::initMatch(anyNodeAnno, leftAlignedNode));
      }
    }
  }

  return result;
}


Inclusion::~Inclusion()
{
}


