#include <annis/operators/inclusion.h>

#include <annis/wrapper.h>

using namespace annis;

Inclusion::Inclusion(const DB &db)
  : db(db),
    anyNodeAnno(Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID())),
    tokHelper(db)
{
  gsOrder = db.getGraphStorage(ComponentType::ORDERING, annis_ns, "");
  gsLeftToken = db.getGraphStorage(ComponentType::LEFT_TOKEN, annis_ns, "");
  gsRightToken = db.getGraphStorage(ComponentType::RIGHT_TOKEN, annis_ns, "");

}

bool Inclusion::filter(const Match &lhs, const Match &rhs)
{
  nodeid_t lhsLeftToken = tokHelper.leftTokenForNode(lhs.node);
  nodeid_t lhsRightToken = tokHelper.rightTokenForNode(lhs.node);
  int spanLength = spanLength = gsOrder->distance(Init::initEdge(lhsLeftToken, lhsRightToken));

  nodeid_t rhsLeftToken = tokHelper.leftTokenForNode(rhs.node);
  nodeid_t rhsRightToken = tokHelper.rightTokenForNode(rhs.node);

  if(gsOrder->isConnected(Init::initEdge(lhsLeftToken, rhsLeftToken), 0, spanLength)
     && gsOrder->isConnected(Init::initEdge(rhsRightToken, lhsRightToken), 0, spanLength)
    )
  {
    return true;
  }
  return false;
}


std::unique_ptr<AnnoIt> Inclusion::retrieveMatches(const annis::Match &lhs)
{

  std::unique_ptr<ListWrapper> w = std::make_unique<ListWrapper>();

  nodeid_t leftToken;
  nodeid_t rightToken;
  int spanLength = 0;
  if(db.nodeAnnos.getNodeAnnotation(lhs.node, annis_ns, annis_tok).first)
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
  std::unique_ptr<EdgeIterator> itIncludedStart = gsOrder->findConnected(leftToken, 0, spanLength);
  for(std::pair<bool, nodeid_t> includedStart = itIncludedStart->next();
      includedStart.first;
      includedStart = itIncludedStart->next())
  {
    const nodeid_t& includedTok = includedStart.second;
    // add the token itself
    w->addMatch(Init::initMatch(anyNodeAnno, includedTok));

    // add aligned nodes
    for(const auto& leftAlignedNode : gsLeftToken->getOutgoingEdges(includedTok))
    {
      nodeid_t includedEndCandiate = gsRightToken->getOutgoingEdges(leftAlignedNode)[0];
      if(gsOrder->isConnected(Init::initEdge(includedEndCandiate, rightToken), 0, spanLength))
      {
        w->addMatch(Init::initMatch(anyNodeAnno, leftAlignedNode));
      }
    }
  }

  return w;
}


Inclusion::~Inclusion()
{
}


